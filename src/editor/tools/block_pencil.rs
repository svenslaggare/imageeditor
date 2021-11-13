use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4};

use crate::rendering::prelude::{Position, Rectangle};
use crate::{editor};
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position, EditorWindow};
use crate::editor::image_operation::{ImageOperation, ImageOperationMarker};
use crate::program::Renders;
use crate::ui::button::{TextButton, GenericButton};

pub struct BlockPencilDrawTool {
    is_drawing: Option<editor::Color>,
    prev_mouse_position: Option<Position>,
    prev_prev_mouse_position: Option<Position>,
    color: editor::Color,
    alternative_color: editor::Color,
    side_half_width: i32,
    change_size_button: TextButton<i32>
}

impl BlockPencilDrawTool {
    pub fn new(renders: &Renders) -> BlockPencilDrawTool {
        BlockPencilDrawTool {
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
            )
        }
    }
}

impl Tool for BlockPencilDrawTool {
    fn handle_command(&mut self, _command_buffer: &mut CommandBuffer, _image: &editor::Image, command: &Command) {
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
            Some(
                ImageOperation::Sequential(
                    None,
                    vec![
                        ImageOperation::Marker(ImageOperationMarker::BeginDraw, Some("Block pencil".to_owned())),
                        ImageOperation::Block {
                            x: mouse_position.x as i32,
                            y: mouse_position.y as i32,
                            side_half_width: this.side_half_width,
                            color
                        }
                    ]
                )
            )
        };

        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                let (mouse_x, mouse_y) = window.get_cursor_pos();
                let mouse_position = Position::new(mouse_x as f32, mouse_y as f32);
                if image_area_rectangle.contains(&mouse_position) {
                    let already_drawing = self.is_drawing.is_some();
                    self.is_drawing = Some(self.color);

                    if !already_drawing {
                        op = create_begin_draw(self, get_transformed_mouse_position(window, image_area_transform), self.color);
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Press, _) => {
                let (mouse_x, mouse_y) = window.get_cursor_pos();
                let mouse_position = Position::new(mouse_x as f32, mouse_y as f32);
                if image_area_rectangle.contains(&mouse_position) {
                    let already_drawing = self.is_drawing.is_some();
                    self.is_drawing = Some(self.alternative_color);

                    if !already_drawing {
                        op = create_begin_draw(self, get_transformed_mouse_position(window, image_area_transform), self.alternative_color);
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1 | glfw::MouseButton::Button2, Action::Release, _) => {
                if self.is_drawing.is_some() {
                    self.is_drawing = None;
                    self.prev_mouse_position = None;
                    self.prev_prev_mouse_position = None;
                    op = Some(ImageOperation::Marker(ImageOperationMarker::EndDraw, None));
                }
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                if let Some(color) = self.is_drawing {
                    let mouse_position = image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                    if let Some(prev_mouse_position) = self.prev_mouse_position {
                        op = Some(
                            ImageOperation::Line {
                                start_x: prev_mouse_position.x as i32,
                                start_y: prev_mouse_position.y as i32,
                                end_x: mouse_position.x as i32,
                                end_y: mouse_position.y as i32,
                                color,
                                anti_aliased: Some(false),
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

        return op;
    }

    fn preview(&mut self,
               _image: &editor::Image,
               _preview_image: &mut editor::Image,
               _transparent_area: &mut Option<Rectangle>) -> bool {
        false
    }

    fn render_ui(&mut self, renders: &Renders, transform: &Matrix4<f32>, _image_area_transform: &Matrix4<f32>, _image: &editor::Image) {
        self.change_size_button.change_text(format!("Pencil size: {}", self.side_half_width * 2 + 1));
        self.change_size_button.render(renders, transform);
    }
}
