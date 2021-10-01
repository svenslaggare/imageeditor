use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::Command;
use crate::editor::tools::{Tool, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation, ImageOperationMarker};

pub struct PencilDrawTool {
    is_drawing: Option<editor::Color>,
    prev_mouse_position: Option<Position>,
    color: editor::Color,
    alternative_color: editor::Color,
    side_half_width: i32
}

impl PencilDrawTool {
    pub fn new() -> PencilDrawTool {
        PencilDrawTool {
            is_drawing: None,
            prev_mouse_position: None,
            color: image::Rgba([0, 0, 0, 255]),
            alternative_color: image::Rgba([0, 0, 0, 255]),
            side_half_width: 1
        }
    }
}

impl Tool for PencilDrawTool {
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
                     _image: &editor::Image) -> Option<ImageOperation> {
        let create_begin_draw = |this: &Self, mouse_position: Position, color: editor::Color| {
            Some(ImageOperation::Sequential(vec![
                ImageOperation::Marker(ImageOperationMarker::BeginDraw),
                ImageOperation::DrawBlock {
                    x: mouse_position.x as i32,
                    y: mouse_position.y as i32,
                    color,
                    side_half_width: this.side_half_width
                }
            ]))
        };

        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.is_drawing = Some(self.color);
                op = create_begin_draw(self, get_transformed_mouse_position(window, transform), self.color);
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Press, _) => {
                self.is_drawing = Some(self.alternative_color);
                op = create_begin_draw(self, get_transformed_mouse_position(window, transform), self.alternative_color);
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1 | glfw::MouseButton::Button2, Action::Release, _) => {
                self.is_drawing = None;
                self.prev_mouse_position = None;
                op = Some(ImageOperation::Marker(ImageOperationMarker::EndDraw));
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                if let Some(color) = self.is_drawing {
                    let mouse_position = transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                    if let Some(prev_mouse_position) = self.prev_mouse_position {
                        op = Some(ImageOperation::DrawLine {
                            start_x: prev_mouse_position.x as i32,
                            start_y: prev_mouse_position.y as i32,
                            end_x: mouse_position.x as i32,
                            end_y: mouse_position.y as i32,
                            color,
                            side_half_width: self.side_half_width
                        });
                    }

                    self.prev_mouse_position = Some(mouse_position);
                }
            }
            _ => {}
        }

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, _preview_image: &mut editor::Image) -> bool {
        false
    }
}