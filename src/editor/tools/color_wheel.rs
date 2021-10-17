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
use crate::ui::color_wheel::ColorWheel;

pub struct ColorWheelTool {
    color_wheel: ColorWheel
}

impl ColorWheelTool {
    pub fn new(view_size: (u32, u32)) -> ColorWheelTool {
        ColorWheelTool {
            color_wheel: ColorWheel::new(Position::new(0.5 * view_size.0 as f32, 0.5 * view_size.1 as f32))
        }
    }

}

impl Tool for ColorWheelTool {
    fn handle_command(&mut self, command: &Command) {

    }

    fn process_gui_event(&mut self,
                         window: &mut dyn EditorWindow,
                         event: &WindowEvent,
                         _image_area_transform: &Matrix3<f32>,
                         _image_area_rectangle: &Rectangle,
                         command_buffer: &mut CommandBuffer,
                         _image: &editor::Image) -> Option<ImageOperation> {
        self.color_wheel.process_gui_event(
            window,
            event,
            command_buffer
        );

        return None;
    }

    fn preview(&mut self, _image: &editor::Image, _preview_image: &mut editor::Image) -> bool {
        return false;
    }

    fn render(&mut self, renders: &Renders, transform: &Matrix4<f32>, _image_area_transform: &Matrix4<f32>) {
        self.color_wheel.render(renders, transform);
    }
}
