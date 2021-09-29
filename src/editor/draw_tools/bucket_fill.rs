use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::Command;
use crate::editor::draw_tools::{DrawTool, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation};

pub struct BucketFillDrawTool {
    color: editor::Color,
}

impl BucketFillDrawTool {
    pub fn new() -> BucketFillDrawTool {
        BucketFillDrawTool {
            color: image::Rgba([255, 0, 0, 255]),
        }
    }

    fn create_op(&self, position: &Position) -> ImageOperation {
        ImageOperation::BucketFill {
            start_x: position.x as i32,
            start_y: position.y as i32,
            fill_color: self.color,
        }
    }
}

impl DrawTool for BucketFillDrawTool {
    fn handle_command(&mut self, command: &Command) {
        match command {
            Command::SetColor(color) => {
                self.color = *color;
            }
            _ => {}
        }
    }

    fn process_event(&mut self,
                     window: &mut glfw::Window,
                     event: &WindowEvent,
                     transform: &Matrix3<f32>,
                     _image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;

        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                let mouse_position = get_transformed_mouse_position(window, transform);
                op = Some(self.create_op(&mouse_position));
            }
            _ => {}
        }

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, _preview_image: &mut editor::Image) -> bool {
        return false;
    }
}
