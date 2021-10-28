use std::ops::DerefMut;

use glfw::{WindowEvent, Action};
use cgmath::Matrix4;

use crate::rendering::texture::Texture;
use crate::rendering::prelude::{Position, Rectangle};
use crate::rendering::prelude::Color as RenderingColor;
use crate::rendering::prelude::Color4 as RenderingColor4;
use crate::rendering::texture_render::TextureRender;
use crate::rendering::shader::Shader;
use crate::command_buffer::{CommandBuffer, Command};
use crate::rendering::text_render::{TextRender, TextAlignment};
use crate::rendering::font::{Font, FontRef};
use crate::editor::Color;
use crate::rendering::solid_rectangle_render::SolidRectangleRender;
use crate::rendering::ShaderAndRender;
use crate::program::Renders;
use crate::editor::tools::EditorWindow;

pub type ButtonAction<T> = Box<dyn Fn(&mut T)>;
pub type CommandAction<T> = Box<dyn Fn(&mut T, &Command)>;

pub trait GenericButton<T> {
    fn process_gui_event(&mut self, window: &mut dyn EditorWindow, event: &glfw::WindowEvent, argument: &mut T);
    fn process_command(&mut self, command: &Command);
    fn render(&self, renders: &Renders, transform: &Matrix4<f32>);
}

pub struct TextureButton<T=CommandBuffer> {
    texture: Texture,
    position: Position,
    left_click_action: Option<ButtonAction<T>>,
    right_click_action: Option<ButtonAction<T>>,
    command_action: Option<CommandAction<Self>>
}

impl<T> TextureButton<T> {
    pub fn new(image: &image::RgbaImage,
               position: Position,
               left_click_action: Option<ButtonAction<T>>,
               right_click_action: Option<ButtonAction<T>>,
               command_action: Option<CommandAction<Self>>) -> TextureButton<T> {
        TextureButton {
            texture: Texture::from_image(image),
            position,
            left_click_action,
            right_click_action,
            command_action
        }
    }
}

impl<T> GenericButton<T> for TextureButton<T> {
    fn process_gui_event(&mut self, window: &mut dyn EditorWindow, event: &glfw::WindowEvent, argument: &mut T) {
        let bounding_rectangle = Rectangle::new(
            self.position.x,
            self.position.y,
            self.texture.width() as f32,
            self.texture.height() as f32
        );

        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                if let Some(left_click_action) = self.left_click_action.as_ref() {
                    let mouse_position = window.get_cursor_pos();
                    if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (left_click_action)(argument);
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Release, _) => {
                if let Some(right_click_action) = self.right_click_action.as_ref() {
                    let mouse_position = window.get_cursor_pos();
                    if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (right_click_action)(argument);
                    }
                }
            }
            _ => {}
        }
    }

    fn process_command(&mut self, command: &Command) {
        let command_action = self.command_action.take();
        if let Some(command_action) = command_action.as_ref() {
            (command_action)(self, command);
        }
        self.command_action = command_action;
    }

    fn render(&self, renders: &Renders, transform: &Matrix4<f32>) {
        renders.texture_render.render(
            renders.texture_render.shader(),
            &transform,
            &self.texture,
            self.position
        );
    }
}

pub struct SolidColorButton<T=CommandBuffer> {
    color: Color,
    rectangle: Rectangle,
    left_click_action: Option<ButtonAction<T>>,
    right_click_action: Option<ButtonAction<T>>,
    command_action: Option<CommandAction<Self>>
}

impl<T> SolidColorButton<T> {
    pub fn new(color: Color,
               rectangle: Rectangle,
               left_click_action: Option<ButtonAction<T>>,
               right_click_action: Option<ButtonAction<T>>,
               command_action: Option<CommandAction<Self>>) -> SolidColorButton<T> {
        SolidColorButton {
            color,
            rectangle,
            left_click_action,
            right_click_action,
            command_action
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl<T> GenericButton<T> for SolidColorButton<T> {
    fn process_gui_event(&mut self, window: &mut dyn EditorWindow, event: &glfw::WindowEvent, argument: &mut T) {
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                if let Some(left_click_action) = self.left_click_action.as_ref() {
                    let mouse_position = window.get_cursor_pos();
                    if self.rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (left_click_action)(argument);
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Release, _) => {
                if let Some(right_click_action) = self.right_click_action.as_ref() {
                    let mouse_position = window.get_cursor_pos();
                    if self.rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (right_click_action)(argument);
                    }
                }
            }
            _ => {}
        }
    }

    fn process_command(&mut self, command: &Command) {
        let command_action = self.command_action.take();
        if let Some(command_action) = command_action.as_ref() {
            (command_action)(self, command);
        }
        self.command_action = command_action;
    }

    fn render(&self, renders: &Renders, transform: &Matrix4<f32>) {
        renders.solid_rectangle_render.render(
            renders.solid_rectangle_render.shader(),
            &transform,
            self.rectangle.position,
            self.rectangle.size,
            RenderingColor4::new(self.color[0], self.color[1], self.color[2], 255)
        );
    }
}

pub struct TextButton<T=CommandBuffer> {
    font: FontRef,
    text: String,
    position: Position,
    left_click_action: Option<ButtonAction<T>>,
    right_click_action: Option<ButtonAction<T>>,
    command_action: Option<CommandAction<Self>>
}

impl<T> TextButton<T> {
    pub fn new(font: FontRef,
               text: String,
               position: Position,
               left_click_action: Option<ButtonAction<T>>,
               right_click_action: Option<ButtonAction<T>>,
               command_action: Option<CommandAction<Self>>) -> TextButton<T> {
        TextButton {
            font,
            text,
            position,
            left_click_action,
            right_click_action,
            command_action
        }
    }

