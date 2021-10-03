use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation};

pub struct CircleDrawTool {
    start_position: Option<Position>,
    end_position: Option<Position>,
    border_color: editor::Color,
    fill_color: editor::Color,
}

impl CircleDrawTool {
    pub fn new() -> CircleDrawTool {
        CircleDrawTool {
            start_position: None,
            end_position: None,
            border_color: image::Rgba([0, 0, 0, 255]),
            fill_color: image::Rgba([255, 0, 0, 255])
        }
    }

    fn create_op(&self, start_position: &Position, end_position: &Position) -> ImageOperation {
        let start_x = start_position.x as i32;
        let start_y = start_position.y as i32;
        let end_x = end_position.x as i32;
        let end_y = end_position.y as i32;
        let radius = (((end_x - start_x).pow(2) + (end_y - start_y).pow(2)) as f64).sqrt() as i32;

        ImageOperation::Sequential(vec![
            ImageOperation::FillCircle {
                center_x: start_x,
                center_y: start_y,
                radius,
                color: self.fill_color,
            },
            ImageOperation::Circle {
                center_x: start_x,
                center_y: start_y,
                radius,
                border_side_half_width: 1,
                color: self.border_color,
            },
        ])
    }
}

impl Tool for CircleDrawTool {
    fn handle_command(&mut self, command: &Command) {
        match command {
            Command::SetColor(color) => {
                self.fill_color = *color;
            }
            Command::SetAlternativeColor(color) => {
                self.border_color = *color;
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
                self.start_position = Some(get_transformed_mouse_position(window, transform));
                self.end_position = None;
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                    op = Some(self.create_op(start_position, end_position));
                }

                self.start_position = None;
                self.end_position = None;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                let mouse_position = transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));
                self.end_position = Some(mouse_position);
            }
            _ => {}
        }

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        let mut update_op = preview_image.update_operation();
        if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
            self.create_op(start_position, end_position).apply(&mut update_op, false);
        }

        return true;
    }
}
