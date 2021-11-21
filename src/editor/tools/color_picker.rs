use glfw::{WindowEvent, Action};
use cgmath::{Matrix3};

use crate::rendering::prelude::{Rectangle};
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position, EditorWindow};
use crate::editor::image_operation::{ImageOperation, ImageSource};

enum ColorPickerMode {
    None,
    Color,
    AlternativeColor
}

pub struct ColorPickerTool {
    mode: ColorPickerMode
}

impl ColorPickerTool {
    pub fn new() -> ColorPickerTool {
        ColorPickerTool {
            mode: ColorPickerMode::None
        }
    }

    fn do_color_select(&self,
                       window: &mut dyn EditorWindow,
                       image_area_transform: &Matrix3<f32>,
                       image: &editor::Image,
                       command_buffer: &mut CommandBuffer) {
        match self.mode {
            ColorPickerMode::None => {

            }
            ColorPickerMode::Color => {
                if let Some(color) = self.select_color(window, image_area_transform, image) {
                    command_buffer.push(Command::SetPrimaryColor(color))
                }
            }
            ColorPickerMode::AlternativeColor => {
                if let Some(color) = self.select_color(window, image_area_transform, image) {
                    command_buffer.push(Command::SetSecondaryColor(color))
                }
            }
        }
    }

    fn select_color(&self,
                    window: &mut dyn EditorWindow,
                    transform: &Matrix3<f32>,
                    image: &editor::Image) -> Option<editor::Color> {
        let position = get_transformed_mouse_position(window, transform);
        let position_x = position.x.round() as i32;
        let position_y = position.y.round() as i32;

        if position_x >= 0 && position_x < image.width() as i32 && position_y >= 0 && position_y < image.height() as i32 {
            Some(image.get_pixel(position_x as u32, position_y as u32))
        } else {
            None
        }
    }
}

impl Tool for ColorPickerTool {
    fn process_gui_event(&mut self,
                         window: &mut dyn EditorWindow,
                         event: &WindowEvent,
                         image_area_transform: &Matrix3<f32>,
                         _image_area_rectangle: &Rectangle,
                         command_buffer: &mut CommandBuffer,
                         image: &editor::Image) -> Option<ImageOperation> {
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.mode = ColorPickerMode::Color;
                self.do_color_select(window, image_area_transform, image, command_buffer);
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.mode = ColorPickerMode::None;
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Press, _) => {
                self.mode = ColorPickerMode::AlternativeColor;
                self.do_color_select(window, image_area_transform, image, command_buffer);
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button2, Action::Release, _) => {
                self.mode = ColorPickerMode::None;
            }
            glfw::WindowEvent::CursorPos(_, _) => {
                self.do_color_select(window, image_area_transform, image, command_buffer);
            }
            _ => {}
        }

        None
    }

    fn preview(&mut self,
               _image: &editor::Image,
               _preview_image: &mut editor::Image,
               _transparent_area: &mut Option<Rectangle>) -> bool {
        false
    }
}