    pub fn change_text(&mut self, text: String) {
        self.text = text;
    }

    fn bounding_rectangle(&self) -> Rectangle {
        let mut font = self.font.borrow_mut();
        let width = font.line_width(&self.text);
        let height = font.line_height();

        Rectangle::new(
            self.position.x,
            self.position.y,
            width,
            height
        )
    }
}

impl<T> GenericButton<T> for TextButton<T> {
    fn process_gui_event(&mut self, window: &mut dyn EditorWindow, event: &glfw::WindowEvent, argument: &mut T) {
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                if let Some(left_click_action) = self.left_click_action.as_ref() {
                    let mouse_position = window.get_cursor_pos();
                    if self.bounding_rectangle().contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (left_click_action)(argument);
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Release, _) => {
                if let Some(right_click_action) = self.right_click_action.as_ref() {
                    let mouse_position = window.get_cursor_pos();
                    if self.bounding_rectangle().contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (right_click_action)(argument);
                    }
                }
            }
            _ => {}
        }
    }

    fn process_command(&mut self, command: &Command) {
        let command_action = self.command_action.take();
        if let Some(command_action) = command_action.as_ref() {
            (command_action)(self, command);
        }
        self.command_action = command_action;
    }

    fn render(&self, renders: &Renders, transform: &Matrix4<f32>) {
        // let bounding_rectangle = self.bounding_rectangle();
        // renders.solid_rectangle_render.render(
        //     renders.solid_rectangle_render.shader(),
        //     transform,
        //     bounding_rectangle.position,
        //     bounding_rectangle.size,
        //     RenderingColor::new(255, 0, 0)
        // );

        renders.text_render.draw_line(
            renders.text_render.shader(),
            transform,
            self.font.borrow_mut().deref_mut(),
            self.text.chars().map(|c| (c, RenderingColor::new(0, 0, 0))),
            self.position,
            TextAlignment::Top
        );
    }
}

pub struct Checkbox<T=CommandBuffer> {
    unchecked_texture: Texture,
    checked_texture: Texture,
    font: FontRef,
    text: String,
    pub checked: bool,
    position: Position,
    command_action: Option<CommandAction<Self>>
}

impl<T> Checkbox<T> {
    pub fn new(unchecked_image: &image::RgbaImage,
               checked_image: &image::RgbaImage,
               font: FontRef,
               text: String,
               checked: bool,
               position: Position,
               command_action: Option<CommandAction<Self>>) -> Checkbox<T> {
        Checkbox {
            unchecked_texture: Texture::from_image(unchecked_image),
            checked_texture: Texture::from_image(checked_image),
            font,
            text,
            checked,
            position,
            command_action
        }
    }
}

impl<T> GenericButton<T> for Checkbox<T> {
    fn process_gui_event(&mut self, window: &mut dyn EditorWindow, event: &glfw::WindowEvent, _argument: &mut T) {
        let bounding_rectangle = Rectangle::new(
            self.position.x,
            self.position.y,
            self.checked_texture.width() as f32,
            self.checked_texture.height() as f32
        );

        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                let mouse_position = window.get_cursor_pos();
                if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                    self.checked = !self.checked;
                }
            }
            _ => {}
        }
    }

    fn process_command(&mut self, command: &Command) {
        let command_action = self.command_action.take();
        if let Some(command_action) = command_action.as_ref() {
            (command_action)(self, command);
        }
        self.command_action = command_action;
    }

    fn render(&self, renders: &Renders, transform: &Matrix4<f32>) {
        if self.checked {
            renders.texture_render.render(
                renders.texture_render.shader(),
                &transform,
                &self.checked_texture,
                self.position
            );
        } else {
            renders.texture_render.render(
                renders.texture_render.shader(),
                &transform,
                &self.unchecked_texture,
                self.position
            );
        }

        renders.text_render.draw_line(
            renders.text_render.shader(),
            transform,
            self.font.borrow_mut().deref_mut(),
            self.text.chars().map(|c| (c, RenderingColor::new(0, 0, 0))),
            Position::new(self.position.x + 6.0 + self.unchecked_texture.width() as f32, self.position.y - 0.5 * self.unchecked_texture.height() as f32),
            TextAlignment::Top
        );
    }
}