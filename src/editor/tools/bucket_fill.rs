use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4};

use crate::rendering::prelude::Position;
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position};
use crate::editor::image_operation::{ImageOperation};
use crate::ui::button::{TextButton, GenericButton};
use crate::program::Renders;

pub struct BucketFillDrawTool {
    color: editor::Color,
    alternative_color: editor::Color,
    tolerance: f32,
    change_tolerance_button: TextButton<f32>
}

impl BucketFillDrawTool {
    pub fn new(renders: &Renders) -> BucketFillDrawTool {
        BucketFillDrawTool {
            color: image::Rgba([0, 0, 0, 255]),
            alternative_color: image::Rgba([0, 0, 0, 255]),
            tolerance: 0.1,
            change_tolerance_button: TextButton::new(
                renders.ui_font.clone(),
                "".to_owned(),
                Position::new(70.0, 10.0),
                Some(Box::new(|tolerance| {
                    *tolerance = (*tolerance + 0.05).min(1.0);
                })),
                Some(Box::new(|tolerance| {
                    *tolerance = (*tolerance - 0.05).max(0.0);
                })),
                None,
            )
        }
    }
}

impl Tool for BucketFillDrawTool {
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

    fn process_gui_event(&mut self,
                         window: &mut glfw::Window,
                         event: &WindowEvent,
                         transform: &Matrix3<f32>,
                         _command_buffer: &mut CommandBuffer,
                         _image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;

        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                let mouse_position = get_transformed_mouse_position(window, transform);
                op = Some(
                    ImageOperation::BucketFill {
                        start_x: mouse_position.x as i32,
                        start_y: mouse_position.y as i32,
                        fill_color: self.color,
                        tolerance: self.tolerance
                    }
                );
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Press, _) => {
                let mouse_position = get_transformed_mouse_position(window, transform);
                op = Some(
                    ImageOperation::BucketFill {
                        start_x: mouse_position.x as i32,
                        start_y: mouse_position.y as i32,
                        fill_color: self.alternative_color,
                        tolerance: self.tolerance
                    }
                );
            }
            _ => {}
        }

        self.change_tolerance_button.process_gui_event(window, event, &mut self.tolerance);

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, _preview_image: &mut editor::Image) -> bool {
        return false;
    }

    fn render(&mut self, renders: &Renders, transform: &Matrix4<f32>) {
        self.change_tolerance_button.change_text(format!("Tolerance: {:.0} %", self.tolerance * 100.0));
        self.change_tolerance_button.render(renders, transform);
    }
}
