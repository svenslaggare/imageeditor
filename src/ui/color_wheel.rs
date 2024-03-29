use glfw::Action;
use cgmath::Matrix4;

use image::FilterType;

use crate::command_buffer::{CommandBuffer, Command};
use crate::rendering::texture::Texture;
use crate::rendering::prelude::{Position, Rectangle, Size};
use crate::rendering::prelude::Color4 as RenderingColor4;
use crate::ui::button::{GenericButton};
use crate::editor::tools::{EditorWindow, SelectColorMode};
use crate::program::Renders;
use crate::editor::image_operation_helpers::{hsv_to_rgb, rgb_to_hsv};
use crate::editor;

pub struct ColorWheel {
    hue_wheel_texture: Texture,
    hue_wheel_image: image::RgbaImage,
    saturation_value_texture: Texture,
    saturation_value_image: image::RgbaImage,
    position: Position,
    started_selecting_hue: bool,
    mode: SelectColorMode,
    started_selecting_color: bool
}

const SATURATION_VALUE_IMAGE_WIDTH: u32 = 100;
const SATURATION_VALUE_IMAGE_HEIGHT: u32 = 100;

impl ColorWheel {
    pub fn new() -> ColorWheel {
        let hue_wheel_image = create_hue_selector(100, 2);
        let saturation_value_image = create_saturation_value_selector(SATURATION_VALUE_IMAGE_WIDTH, SATURATION_VALUE_IMAGE_HEIGHT, 0.0);

        ColorWheel {
            hue_wheel_texture: Texture::from_image(&hue_wheel_image),
            hue_wheel_image,
            saturation_value_texture: Texture::from_image(&saturation_value_image),
            saturation_value_image,
            position: Position::new(0.0, 0.0),
            started_selecting_hue: false,
            mode: SelectColorMode::PrimaryColor,
            started_selecting_color: false,
        }
    }

    pub fn set_mode(&mut self, mode: SelectColorMode) {
        self.mode = mode;
    }

    pub fn update_position(&mut self, window: &dyn EditorWindow) {
        self.position = Position::new(
            0.5 * window.width() as f32 - 0.5 * self.hue_wheel_image.width() as f32,
            0.5 * window.height() as f32 - 0.5 * self.hue_wheel_image.height() as f32
        );
    }

    pub fn select_color(&mut self, window: &dyn EditorWindow, event: &glfw::WindowEvent) -> Option<editor::Color> {
        self.update_position(window);

        let bounding_rectangle = Rectangle::new(
            self.position.x,
            self.position.y,
            self.hue_wheel_texture.width() as f32,
            self.hue_wheel_texture.height() as f32
        );

        let select_hue = || {
            let mouse_position = window.get_cursor_pos();
            if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                let x = (mouse_position.0 - self.position.x as f64) as i32;
                let y = (mouse_position.1 - self.position.y as f64) as i32;
                if x >= 0 && x < self.hue_wheel_image.width() as i32 && y >= 0 && y < self.hue_wheel_image.height() as i32 {
                    let color = self.hue_wheel_image.get_pixel(x as u32, y as u32);
                    if color != &image::Rgba([0u8, 0u8, 0u8, 0u8]) {
                        return Some(*color);
                    }
                }
            }

            None
        };

        let select_color = || {
            let mouse_position = window.get_cursor_pos();
            if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                let x = (mouse_position.0 - (self.position.x as f64 + 0.5 * self.saturation_value_image.width() as f64)) as i32;
                let y = (mouse_position.1 - (self.position.y as f64 + 0.5 * self.saturation_value_image.height() as f64)) as i32;
                if x >= 0 && x < self.saturation_value_image.width() as i32 && y >= 0 && y < self.saturation_value_image.height() as i32 {
                    let color = self.saturation_value_image.get_pixel(x as u32, y as u32);
                    if color != &image::Rgba([0u8, 0u8, 0u8, 0u8]) {
                        return Some(*color);
                    }
                }
            }

            None
        };

        let mut selected_color = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                let mut started_selecting_color = false;
                if let Some(color) = select_color() {
                    selected_color = Some(color);
                    started_selecting_color = true;
                }

                if let Some(color) = select_hue() {
                    self.saturation_value_image = create_saturation_value_selector(SATURATION_VALUE_IMAGE_WIDTH, SATURATION_VALUE_IMAGE_HEIGHT, rgb_to_hsv(color).0);
                    self.saturation_value_texture.upload(&self.saturation_value_image.as_ref());
                    self.started_selecting_hue = true;
                }

                if started_selecting_color {
                    self.started_selecting_color = started_selecting_color;
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.started_selecting_hue = false;
                self.started_selecting_color = false;
            }
            glfw::WindowEvent::CursorPos(_, _) => {
                if self.started_selecting_color {
                    selected_color = select_color();
                }

                if self.started_selecting_hue {
                    if let Some(color) = select_hue() {
                        self.saturation_value_image = create_saturation_value_selector(SATURATION_VALUE_IMAGE_WIDTH, SATURATION_VALUE_IMAGE_HEIGHT, rgb_to_hsv(color).0);
                        self.saturation_value_texture.upload(&self.saturation_value_image.as_ref());
                    }
                }
            }
            _ => {}
        }

        selected_color
    }
}

