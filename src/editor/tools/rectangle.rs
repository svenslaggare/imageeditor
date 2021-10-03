use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_valid_rectangle, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation};
use crate::ui::button::{TextButton, GenericButton};
use crate::program::Renders;

pub struct RectangleDrawTool {
    start_position: Option<Position>,
    end_position: Option<Position>,
    border_color: editor::Color,
    fill_color: editor::Color,
    border_half_width: i32,
    change_border_size_button: TextButton<i32>
}

impl RectangleDrawTool {
    pub fn new(renders: &Renders) -> RectangleDrawTool {
        RectangleDrawTool {
            start_position: None,
            end_position: None,
            border_color: image::Rgba([0, 0, 0, 255]),
            fill_color: image::Rgba([255, 0, 0, 255]),
            border_half_width: 0,
            change_border_size_button: TextButton::new(
                renders.ui_font.clone(),
                "".to_owned(),
                Position::new(70.0, 10.0),
                Some(Box::new(|border_half_width| {
                    *border_half_width += 1;
                })),
                Some(Box::new(|border_half_width| {
                    *border_half_width = (*border_half_width - 1).max(0);
                })),
                None,
            )
        }
    }

    fn create_op(&self, start_position: &Position, end_position: &Position) -> ImageOperation {
        let (start_x, start_y, end_x, end_y) = get_valid_rectangle(start_position, end_position);

        ImageOperation::Sequential(vec![
            ImageOperation::FillRectangle {
                start_x,
                start_y,
                end_x,
                end_y,
                color: self.fill_color,
                blend: false
            },
            ImageOperation::Rectangle {
                start_x,
                start_y,
                end_x,
                end_y,
                color: self.border_color,
                border_half_width: self.border_half_width
            }
        ])
    }
}

impl Tool for RectangleDrawTool {
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

    fn process_gui_event(&mut self,
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

        self.change_border_size_button.process_gui_event(window, event, &mut self.border_half_width);

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        let mut update_op = preview_image.update_operation();
        if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
            self.create_op(start_position, end_position).apply(&mut update_op, false);
        }

        return true;
    }

    fn render(&mut self, renders: &Renders, transform: &Matrix4<f32>) {
        self.change_border_size_button.change_text(format!("Border size: {}", self.border_half_width * 2 + 1));
        self.change_border_size_button.render(renders, transform);
    }
}
