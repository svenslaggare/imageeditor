use std::ops::DerefMut;

use glfw::{WindowEvent, Action};
use cgmath::Matrix4;

use crate::rendering::texture::Texture;
use crate::rendering::prelude::{Position, Rectangle};
use crate::rendering::prelude::Color as RenderingColor;
use crate::rendering::texture_render::TextureRender;
use crate::rendering::shader::Shader;
use crate::command_buffer::{CommandBuffer, Command};
use crate::rendering::text_render::{TextRender, TextAlignment};
use crate::rendering::font::{Font, FontRef};
use crate::editor::Color;
use crate::rendering::solid_rectangle_render::SolidRectangleRender;

pub type ButtonAction = Box<dyn Fn(&mut CommandBuffer)>;
pub type CommandAction<T> = Box<dyn Fn(&mut T, &Command)>;

pub trait GenericButton {
    fn process_gui_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer);
    fn process_command(&mut self, command: &Command);
}

pub struct TextureButton {
    texture: Texture,
    position: Position,
    left_click_action: Option<ButtonAction>,
    right_click_action: Option<ButtonAction>,
    command_action: Option<CommandAction<Self>>
}

impl TextureButton {
    pub fn new(image: &image::RgbaImage,
               position: Position,
               left_click_action: Option<ButtonAction>,
               right_click_action: Option<ButtonAction>,
               command_action: Option<CommandAction<Self>>) -> TextureButton {
        TextureButton {
            texture: Texture::from_image(image),
            position,
            left_click_action,
            right_click_action,
            command_action
        }
    }

    pub fn render(&self, shader: &Shader, texture_render: &TextureRender, transform: &Matrix4<f32>) {
        texture_render.render(
            &shader,
            &transform,
            &self.texture,
            self.position
        );
    }
}

impl GenericButton for TextureButton {
    fn process_gui_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                if let Some(left_click_action) = self.left_click_action.as_ref() {
                    let bounding_rectangle = Rectangle::new(
                        self.position.x,
                        self.position.y,
                        self.texture.width() as f32,
                        self.texture.height() as f32
                    );

                    let mouse_position = window.get_cursor_pos();
                    if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (left_click_action)(command_buffer);
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Release, _) => {
                if let Some(right_click_action) = self.right_click_action.as_ref() {
                    let bounding_rectangle = Rectangle::new(
                        self.position.x,
                        self.position.y,
                        self.texture.width() as f32,
                        self.texture.height() as f32
                    );

                    let mouse_position = window.get_cursor_pos();
                    if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (right_click_action)(command_buffer);
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
}

pub struct SolidColorButton {
    color: Color,
    rectangle: Rectangle,
    left_click_action: Option<ButtonAction>,
    right_click_action: Option<ButtonAction>,
    command_action: Option<CommandAction<Self>>
}

impl SolidColorButton {
    pub fn new(color: Color,
               rectangle: Rectangle,
               left_click_action: Option<ButtonAction>,
               right_click_action: Option<ButtonAction>,
               command_action: Option<CommandAction<Self>>) -> SolidColorButton {
        SolidColorButton {
            color,
            rectangle,
            left_click_action,
            right_click_action,
            command_action
        }
    }

    pub fn render(&self, shader: &Shader, solid_rectangle_render: &SolidRectangleRender, transform: &Matrix4<f32>) {
        solid_rectangle_render.render(
            shader,
            &transform,
            self.rectangle.position,
            self.rectangle.size,
            RenderingColor::new(self.color[0], self.color[1], self.color[2])
        );
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl GenericButton for SolidColorButton {
    fn process_gui_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                if let Some(left_click_action) = self.left_click_action.as_ref() {
                    let mouse_position = window.get_cursor_pos();
                    if self.rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (left_click_action)(command_buffer);
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Release, _) => {
                if let Some(right_click_action) = self.right_click_action.as_ref() {
                    let mouse_position = window.get_cursor_pos();
                    if self.rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (right_click_action)(command_buffer);
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
}

pub struct TextButton {
    font: FontRef,
    text: String,
    position: Position,
    left_click_action: Option<ButtonAction>,
    right_click_action: Option<ButtonAction>,
    command_action: Option<CommandAction<Self>>
}

impl TextButton {
    pub fn new(font: FontRef,
               text: String,
               position: Position,
               left_click_action: Option<ButtonAction>,
               right_click_action: Option<ButtonAction>,
               command_action: Option<CommandAction<Self>>) -> TextButton {
        TextButton {
            font,
            text,
            position,
            left_click_action,
            right_click_action,
            command_action
        }
    }

    pub fn render(&self,
                  shader: &Shader,
                  text_render: &TextRender,
                  transform: &Matrix4<f32>) {
        text_render.draw_line(
            shader,
            transform,
            self.font.borrow_mut().deref_mut(),
            self.text.chars().map(|c| (c, RenderingColor::new(0, 0, 0))),
            self.position,
            TextAlignment::Top
        );
    }
}

impl GenericButton for TextButton {
    fn process_gui_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
        let mut font = self.font.borrow_mut();
        let width = font.line_width(&self.text);
        let height = font.line_height();

        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                if let Some(left_click_action) = self.left_click_action.as_ref() {
                    let bounding_rectangle = Rectangle::new(
                        self.position.x,
                        self.position.y,
                        width,
                        height
                    );

                    let mouse_position = window.get_cursor_pos();
                    if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (left_click_action)(command_buffer);
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Release, _) => {
                if let Some(right_click_action) = self.right_click_action.as_ref() {
                    let bounding_rectangle = Rectangle::new(
                        self.position.x,
                        self.position.y,
                        width,
                        height
                    );

                    let mouse_position = window.get_cursor_pos();
                    if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                        (right_click_action)(command_buffer);
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
}