impl GenericButton<CommandBuffer> for ColorWheel {
    fn process_gui_event(&mut self, window: &dyn EditorWindow, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
        if let Some(color) = self.select_color(window, event) {
            match self.mode {
                SelectColorMode::PrimaryColor => {
                    command_buffer.push(Command::SetPrimaryColor(color));
                }
                SelectColorMode::SecondaryColor => {
                    command_buffer.push(Command::SetSecondaryColor(color));
                }
            }
        }
    }

    fn process_command(&mut self, _command: &Command) {

    }

    fn render(&self, renders: &Renders, transform: &Matrix4<f32>) {
        let buffer = 10.0;
        let background_start = Position::new(self.position.x - buffer, self.position.y - buffer);
        let background_size = Size::new(
            self.hue_wheel_image.width() as f32 + buffer * 2.0,
            self.hue_wheel_texture.height() as f32 + buffer * 2.0
        );

        renders.solid_rectangle_render.render(
            renders.solid_rectangle_render.shader(),
            transform,
            &Rectangle::from_position_and_size(
                background_start,
                background_size,
            ),
            RenderingColor4::new(214, 214, 214, 255)
        );

        renders.rectangle_render.render(
            renders.rectangle_render.shader(),
            transform,
            &Rectangle::new(background_start.x, background_start.y, background_size.x, background_size.y),
            RenderingColor4::new(128, 128, 128, 255)
        );

        renders.texture_render.render(
            renders.texture_render.shader(),
            &transform,
            &self.hue_wheel_texture,
            self.position
        );

        renders.texture_render.render(
            renders.texture_render.shader(),
            &transform,
            &self.saturation_value_texture,
            Position::new(
                self.position.x + 0.5 * self.saturation_value_image.width() as f32,
                self.position.y + 0.5 * self.saturation_value_image.height() as f32
            )
        );
    }
}

fn create_hue_selector(max_radius: i32, sample: i32) -> image::RgbaImage {
    let scaled_max_radius = max_radius * sample;

    let mut wheel_image: image::RgbaImage = image::RgbaImage::new(
        (scaled_max_radius * 2) as u32,
        (scaled_max_radius * 2) as u32
    );

    let mut hue = 0.0;
    while hue <= 360.0 {
        if let Some(color) = hsv_to_rgb(hue as f64, 100.0, 100.0) {
            let hue_angle = hue as f64 * (std::f64::consts::PI / 180.0);

            for radius in (scaled_max_radius - 15 * sample)..scaled_max_radius {
                let radius = radius as f64;
                let x = (scaled_max_radius as f64 + radius * hue_angle.cos()).round() as i32;
                let y = (scaled_max_radius as f64 + radius * hue_angle.sin()).round() as i32;

                if x >= 0 && x < wheel_image.width() as i32 && y >= 0 && y < wheel_image.height() as i32 {
                    *wheel_image.get_pixel_mut(x as u32, y as u32) = color;
                }
            }
        }

        hue += 0.1;
    }

    if sample > 1 {
        wheel_image = image::imageops::resize(
            &wheel_image,
            max_radius as u32 * 2,
            max_radius as u32 * 2,
            FilterType::Triangle
        );
    }

    wheel_image
}

fn create_saturation_value_selector(width: u32, height: u32, hue: f64) -> image::RgbaImage {
    let mut wheel_image: image::RgbaImage = image::RgbaImage::new(width, height);

    let mut saturation = 0.0f64;

    let scale_x = width as f64 / 100.0;
    let scale_y = height as f64 / 100.0;

    while saturation <= 100.0 {
        let mut value = 0.0f64;
        while value <= 100.0 {
            if let Some(color) = hsv_to_rgb(hue, saturation, value) {
                let x = (value * scale_y).round() as i32;
                let y = (saturation * scale_x).round() as i32;

                if x >= 0 && x < wheel_image.width() as i32 && y >= 0 && y < wheel_image.height() as i32 {
                    *wheel_image.get_pixel_mut(x as u32, y as u32) = color;
                }
            }

            value += 0.1;
        }

        saturation += 0.1;
    }

    wheel_image
}