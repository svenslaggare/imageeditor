use std::ops::DerefMut;
use std::cell::RefCell;
use std::rc::Rc;

use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4, Matrix};

use crate::rendering::prelude::{Position, Rectangle};
use crate::{editor, rendering};
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position, EditorWindow};
use crate::editor::image_operation::{ImageOperation, ImageOperationMarker};
use crate::program::Renders;
use crate::rendering::text_render::TextAlignment;
use crate::ui::button::{TextButton, GenericButton, Checkbox};
use crate::rendering::font::Font;
use crate::editor::Image;

pub struct PencilDrawTool {
    is_drawing: Option<editor::Color>,
    prev_mouse_position: Option<Position>,
    prev_prev_mouse_position: Option<Position>,
    color: editor::Color,
    alternative_color: editor::Color,
    side_half_width: i32,
    change_size_button: TextButton<i32>,
    anti_aliasing_checkbox: Checkbox<()>
}

impl PencilDrawTool {
    pub fn new(renders: &Renders) -> PencilDrawTool {
        PencilDrawTool {
            is_drawing: None,
            prev_mouse_position: None,
            prev_prev_mouse_position: None,
            color: image::Rgba([0, 0, 0, 255]),
            alternative_color: image::Rgba([0, 0, 0, 255]),
            side_half_width: 1,
            change_size_button: TextButton::new(
                renders.ui_font.clone(),
                "".to_owned(),
                Position::new(70.0, 10.0),
                Some(Box::new(|side_half_width| {
                    *side_half_width = (*side_half_width + 1).min(100);
                })),
                Some(Box::new(|side_half_width| {
                    *side_half_width = (*side_half_width - 1).max(0);
                })),
                None,
            ),
            anti_aliasing_checkbox: Checkbox::new(
                &image::open("content/ui/checkbox_unchecked.png").unwrap().into_rgba(),
                &image::open("content/ui/checkbox_checked.png").unwrap().into_rgba(),
                renders.ui_font.clone(),
                "Anti-aliasing".to_owned(),
                true,
                Position::new(235.0, 16.0),
                None
            )
        }
    }
}

impl Tool for PencilDrawTool {
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
                         window: &mut dyn EditorWindow,
                         event: &WindowEvent,
                         image_area_transform: &Matrix3<f32>,
                         image_area_rectangle: &Rectangle,
                         _command_buffer: &mut CommandBuffer,
                         _image: &editor::Image) -> Option<ImageOperation> {
        let create_begin_draw = |this: &Self, mouse_position: Position, color: editor::Color| {
            if this.anti_aliasing_checkbox.checked {
                Some(
                    ImageOperation::Sequential(
                        vec![
                            ImageOperation::Marker(ImageOperationMarker::BeginDraw),
                            ImageOperation::FillCircle {
                                center_x: mouse_position.x as i32,
                                center_y: mouse_position.y as i32,
                                radius: this.side_half_width,
                                color
                            },
                            ImageOperation::Circle {
                                center_x: mouse_position.x as i32,
                                center_y: mouse_position.y as i32,
                                radius: this.side_half_width - 4,
                                border_half_width: 2,
                                color,
                                anti_aliased: Some(this.anti_aliasing_checkbox.checked)
                            }
                        ]
                    )
                )
            } else {
                Some(
                    ImageOperation::Sequential(
                        vec![
                            ImageOperation::Marker(ImageOperationMarker::BeginDraw),
                            ImageOperation::Block {
                                x: mouse_position.x as i32,
                                y: mouse_position.y as i32,
                                side_half_width: this.side_half_width,
                                color
                            },
                        ]
                    )
                )
            }
        };

        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                let (mouse_x, mouse_y) = window.get_cursor_pos();
                let mouse_position = Position::new(mouse_x as f32, mouse_y as f32);
                if image_area_rectangle.contains(&mouse_position) {
                    self.is_drawing = Some(self.color);
                    op = create_begin_draw(self, get_transformed_mouse_position(window, image_area_transform), self.color);
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Press, _) => {
                let (mouse_x, mouse_y) = window.get_cursor_pos();
                let mouse_position = Position::new(mouse_x as f32, mouse_y as f32);
                if image_area_rectangle.contains(&mouse_position) {
                    self.is_drawing = Some(self.alternative_color);
                    op = create_begin_draw(self, get_transformed_mouse_position(window, image_area_transform), self.alternative_color);
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1 | glfw::MouseButton::Button2, Action::Release, _) => {
                if self.is_drawing.is_some() {
                    self.is_drawing = None;
                    self.prev_mouse_position = None;
                    self.prev_prev_mouse_position = None;
                    op = Some(ImageOperation::Marker(ImageOperationMarker::EndDraw));
                }
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                if let Some(color) = self.is_drawing {
                    let mouse_position = image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                    if let Some(prev_mouse_position) = self.prev_mouse_position {
                        op = Some(
                            ImageOperation::PencilStroke {
                                start_x: prev_mouse_position.x as i32,
                                start_y: prev_mouse_position.y as i32,
                                end_x: mouse_position.x as i32,
                                end_y: mouse_position.y as i32,
                                prev_start_x: self.prev_prev_mouse_position.map(|pos| pos.x as i32),
                                prev_start_y: self.prev_prev_mouse_position.map(|pos| pos.y as i32),
                                color,
                                anti_aliased: Some(self.anti_aliasing_checkbox.checked),
                                side_half_width: self.side_half_width
                            }
                        );
                    }

                    self.prev_prev_mouse_position = self.prev_mouse_position;
                    self.prev_mouse_position = Some(mouse_position);
                }
            }
            _ => {}
        }

        self.change_size_button.process_gui_event(window, event, &mut self.side_half_width);
        self.anti_aliasing_checkbox.process_gui_event(window, event, &mut ());

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, _preview_image: &mut editor::Image) -> bool {
        false
    }

    fn render(&mut self, renders: &Renders, transform: &Matrix4<f32>, _image_area_transform: &Matrix4<f32>, _image: &editor::Image) {
        self.change_size_button.change_text(format!("Pencil size: {}", self.side_half_width * 2 + 1));
        self.change_size_button.render(renders, transform);

        self.anti_aliasing_checkbox.render(renders, transform);
    }
}
