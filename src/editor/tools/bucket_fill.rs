use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation};

pub struct BucketFillDrawTool {
    color: editor::Color,
    alternative_color: editor::Color
}

impl BucketFillDrawTool {
    pub fn new() -> BucketFillDrawTool {
        BucketFillDrawTool {
            color: image::Rgba([0, 0, 0, 255]),
            alternative_color: image::Rgba([0, 0, 0, 255]),
        }
    }
}

impl Tool for BucketFillDrawTool {
    fn handle_command(&mut self, command: &Command) {
        match command {
            Command::SetColor(color) => {
                self.color = *color;
            }
            Command::SetAlternativeColor(color) => {
                self.alternative_color = *color;
            }
            _ => {}
        }
    }

    fn process_event(&mut self,
                     window: &mut glfw::Window,
                     event: &WindowEvent,
                     transform: &Matrix3<f32>,
                     _command_buffer: &mut CommandBuffer,
                     _image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;

        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                let mouse_position = get_transformed_mouse_position(window, transform);
                op = Some(
                    ImageOperation::BucketFill {
                        start_x: mouse_position.x as i32,
                        start_y: mouse_position.y as i32,
                        fill_color: self.color,
                    }
                );
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Press, _) => {
                let mouse_position = get_transformed_mouse_position(window, transform);
                op = Some(
                    ImageOperation::BucketFill {
                        start_x: mouse_position.x as i32,
                        start_y: mouse_position.y as i32,
                        fill_color: self.alternative_color,
                    }
                );
            }
            _ => {}
        }

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, _preview_image: &mut editor::Image) -> bool {
        return false;
    }
}
