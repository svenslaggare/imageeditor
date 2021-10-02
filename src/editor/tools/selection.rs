use glfw::{WindowEvent, Action, Key, Modifiers, Window};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer, Selection};
use crate::editor::tools::{Tool, get_valid_rectangle};
use crate::editor::image_operation::{ImageOperation, ImageSource};
use crate::editor::image_operation_helpers::sub_image;

pub struct SelectionTool {
    current_mouse_position: Option<Position>,
    start_position: Option<Position>,
    end_position: Option<Position>,
    is_selecting: bool,
    copied_image: Option<image::RgbaImage>
}

impl SelectionTool {
    pub fn new() -> SelectionTool {
        SelectionTool {
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
                blend: false
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

    fn send_set_selection(&self, command_buffer: &mut CommandBuffer) {
        match (self.start_position, self.end_position) {
            (Some(start_position), Some(end_position)) => {
                let (start_x, start_y, end_x, end_y) = get_valid_rectangle(&start_position, &end_position);
                command_buffer.push(Command::SetSelection(Some(Selection {
                    start_x,
                    start_y,
                    end_x,
                    end_y
                })));
            }
            _ => command_buffer.push(Command::SetSelection(None))
        }
    }
}

impl Tool for SelectionTool {
    fn handle_command(&mut self, command: &Command) {
        match command {
            Command::SetSelection(Some(selection)) if !self.is_selecting => {
                self.start_position = Some(selection.start_position());
                self.end_position = Some(selection.end_position());
            }
            Command::SetSelection(None) if !self.is_selecting => {
                self.start_position = None;
                self.end_position = None;
            }
            _ => {}
        }
    }

    fn process_event(&mut self,
                     _window: &mut Window,
                     event: &WindowEvent,
                     transform: &Matrix3<f32>,
                     command_buffer: &mut CommandBuffer,
                     image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                if let Some(current_mouse_position) = self.current_mouse_position.as_ref() {
                    if current_mouse_position.x >= 0.0 && current_mouse_position.y >= 0.0 {
                        self.start_position = Some(current_mouse_position.clone());
                        self.end_position = None;
                        self.is_selecting = true;
                        self.send_set_selection(command_buffer);
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.is_selecting = false;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                let mouse_position = transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                if self.is_selecting {
                    self.end_position = Some(mouse_position);
                    self.send_set_selection(command_buffer);
                }

                self.current_mouse_position = Some(mouse_position);
            }
            glfw::WindowEvent::Key(Key::A, _, Action::Press, Modifiers::Control) => {
                self.start_position = Some(Position::new(0.0, 0.0));
                self.end_position = Some(Position::new(image.width() as f32, image.height() as f32));
                self.send_set_selection(command_buffer);
            }
            glfw::WindowEvent::Key(Key::Delete, _, Action::Press, _) => {
                if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                    let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);
                    op = Some(
                        ImageOperation::FillRectangle {
                            start_x,
                            start_y,
                            end_x,
                            end_y,
                            color: image::Rgba([0, 0, 0, 0]),
                            blend: false
                        }
                    );

                    self.start_position = None;
                    self.end_position = None;
                    self.send_set_selection(command_buffer);
                }
            }
            glfw::WindowEvent::Key(Key::C, _, Action::Press, Modifiers::Control) => {
                if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                    let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);
                    self.copied_image = Some(sub_image(image, start_x, start_y, end_x, end_y));
                    self.start_position = None;
                    self.end_position = None;
                    self.send_set_selection(command_buffer);
                }
            }
            glfw::WindowEvent::Key(Key::V, _, Action::Press, Modifiers::Control) => {
                if let Some(mouse_position) = self.current_mouse_position.as_ref() {
                    let start_x = mouse_position.x as i32;
                    let start_y = mouse_position.y as i32;
                    if let Some(copied_image) = self.copied_image.as_ref() {
                        op = Some(ImageOperation::SetImage { start_x, start_y, image: copied_image.clone(), blend: false });
                    }
                }
            }
            glfw::WindowEvent::Key(Key::X, _, Action::Press, Modifiers::Control) => {
                if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                    let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);

                    op = Some(
                        ImageOperation::FillRectangle {
                            start_x,
                            start_y,
                            end_x,
                            end_y,
                            color: image::Rgba([0, 0, 0, 0]),
                            blend: false
                        }
                    );

                    self.copied_image = Some(sub_image(image, start_x, start_y, end_x, end_y));

                    self.start_position = None;
                    self.end_position = None;
                    self.send_set_selection(command_buffer);
                }
            }
            _ => {}
        }

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        let mut update_op = preview_image.update_operation();
        if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
            self.create_preview(start_position, end_position).apply(&mut update_op, false);
        }

        return true;
    }
}