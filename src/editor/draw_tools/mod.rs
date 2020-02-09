use glfw::WindowEvent;
use cgmath::{Matrix3, Transform};

use crate::command_buffer::Command;
use crate::editor;
use crate::editor::image_operation::ImageOperation;
use crate::rendering::prelude::Position;
use crate::editor::draw_tools::pencil::PencilDrawTool;
use crate::editor::draw_tools::line::LineDrawTool;
use crate::editor::draw_tools::rectangle::RectangleDrawTool;
use crate::editor::draw_tools::selection::SelectionDrawTool;
use crate::editor::draw_tools::effect::EffectDrawTool;

pub trait DrawTool {
    fn on_active(&mut self) {

    }

    fn update(&mut self) {

    }

    fn handle_command(&mut self, command: &Command);

    fn process_event(&mut self,
                     window: &mut glfw::Window,
                     event: &WindowEvent,
                     transform: &Matrix3<f32>,
                     image: &editor::Image) -> Option<ImageOperation>;

    fn preview(&mut self, image: &editor::Image, preview_image: &mut editor::Image) -> bool;
}

pub mod pencil;
pub mod line;
pub mod rectangle;
pub mod selection;
pub mod effect;

pub fn get_valid_rectangle(start_position: &Position, end_position: &Position) -> (i32, i32, i32, i32) {
    let mut start_x = start_position.x as i32;
    let mut start_y = start_position.y as i32;
    let mut end_x = end_position.x as i32;
    let mut end_y = end_position.y as i32;

    if start_x > end_x {
        std::mem::swap(&mut start_x, &mut end_x);
    }

    if start_y > end_y {
        std::mem::swap(&mut start_y, &mut end_y);
    }

    (start_x, start_y, end_x, end_y)
}

pub fn get_transformed_mouse_position(window: &mut glfw::Window, transform: &Matrix3<f32>) -> Position {
    let (mouse_x, mouse_y) = window.get_cursor_pos();
    transform.transform_point(cgmath::Point2::new(mouse_x as f32, mouse_y as f32))
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DrawTools {
    Pencil = 0,
    Line = 1,
    Rectangle = 2,
    Selection = 3,
    Effect = 4,
}

pub fn create_draw_tools() -> Vec<Box<dyn DrawTool>> {
    vec![
        Box::new(PencilDrawTool::new()),
        Box::new(LineDrawTool::new()),
        Box::new(RectangleDrawTool::new()),
        Box::new(SelectionDrawTool::new()),
        Box::new(EffectDrawTool::new("content/shaders/sample.fs"))
    ]
}