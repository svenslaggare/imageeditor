use cgmath::Matrix4;

use crate::ui::TextureButton;
use crate::ui::button::{TextButton, SolidColorButton, GenericButton};
use crate::rendering::shader::Shader;
use crate::rendering::texture_render::TextureRender;
use crate::command_buffer::{CommandBuffer, Command};
use crate::rendering::text_render::TextRender;
use crate::rendering::solid_rectangle_render::SolidRectangleRender;

pub struct Manager {
    texture_buttons: Vec<TextureButton>,
    solid_color_buttons: Vec<SolidColorButton>,
    text_buttons: Vec<TextButton>
}

impl Manager {
    pub fn new(texture_buttons: Vec<TextureButton>,
               solid_color_buttons: Vec<SolidColorButton>,
               text_buttons: Vec<TextButton>) -> Manager {
        Manager {
            texture_buttons,
            solid_color_buttons,
            text_buttons
        }
    }

    pub fn process_gui_event(&mut self, window: &mut glfw::Window, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
        for button in &mut self.texture_buttons {
            button.process_gui_event(window, event, command_buffer);
        }

        for button in &mut self.solid_color_buttons {
            button.process_gui_event(window, event, command_buffer);
        }

        for button in &mut self.text_buttons {
            button.process_gui_event(window, event, command_buffer);
        }
    }

    pub fn process_command(&mut self, command: &Command) {
        for button in &mut self.texture_buttons {
            button.process_command(command);
        }

        for button in &mut self.solid_color_buttons {
            button.process_command(command);
        }

        for button in &mut self.text_buttons {
            button.process_command(command);
        }
    }

    pub fn render(&self,
                  texture_shader: &Shader,
                  texture_render: &TextureRender,
                  solid_rectangle_shader: &Shader,
                  solid_rectangle_render: &SolidRectangleRender,
                  text_shader: &Shader,
                  text_render: &TextRender,
                  transform: &Matrix4<f32>) {
        for button in &self.texture_buttons {
            button.render(
                texture_shader,
                texture_render,
                transform
            );
        }

        for button in &self.solid_color_buttons {
            button.render(
                solid_rectangle_shader,
                solid_rectangle_render,
                transform
            );
        }

        for button in &self.text_buttons {
            button.render(
                text_shader,
                text_render,
                transform
            );
        }
    }
}