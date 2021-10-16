use cgmath::Matrix4;

use crate::ui::TextureButton;
use crate::ui::button::{TextButton, SolidColorButton, GenericButton};
use crate::command_buffer::{CommandBuffer, Command};
use crate::program::Renders;
use crate::editor::tools::EditorWindow;

pub struct Manager {
    texture_buttons: Vec<TextureButton<CommandBuffer>>,
    solid_color_buttons: Vec<SolidColorButton<CommandBuffer>>,
    text_buttons: Vec<TextButton<CommandBuffer>>
}

impl Manager {
    pub fn new(texture_buttons: Vec<TextureButton<CommandBuffer>>,
               solid_color_buttons: Vec<SolidColorButton<CommandBuffer>>,
               text_buttons: Vec<TextButton<CommandBuffer>>) -> Manager {
        Manager {
            texture_buttons,
            solid_color_buttons,
            text_buttons
        }
    }

    pub fn process_gui_event(&mut self, window: &mut dyn EditorWindow, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
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

    pub fn render(&self, renders: &Renders, transform: &Matrix4<f32>) {
        for button in &self.texture_buttons {
            button.render(renders, transform);
        }

        for button in &self.solid_color_buttons {
            button.render(renders, transform);
        }

        for button in &self.text_buttons {
            button.render(renders, transform);
        }
    }
}