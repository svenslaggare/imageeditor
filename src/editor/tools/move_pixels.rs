use glfw::{WindowEvent, Action, Key, Modifiers, Window};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::Command;
use crate::editor::tools::{Tool, get_valid_rectangle};
use crate::editor::image_operation::{ImageOperation, ImageSource};
use crate::editor::image_operation_helpers::sub_image;

pub struct MovePixelsTool {
    current_mouse_position: Option<Position>,
    start_position: Option<Position>,
    end_position: Option<Position>,
    is_selecting: bool,
    is_moving: bool,
    move_position: Option<Position>,
    moved_pixels_image: Option<image::RgbaImage>
}

impl MovePixelsTool {
    pub fn new() -> MovePixelsTool {
        MovePixelsTool {
            current_mouse_position: None,
            start_position: None,
            end_position: None,
            is_selecting: false,
            is_moving: false,
            move_position: None,
            moved_pixels_image: None
        }
    }

    fn create_selection(&self, start_position: &Position, end_position: &Position) -> ImageOperation {
        let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);

        ImageOperation::Sequential(vec![
            ImageOperation::FillRectangle {
                start_x,
                start_y,
                end_x,
                end_y,
                color: image::Rgba([0, 148, 255, 64]),
                blend: true
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

    fn create_move(&self, preview: bool) -> Option<ImageOperation> {
        match (self.start_position.as_ref(),
               self.end_position.as_ref(),
               self.move_position.as_ref(),
               self.moved_pixels_image.as_ref()) {
            (Some(start_position), Some(end_position), Some(move_position), Some(moved_pixels_image)) => {
                let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);

                return Some(
                    ImageOperation::Sequential(
                        vec![
                            ImageOperation::FillRectangle {
                                start_x,
                                start_y,
                                end_x,
                                end_y,
                                color: if preview {image::Rgba([255, 255, 255, 255])} else {image::Rgba([0, 0, 0, 0])},
                                blend: preview
                            },
                            ImageOperation::SetImage {
                                start_x: move_position.x as i32,
                                start_y: move_position.y as i32,
                                image: moved_pixels_image.clone(),
                                blend: true
                            }
                        ]
                    )
                );
            }
            _ => {}
        }

        None
    }
}

impl Tool for MovePixelsTool {
    fn handle_command(&mut self, _command: &Command) {

    }

    fn process_event(&mut self,
                     _window: &mut Window,
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
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Press, _) => {
                if self.moved_pixels_image.is_none() {
                    if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                        let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);
                        self.moved_pixels_image = Some(sub_image(image, start_x, start_y, end_x, end_y));
                        self.is_moving = true;
                    }
                } else {
                    self.is_moving = true;
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Release, _) => {
                self.is_moving = false;
            }
            glfw::WindowEvent::Key(Key::Enter, _, Action::Release, _) => {
                op = self.create_move(false);

                self.start_position = None;
                self.end_position = None;

                self.is_moving = false;
                self.moved_pixels_image = None;
                self.move_position = None;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                let mouse_position = transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                if self.is_selecting {
                    self.end_position = Some(mouse_position);
                }

                if self.is_moving {
                    self.move_position = Some(mouse_position);
                }

                self.current_mouse_position = Some(mouse_position);
            }
            glfw::WindowEvent::Key(Key::A, _, Action::Press, Modifiers::Control) => {
                self.start_position = Some(Position::new(0.0, 0.0));
                self.end_position = Some(Position::new(image.width() as f32, image.height() as f32));
            }
            _ => {}
        }

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        let mut update_op = preview_image.update_operation();
        if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
            if let Some(move_op) = self.create_move(true) {
                let move_position = self.move_position.as_ref().unwrap();
                let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);
                move_op.apply(&mut update_op, false);
                self.create_selection(
                    move_position,
                    &Position::new(
                        move_position.x + (end_x as f32 - start_x as f32),
                        move_position.y + (end_y as f32 - start_y as f32)
                    )
                ).apply(&mut update_op, false);
            } else {
                self.create_selection(start_position, end_position).apply(&mut update_op, false);
            }
        }

        return true;
    }
}