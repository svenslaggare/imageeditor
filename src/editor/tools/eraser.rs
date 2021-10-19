use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4};

use crate::rendering::prelude::{Position, Rectangle};
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position, EditorWindow};
use crate::editor::image_operation::{ImageOperation, ImageOperationMarker};
use crate::ui::button::{TextButton, GenericButton};
use crate::program::Renders;
use crate::editor::Image;

pub struct EraserDrawTool {
    is_drawing: bool,
    prev_mouse_position: Option<Position>,
    side_half_width: i32,
    change_size_button: TextButton<i32>
}

impl EraserDrawTool {
    pub fn new(renders: &Renders) -> EraserDrawTool {
        EraserDrawTool {
            is_drawing: false,
            prev_mouse_position: None,
            side_half_width: 3,
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
}

impl Tool for EraserDrawTool {
    fn process_gui_event(&mut self,
                         window: &mut dyn EditorWindow,
                         event: &WindowEvent,
                         image_area_transform: &Matrix3<f32>,
                         image_area_rectangle: &Rectangle,
                         command_buffer: &mut CommandBuffer,
                         image: &editor::Image) -> Option<ImageOperation> {
        let create_begin_draw = |this: &Self, mouse_position: Position| {
            Some(ImageOperation::Sequential(vec![
                ImageOperation::Marker(ImageOperationMarker::BeginDraw),
                ImageOperation::Block {
                    x: mouse_position.x as i32,
                    y: mouse_position.y as i32,
                    color: image::Rgba([0, 0, 0, 0]),
                    side_half_width: this.side_half_width
                }
            ]))
        };

        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.is_drawing = true;
                op = create_begin_draw(self, get_transformed_mouse_position(window, image_area_transform));
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1 | glfw::MouseButton::Button2, Action::Release, _) => {
                self.is_drawing = false;
                self.prev_mouse_position = None;
                op = Some(ImageOperation::Marker(ImageOperationMarker::EndDraw));
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                if self.is_drawing {
                    let mouse_position = image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                    if let Some(prev_mouse_position) = self.prev_mouse_position {
                        op = Some(
                            ImageOperation::Line {
                                start_x: prev_mouse_position.x as i32,
                                start_y: prev_mouse_position.y as i32,
                                end_x: mouse_position.x as i32,
                                end_y: mouse_position.y as i32,
                                color: image::Rgba([0, 0, 0, 0]),
                                anti_aliased: Some(false),
                                side_half_width: self.side_half_width
                            }
                        );
                    }

                    self.prev_mouse_position = Some(mouse_position);
                }
            }
            _ => {}
        }

        self.change_size_button.process_gui_event(window, event, &mut self.side_half_width);

        return op;
    }

    fn preview(&mut self, _image: &editor::Image, _preview_image: &mut editor::Image) -> bool {
        false
    }

    fn render(&mut self, renders: &Renders, transform: &Matrix4<f32>, image_area_transform: &Matrix4<f32>, _image: &editor::Image) {
        self.change_size_button.change_text(format!("Eraser size: {}", self.side_half_width * 2 + 1));
        self.change_size_button.render(renders, transform);
    }
}
