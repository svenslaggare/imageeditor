use cgmath::Matrix4;

use crate::ui::TextureButton;
use crate::ui::button::{TextButton, SolidColorButton, GenericButton};
use crate::command_buffer::{CommandBuffer, Command};
use crate::program::Renders;
use crate::editor::tools::EditorWindow;

pub type BoxGenericButton = Box<dyn GenericButton<CommandBuffer>>;

pub struct Manager {
    buttons: Vec<BoxGenericButton>
}

impl Manager {
    pub fn new(buttons: Vec<BoxGenericButton>) -> Manager {
        Manager {
            buttons
        }
    }

    pub fn process_gui_event(&mut self, window: &mut dyn EditorWindow, event: &glfw::WindowEvent, command_buffer: &mut CommandBuffer) {
        for button in &mut self.buttons {
            button.process_gui_event(window, event, command_buffer);
        }
    }

    pub fn process_command(&mut self, command: &Command) {
        for button in &mut self.buttons {
            button.process_command(command);
        }
    }

    pub fn render(&self, renders: &Renders, transform: &Matrix4<f32>) {
        for button in &self.buttons {
            button.render(renders, transform);
        }
    }
}