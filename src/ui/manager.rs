use cgmath::Matrix4;

use crate::ui::Button;
use crate::rendering::shader::Shader;
use crate::rendering::texture_render::TextureRender;
use crate::command_buffer::CommandBuffer;

pub struct Manager {
    buttons: Vec<Button>
}

impl Manager {
    pub fn new(buttons: Vec<Button>) -> Manager {
        Manager {
            buttons
        }
    }

    pub fn process_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
        for button in &mut self.buttons {
            button.process_event(window, event, command_buffer);
        }
    }

    pub fn render(&self, shader: &Shader, texture_render: &TextureRender, transform: &Matrix4<f32>) {
        for button in &self.buttons {
            button.render(
                &shader,
                &texture_render,
                &transform
            );
        }
    }
}