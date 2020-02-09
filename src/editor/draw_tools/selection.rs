use glfw::{WindowEvent, Action, Key, Modifiers, Window};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::Command;
use crate::editor::draw_tools::{DrawTool, get_transformed_mouse_position, get_valid_rectangle};
use crate::editor::image_operation::{ImageOperation, ImageOperationMarker};
use crate::editor::image_operation_helpers::sub_image;
use crate::editor::image::ImageUpdateOperation;

pub struct SelectionDrawTool {
    current_mouse_position: Option<Position>,
    start_position: Option<Position>,
    end_position: Option<Position>,
    is_selecting: bool,
    copied_image: Option<image::RgbaImage>
}

impl SelectionDrawTool {
    pub fn new() -> SelectionDrawTool {
        SelectionDrawTool {
            current_mouse_position: None,
            start_position: None,
            end_position: None,
            is_selecting: false,
            copied_image: None
        }
    }

    fn create_preview(&self, start_position: &Position, end_position: &Position) -> ImageOperation {
        let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);

        ImageOperation::Sequential(vec![
            ImageOperation::FillRectangle {
                start_x,
                start_y,
                end_x,
                end_y,
                color: image::Rgba([0, 148, 255, 64]),
            },
            ImageOperation::DrawRectangle {
                start_x,
                start_y,
                end_x,
                end_y,
                color: image::Rgba([0, 0, 0, 255])
            }
        ])
    }
}

impl DrawTool for SelectionDrawTool {
    fn handle_command(&mut self, command: &Command) {

    }

    fn process_event(&mut self,
                     window: &mut Window,
                     event: &WindowEvent,
                     transform: &Matrix3<f32>,
                     image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                if self.current_mouse_position.is_some() {
                    self.start_position = self.current_mouse_position.clone();
                }

                self.end_position = None;
                self.is_selecting = true;
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.is_selecting = false;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                let mouse_position = transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                if self.is_selecting {
                    self.end_position = Some(mouse_position);
                }

                self.current_mouse_position = Some(mouse_position);
            }
            glfw::WindowEvent::Key(Key::A, _, Action::Press, Modifiers::Control) => {
                self.start_position = Some(Position::new(0.0, 0.0));
                self.end_position = Some(Position::new(image.width() as f32, image.height() as f32));
            }
            glfw::WindowEvent::Key(Key::Delete, _, Action::Press, _) => {
                if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                    let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);
                    op = Some(ImageOperation::FillRectangle {
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        color: image::Rgba([0, 0, 0, 0]),
                    });

                    self.start_position = None;
                    self.end_position = None;
                }
            }
            glfw::WindowEvent::Key(Key::C, _, Action::Press, Modifiers::Control) => {
                if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                    let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);
                    self.copied_image = Some(sub_image(image, start_x, start_y, end_x, end_y));
                    self.start_position = None;
                    self.end_position = None;
                }
            }
            glfw::WindowEvent::Key(Key::V, _, Action::Press, Modifiers::Control) => {
                if let Some(mouse_position) = self.current_mouse_position.as_ref() {
                    let start_x = mouse_position.x as i32;
                    let start_y = mouse_position.y as i32;
                    if let Some(copied_image) = self.copied_image.as_ref() {
                        op = Some(ImageOperation::SetImage { start_x, start_y, image: copied_image.clone() });
                    }
                }
            }
            glfw::WindowEvent::Key(Key::X, _, Action::Press, Modifiers::Control) => {
                if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                    let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);

                    op = Some(ImageOperation::FillRectangle {
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        color: image::Rgba([0, 0, 0, 0]),
                    });

                    self.copied_image = Some(sub_image(image, start_x, start_y, end_x, end_y));

                    self.start_position = None;
                    self.end_position = None;
                }
            }
            _ => {}
        }

        return op;
    }

    fn preview(&mut self, image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        let mut update_op = preview_image.update_operation();
        if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
            self.create_preview(start_position, end_position).apply(&mut update_op, false);
        }

        return true;
    }
}