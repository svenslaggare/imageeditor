use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation, ImageOperationMarker};

pub struct EraserDrawTool {
    is_drawing: bool,
    prev_mouse_position: Option<Position>,
    side_half_width: i32
}

impl EraserDrawTool {
    pub fn new() -> EraserDrawTool {
        EraserDrawTool {
            is_drawing: false,
            prev_mouse_position: None,
            side_half_width: 3
        }
    }
}

impl Tool for EraserDrawTool {
    fn process_gui_event(&mut self,
                         window: &mut glfw::Window,
                         event: &WindowEvent,
                         transform: &Matrix3<f32>,
                         _command_buffer: &mut CommandBuffer,
                         _image: &editor::Image) -> Option<ImageOperation> {
        let create_begin_draw = |this: &Self, mouse_position: Position| {
            Some(ImageOperation::Sequential(vec![
                ImageOperation::Marker(ImageOperationMarker::BeginDraw),
                ImageOperation::Line {
                    start_x: mouse_position.x as i32,
                    start_y: mouse_position.y as i32,
                    end_x: mouse_position.x as i32,
                    end_y: mouse_position.y as i32,
                    color: image::Rgba([0, 0, 0, 0]),
                    side_half_width: this.side_half_width
                }
            ]))
        };

        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.is_drawing = true;
                op = create_begin_draw(self, get_transformed_mouse_position(window, transform));
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1 | glfw::MouseButton::Button2, Action::Release, _) => {
                self.is_drawing = false;
                self.prev_mouse_position = None;
                op = Some(ImageOperation::Marker(ImageOperationMarker::EndDraw));
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                if self.is_drawing {
                    let mouse_position = transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                    if let Some(prev_mouse_position) = self.prev_mouse_position {
                        op = Some(ImageOperation::Line {
                            start_x: prev_mouse_position.x as i32,
                            start_y: prev_mouse_position.y as i32,
                            end_x: mouse_position.x as i32,
                            end_y: mouse_position.y as i32,
                            color: image::Rgba([0, 0, 0, 0]),
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
