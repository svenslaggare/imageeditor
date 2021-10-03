use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation, ImageOperationMarker, ImageSource};

pub struct ColorPickerTool {

}

impl ColorPickerTool {
    pub fn new() -> ColorPickerTool {
        ColorPickerTool {

        }
    }

    fn select_color(&self,
                    window: &mut glfw::Window,
                    transform: &Matrix3<f32>,
                    image: &editor::Image) -> Option<editor::Color> {
        let position = get_transformed_mouse_position(window, transform);
        let position_x = position.x.round() as i32;
        let position_y = position.y.round() as i32;

        if position_x >= 0 && position_x < image.width() as i32 && position_y >= 0 && position_y < image.height() as i32 {
            Some(image.get_pixel(position_x as u32, position_y as u32))
        } else {
            None
        }
    }
}

impl Tool for ColorPickerTool {
    fn process_gui_event(&mut self,
                         window: &mut glfw::Window,
                         event: &WindowEvent,
                         transform: &Matrix3<f32>,
                         command_buffer: &mut CommandBuffer,
                         image: &editor::Image) -> Option<ImageOperation> {
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                if let Some(color) = self.select_color(window, transform, image) {
                    command_buffer.push(Command::SetColor(color))
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Press, _) => {
                if let Some(color) = self.select_color(window, transform, image) {
                    command_buffer.push(Command::SetAlternativeColor(color))
                }
            }
            _ => {}
        }

        None
    }

    fn preview(&mut self, _image: &editor::Image, _preview_image: &mut editor::Image) -> bool {
        false
    }
}
