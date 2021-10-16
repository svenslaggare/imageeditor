use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4};

use crate::rendering::prelude::{Position, Rectangle};
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position, EditorWindow};
use crate::editor::image_operation::{ImageOperation};
use crate::ui::button::{TextButton, GenericButton};
use crate::program::Renders;
use crate::editor::Image;

pub struct CircleDrawTool {
    start_position: Option<Position>,
    end_position: Option<Position>,
    border_color: editor::Color,
    fill_color: editor::Color,
    border_half_width: i32,
    change_border_size_button: TextButton<i32>
}

impl CircleDrawTool {
    pub fn new(renders: &Renders) -> CircleDrawTool {
        CircleDrawTool {
            start_position: None,
            end_position: None,
            border_color: image::Rgba([0, 0, 0, 255]),
            fill_color: image::Rgba([255, 0, 0, 255]),
            border_half_width: 1,
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
                border_half_width: self.border_half_width,
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

    fn process_gui_event(&mut self,
                         window: &mut dyn EditorWindow,
                         event: &WindowEvent,
                         image_area_transform: &Matrix3<f32>,
                         image_area_rectangle: &Rectangle,
                         command_buffer: &mut CommandBuffer,
                         image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.start_position = Some(get_transformed_mouse_position(window, image_area_transform));
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
                let mouse_position = image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));
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

    fn render(&mut self, renders: &Renders, transform: &Matrix4<f32>, image_area_transform: &Matrix4<f32>) {
        self.change_border_size_button.change_text(format!("Border size: {}", self.border_half_width * 2 + 1));
        self.change_border_size_button.render(renders, transform);
    }
}
