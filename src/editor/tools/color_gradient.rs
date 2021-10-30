use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4, Matrix};

use crate::rendering::prelude::{Position, Rectangle};
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position, EditorWindow};
use crate::editor::image_operation::{ImageOperation, ColorGradientType};
use crate::ui::button::{TextButton, GenericButton};
use crate::program::Renders;
use crate::editor::Image;

pub struct ColorGradientDrawTool {
    start_position: Option<Position>,
    end_position: Option<Position>,
    first_color: editor::Color,
    second_color: editor::Color,
    gradient_type: ColorGradientType,
    set_linear_button: TextButton<ColorGradientType>,
    set_radial_button: TextButton<ColorGradientType>
}

impl ColorGradientDrawTool {
    pub fn new(renders: &Renders) -> ColorGradientDrawTool {
        ColorGradientDrawTool {
            start_position: None,
            end_position: None,
            first_color: image::Rgba([0, 0, 0, 255]),
            second_color: image::Rgba([0, 0, 0, 255]),
            gradient_type: ColorGradientType::Linear,
            set_linear_button: TextButton::new(
                renders.ui_font.clone(),
                "Set linear".to_owned(),
                Position::new(70.0, 10.0),
                Some(Box::new(|gradient_type| {
                    *gradient_type = ColorGradientType::Linear;
                })),
                None,
                None,
            ),
            set_radial_button: TextButton::new(
                renders.ui_font.clone(),
                "Set radial".to_owned(),
                Position::new(180.0, 10.0),
                Some(Box::new(|gradient_type| {
                    *gradient_type = ColorGradientType::Radial;
                })),
                None,
                None,
            )
        }
    }

    fn create_op(&self, start_position: &Position, end_position: &Position) -> ImageOperation {
        ImageOperation::ColorGradient {
            start_x: start_position.x as i32,
            start_y: start_position.y as i32,
            end_x: end_position.x as i32,
            end_y: end_position.y as i32,
            first_color: self.first_color,
            second_color: self.second_color,
            gradient_type: self.gradient_type.clone()
        }
    }
}

impl Tool for ColorGradientDrawTool {
    fn handle_command(&mut self, command: &Command) {
        match command {
            Command::SetColor(color) => {
                self.first_color = *color;
            }
            Command::SetAlternativeColor(color) => {
                self.second_color = *color;
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
                    op = Some(self.create_op(&start_position, &end_position));
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

        self.set_linear_button.process_gui_event(window, event, &mut self.gradient_type);
        self.set_radial_button.process_gui_event(window, event, &mut self.gradient_type);

        return op;
    }

    fn preview(&mut self,
               _image: &editor::Image,
               preview_image: &mut editor::Image,
               _transparent_area: &mut Option<Rectangle>) -> bool {
        let mut update_op = preview_image.update_operation();
        if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
            self.create_op(&start_position, &end_position).apply(&mut update_op, false);
        }

        return true;
    }

    fn render_ui(&mut self, renders: &Renders, transform: &Matrix4<f32>, image_area_transform: &Matrix4<f32>, _image: &editor::Image) {
        self.set_linear_button.render(renders, transform);
        self.set_radial_button.render(renders, transform);
    }
}
