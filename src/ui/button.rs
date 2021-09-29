use std::ops::DerefMut;

use glfw::{WindowEvent, Action};
use cgmath::Matrix4;

use crate::rendering::texture::Texture;
use crate::rendering::prelude::{Position, Rectangle, Color};
use crate::rendering::texture_render::TextureRender;
use crate::rendering::shader::Shader;
use crate::command_buffer::CommandBuffer;
use crate::rendering::text_render::{TextRender, TextAlignment};
use crate::rendering::font::{Font, FontRef};

pub type ButtonAction = Box<dyn Fn(&mut CommandBuffer)>;

pub struct Button {
    texture: Texture,
    position: Position,
    left_click_action: Option<ButtonAction>,
    right_click_action: Option<ButtonAction>
}

impl Button {
    pub fn new(image: &image::RgbaImage,
               position: Position,
               left_click_action: Option<ButtonAction>,
               right_click_action: Option<ButtonAction>) -> Button {
        Button {
            texture: Texture::from_image(image),
            position,
            left_click_action,
            right_click_action
        }
    }

    pub fn process_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
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

    pub fn render(&self, shader: &Shader, texture_render: &TextureRender, transform: &Matrix4<f32>) {
        texture_render.render(
            &shader,
            &transform,
            &self.texture,
            self.position
        );
    }
}

pub struct TextButton {
    font: FontRef,
    text: String,
    position: Position,
    left_click_action: Option<ButtonAction>,
    right_click_action: Option<ButtonAction>
}

impl TextButton {
    pub fn new(font: FontRef,
               text: String,
               position: Position,
               left_click_action: Option<ButtonAction>,
               right_click_action: Option<ButtonAction>) -> TextButton {
        TextButton {
            font,
            text,
            position,
            left_click_action,
            right_click_action
        }
    }

    pub fn process_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
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

    pub fn render(&self,
                  shader: &Shader,
                  text_render: &TextRender,
                  transform: &Matrix4<f32>) {
        text_render.draw_line(
            shader,
            transform,
            self.font.borrow_mut().deref_mut(),
            self.text.chars().map(|c| (c, Color::new(0, 0, 0))),
            self.position,
            TextAlignment::Top
        );
    }
}