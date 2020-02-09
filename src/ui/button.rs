use glfw::{WindowEvent, Action};
use cgmath::Matrix4;

use crate::rendering::texture::Texture;
use crate::rendering::prelude::{Position, Rectangle};
use crate::rendering::texture_render::TextureRender;
use crate::rendering::shader::Shader;
use crate::command_buffer::CommandBuffer;

pub type ButtonAction = Box<dyn Fn(&mut CommandBuffer)>;

pub struct Button {
    texture: Texture,
    position: Position,
    action: ButtonAction
}

impl Button {
    pub fn new(image: &image::RgbaImage, position: Position, action: ButtonAction) -> Button {
        Button {
            texture: Texture::from_image(image),
            position,
            action
        }
    }

    pub fn process_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                let bounding_rectangle = Rectangle::new(
                    self.position.x,
                    self.position.y,
                    self.texture.width() as f32,
                    self.texture.height() as f32
                );

                let mouse_position = window.get_cursor_pos();
                if bounding_rectangle.contains(&Position::new(mouse_position.0 as f32, mouse_position.1 as f32)) {
                    (self.action)(command_buffer);
                }
            }
            _ => {}
        }
    }

    pub fn render(&self,
                  shader: &Shader,
                  texture_render: &TextureRender,
                  transform: &Matrix4<f32>) {
        texture_render.render(
            &shader,
            &transform,
            &self.texture,
            self.position
        );
    }
}