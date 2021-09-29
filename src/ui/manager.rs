use cgmath::Matrix4;

use crate::ui::Button;
use crate::ui::button::TextButton;
use crate::rendering::shader::Shader;
use crate::rendering::texture_render::TextureRender;
use crate::command_buffer::CommandBuffer;
use crate::rendering::text_render::TextRender;
use crate::rendering::font::Font;

pub struct Manager {
    buttons: Vec<Button>,
    text_buttons: Vec<TextButton>
}

impl Manager {
    pub fn new(buttons: Vec<Button>, text_buttons: Vec<TextButton>) -> Manager {
        Manager {
            buttons,
            text_buttons
        }
    }

    pub fn process_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
        for button in &mut self.buttons {
            button.process_event(window, event, command_buffer);
        }

        for button in &mut self.text_buttons {
            button.process_event(window, event, command_buffer);
        }
    }

    pub fn render(&self,
                  texture_shader: &Shader,
                  texture_render: &TextureRender,
                  text_shader: &Shader,
                  text_render: &TextRender,
                  transform: &Matrix4<f32>) {
        for button in &self.buttons {
            button.render(
                &texture_shader,
                &texture_render,
                &transform
            );
        }

        for button in &self.text_buttons {
            button.render(
                &text_shader,
                &text_render,
                &transform
            );
        }
    }
}