use glfw::{WindowEvent, Action};
use cgmath::{Matrix3, Transform, Matrix4, Matrix};

use crate::rendering::prelude::{Position, Rectangle};
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_transformed_mouse_position, EditorWindow, Tools};
use crate::editor::image_operation::{ImageOperation, ColorGradientType};
use crate::ui::button::{TextButton, GenericButton};
use crate::program::Renders;
use crate::editor::Image;
use crate::ui::color_wheel::ColorWheel;

pub struct ColorWheelTool {
    color_wheel: ColorWheel
}

impl ColorWheelTool {
    pub fn new() -> ColorWheelTool {
        ColorWheelTool {
            color_wheel: ColorWheel::new()
        }
    }

}

impl Tool for ColorWheelTool {
    fn on_active(&mut self, window: &mut dyn EditorWindow, tool: Tools) -> Option<ImageOperation> {
        self.color_wheel.update_position(window);
        if let Tools::ColorWheel(mode) = tool {
            self.color_wheel.set_mode(mode);
        }

        None
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

    fn preview(&mut self,
               _image: &editor::Image,
               _preview_image: &mut editor::Image,
               _transparent_area: &mut Option<Rectangle>) -> bool {
        return false;
    }

    fn render_ui(&mut self, renders: &Renders, transform: &Matrix4<f32>, _image_area_transform: &Matrix4<f32>, _image: &editor::Image) {
        self.color_wheel.render(renders, transform);
    }
}
