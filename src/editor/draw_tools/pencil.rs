use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::Command;
use crate::editor::draw_tools::{DrawTool, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation, ImageOperationMarker};

pub struct PencilDrawTool {
    is_drawing: bool,
    prev_mouse_position: Option<Position>,
    color: editor::Color,
    side_half_width: i32
}

impl PencilDrawTool {
    pub fn new() -> PencilDrawTool {
        PencilDrawTool {
            is_drawing: false,
            prev_mouse_position: None,
            color: image::Rgba([255, 0, 0, 255]),
            side_half_width: 0
        }
    }
}

impl DrawTool for PencilDrawTool {
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
                     image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.is_drawing = true;

                let mouse_position = get_transformed_mouse_position(window, transform);
                op = Some(ImageOperation::Sequential(vec![
                    ImageOperation::Marker(ImageOperationMarker::BeginDraw),
                    ImageOperation::DrawBlock {
                        x: mouse_position.x as i32,
                        y: mouse_position.y as i32,
                        color: self.color,
                        side_half_width: self.side_half_width
                    }
                ]));
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.is_drawing = false;
                self.prev_mouse_position = None;
                op = Some(ImageOperation::Marker(ImageOperationMarker::EndDraw));
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                let mouse_position = transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                if self.is_drawing {
                    if let Some(prev_mouse_position) = self.prev_mouse_position {
                        op = Some(ImageOperation::DrawLine {
                            start_x: prev_mouse_position.x as i32,
                            start_y: prev_mouse_position.y as i32,
                            end_x: mouse_position.x as i32,
                            end_y: mouse_position.y as i32,
                            color: self.color,
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

    fn preview(&mut self, image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        false
    }
}
