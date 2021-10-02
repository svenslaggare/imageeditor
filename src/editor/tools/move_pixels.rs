use glfw::{WindowEvent, Action, Key, Modifiers, Window};
use cgmath::{Matrix3, Transform};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer, Selection};
use crate::editor::tools::{Tool, get_valid_rectangle};
use crate::editor::image_operation::{ImageOperation, ImageSource};
use crate::editor::image_operation_helpers::sub_image;

pub struct MovePixelsTool {
    current_mouse_position: Option<Position>,
    selection: Option<Selection>,
    is_moving: bool,
    move_position: Option<Position>,
    moved_pixels_image: Option<image::RgbaImage>
}

impl MovePixelsTool {
    pub fn new() -> MovePixelsTool {
        MovePixelsTool {
            current_mouse_position: None,
            selection: None,
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
        match (self.selection.as_ref(),
               self.move_position.as_ref(),
               self.moved_pixels_image.as_ref()) {
            (Some(selection), Some(move_position), Some(moved_pixels_image)) => {
                return Some(
                    ImageOperation::Sequential(
                        vec![
                            ImageOperation::FillRectangle {
                                start_x: selection.start_x,
                                start_y: selection.start_y,
                                end_x: selection.end_x,
                                end_y: selection.end_y,
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
    fn handle_command(&mut self, command: &Command) {
        match command {
            Command::SetSelection(selection) => {
                self.selection = selection.clone();
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
                if self.moved_pixels_image.is_none() {
                    if let Some(selection) = self.selection.as_ref() {
                        self.moved_pixels_image = Some(sub_image(image, selection.start_x, selection.start_y, selection.end_x, selection.end_y));
                        self.is_moving = true;
                    }
                } else {
                    self.is_moving = true;
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.is_moving = false;
            }
            glfw::WindowEvent::Key(Key::Enter, _, Action::Release, _) => {
                op = self.create_move(false);

                self.selection = None;
                command_buffer.push(Command::SetSelection(None));

                self.is_moving = false;
                self.moved_pixels_image = None;
                self.move_position = None;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                let mouse_position = transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                if self.is_moving {
                    self.move_position = Some(mouse_position);
                }

                self.current_mouse_position = Some(mouse_position);
            }
            _ => {}
        }

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        let mut update_op = preview_image.update_operation();
        if let Some(selection) = self.selection.as_ref() {
            if let Some(move_op) = self.create_move(true) {
                let move_position = self.move_position.as_ref().unwrap();
                move_op.apply(&mut update_op, false);
                self.create_selection(
                    move_position,
                    &Position::new(
                        move_position.x + (selection.end_x as f32 - selection.start_x as f32),
                        move_position.y + (selection.end_y as f32 - selection.start_y as f32)
                    )
                ).apply(&mut update_op, false);
            } else {
                self.create_selection(
                    &selection.start_position(),
                    &selection.end_position(),
                ).apply(&mut update_op, false);
            }
        }

        return true;
    }
}