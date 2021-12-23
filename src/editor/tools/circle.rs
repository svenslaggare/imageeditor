use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4};

use crate::rendering::prelude::{Position, Rectangle};
use crate::{editor, content};
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position, EditorWindow};
use crate::editor::image_operation::{ImageOperation};
use crate::ui::button::{TextButton, GenericButton, Checkbox};
use crate::program::Renders;

pub struct CircleDrawTool {
    start_position: Option<Position>,
    end_position: Option<Position>,
    is_alternative_mode: bool,
    border_color: editor::Color,
    fill_color: editor::Color,
    border_half_width: i32,
    change_border_size_button: TextButton<i32>,
    anti_aliasing_checkbox: Checkbox<()>,
    border_checkbox: Checkbox<()>
}

impl CircleDrawTool {
    pub fn new(renders: &Renders) -> CircleDrawTool {
        CircleDrawTool {
            start_position: None,
            end_position: None,
            is_alternative_mode: false,
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
            ),
            anti_aliasing_checkbox: Checkbox::new(
                &image::open(content::get_path("content/ui/checkbox_unchecked.png")).unwrap().into_rgba(),
                &image::open(content::get_path("content/ui/checkbox_checked.png")).unwrap().into_rgba(),
                renders.ui_font.clone(),
                "Anti-aliasing".to_owned(),
                true,
                Position::new(235.0, 16.0),
                None
            ),
            border_checkbox: Checkbox::new(
                &image::open(content::get_path("content/ui/checkbox_unchecked.png")).unwrap().into_rgba(),
                &image::open(content::get_path("content/ui/checkbox_checked.png")).unwrap().into_rgba(),
                renders.ui_font.clone(),
                "Border".to_owned(),
                true,
                Position::new(400.0, 16.0),
                None
            )
        }
    }

    fn create_op(&self,
                 start_position: &Position,
                 end_position: &Position,
                 fill_color: editor::Color,
                 border_color: editor::Color) -> ImageOperation {
        let start_x = start_position.x as i32;
        let start_y = start_position.y as i32;
        let end_x = end_position.x as i32;
        let end_y = end_position.y as i32;
        let radius = (((end_x - start_x).pow(2) + (end_y - start_y).pow(2)) as f64).sqrt() as i32;

        if self.border_checkbox.checked {
            ImageOperation::Sequential(
                Some("Circle".to_owned()),
                vec![
                    ImageOperation::FillCircle {
                        center_x: start_x,
                        center_y: start_y,
                        radius,
                        color: fill_color,
                        blend: true
                    },
                    ImageOperation::Circle {
                        center_x: start_x,
                        center_y: start_y,
                        radius,
                        border_half_width: self.border_half_width,
                        color: border_color,
                        blend: false,
                        anti_aliased: Some(self.anti_aliasing_checkbox.checked)
                    }
                ]
            )
        } else {
            ImageOperation::Sequential(
                Some("Circle".to_owned()),
                vec![
                    ImageOperation::FillCircle {
                        center_x: start_x,
                        center_y: start_y,
                        radius: radius - 4,
                        color: fill_color,
                        blend: true,
                    },
                    ImageOperation::Circle {
                        center_x: start_x,
                        center_y: start_y,
                        radius: radius - 4,
                        border_half_width: 2,
                        color: fill_color,
                        blend: false,
                        anti_aliased: Some(self.anti_aliasing_checkbox.checked)
                    }
                ]
            )
        }
    }
}

impl Tool for CircleDrawTool {
    fn handle_command(&mut self, _command_buffer: &mut CommandBuffer, _image: &editor::Image, command: &Command) {
        match command {
            Command::SetPrimaryColor(color) => {
                self.fill_color = *color;
            }
            Command::SetSecondaryColor(color) => {
                self.border_color = *color;
            }
            _ => {}
        }
    }

    fn process_gui_event(&mut self,
                         window: &mut dyn EditorWindow,
                         event: &WindowEvent,
                         image_area_transform: &Matrix3<f32>,
                         _image_area_rectangle: &Rectangle,
                         _command_buffer: &mut CommandBuffer,
                         _image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.start_position = Some(get_transformed_mouse_position(window, image_area_transform));
                self.end_position = None;
                self.is_alternative_mode = false;
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                    op = Some(self.create_op(start_position, end_position, self.fill_color, self.border_color));
                }

                self.start_position = None;
                self.end_position = None;
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Press, _) => {
                self.start_position = Some(get_transformed_mouse_position(window, image_area_transform));
                self.end_position = None;
                self.is_alternative_mode = true;
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Release, _) => {
                if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
                    op = Some(self.create_op(start_position, end_position, self.border_color, self.fill_color));
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
        self.anti_aliasing_checkbox.process_gui_event(window, event, &mut ());
        self.border_checkbox.process_gui_event(window, event, &mut ());

        return op;
    }

    fn preview(&mut self,
               _image: &editor::Image,
               preview_image: &mut editor::Image,
               _transparent_area: &mut Option<Rectangle>) -> bool {
        let mut update_op = preview_image.update_operation();
        if let (Some(start_position), Some(end_position)) = (self.start_position.as_ref(), self.end_position.as_ref()) {
            let (fill_color, border_color) = if !self.is_alternative_mode {
                (self.fill_color, self.border_color)
            } else {
                (self.border_color, self.fill_color)
            };

            self.create_op(start_position, end_position, fill_color, border_color).apply(&mut update_op, false);
        }

        return true;
    }

    fn render_ui(&mut self, renders: &Renders, transform: &Matrix4<f32>, _image_area_transform: &Matrix4<f32>, _image: &editor::Image) {
        self.change_border_size_button.change_text(format!("Border size: {}", self.border_half_width * 2 + 1));
        self.change_border_size_button.render(renders, transform);

        self.anti_aliasing_checkbox.render(renders, transform);
        self.border_checkbox.render(renders, transform);
    }
}
