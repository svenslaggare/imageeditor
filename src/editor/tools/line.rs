use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation};
use crate::ui::button::{TextButton, GenericButton};
use crate::program::Renders;

pub struct LineDrawTool {
    start_position: Option<Position>,
    end_position: Option<Position>,
    color: editor::Color,
    side_half_width: i32,
    change_size_button: TextButton<i32>
}

impl LineDrawTool {
    pub fn new(renders: &Renders) -> LineDrawTool {
        LineDrawTool {
            start_position: None,
            end_position: None,
            color: image::Rgba([0, 0, 0, 255]),
            side_half_width: 1,
            change_size_button: TextButton::new(
                renders.ui_font.clone(),
                "".to_owned(),
                Position::new(70.0, 10.0),
                Some(Box::new(|side_half_width| {
                    *side_half_width += 1;
                })),
                Some(Box::new(|side_half_width| {
                    *side_half_width = (*side_half_width - 1).max(0);
                })),
                None,
            )
        }
    }

    fn create_op(&self, start_position: &Position, end_position: &Position) -> ImageOperation {
        ImageOperation::Line {
            start_x: start_position.x as i32,
            start_y: start_position.y as i32,
            end_x: end_position.x as i32,
            end_y: end_position.y as i32,
            color: self.color,
            anti_aliased: None,
            side_half_width: self.side_half_width
        }
    }
}

impl Tool for LineDrawTool {
    fn handle_command(&mut self, command: &Command) {
        match command {
            Command::SetColor(color) => {
                self.color = *color;
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
                    op = Some(self.create_op(&start_position, &end_position));
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

        self.change_size_button.process_gui_event(window, event, &mut self.side_half_width);

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        let mut update_op = preview_image.update_operation();
        if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
            self.create_op(&start_position, &end_position).apply(&mut update_op, false);
        }

        return true;
    }

    fn render(&mut self, renders: &Renders, transform: &Matrix4<f32>) {
        self.change_size_button.change_text(format!("Line width: {}", self.side_half_width * 2 + 1));
        self.change_size_button.render(renders, transform);
    }
}
