use glfw::WindowEvent;

use cgmath::{Matrix3, Transform};

use crate::command_buffer::{Command, CommandBuffer};
use crate::editor;
use crate::editor::image_operation::ImageOperation;
use crate::rendering::prelude::Position;
use crate::editor::tools::pencil::PencilDrawTool;
use crate::editor::tools::line::LineDrawTool;
use crate::editor::tools::rectangle::RectangleDrawTool;
use crate::editor::tools::effect::EffectDrawTool;
use crate::editor::tools::circle::CircleDrawTool;
use crate::editor::tools::bucket_fill::BucketFillDrawTool;
use crate::editor::tools::selection::SelectionTool;

pub mod pencil;
pub mod line;
pub mod rectangle;
pub mod circle;
pub mod bucket_fill;
// pub mod selection;
// pub mod move_pixels;
pub mod selection;
pub mod effect;

pub trait Tool {
    fn on_active(&mut self, tool: Tools) -> Option<ImageOperation> {
        None
    }

    fn on_deactivate(&mut self, command_buffer: &mut CommandBuffer) -> Option<ImageOperation> {
        None
    }

    fn update(&mut self) {

    }

    fn handle_command(&mut self, command: &Command) {

    }

    fn process_event(
        &mut self,
        window: &mut glfw::Window,
        event: &WindowEvent,
        transform: &Matrix3<f32>,
        command_buffer: &mut CommandBuffer,
        image: &editor::Image
    ) -> Option<ImageOperation>;

    fn preview(&mut self, image: &editor::Image, preview_image: &mut editor::Image) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tools {
    Pencil,
    Line,
    Rectangle,
    Circle,
    Selection(SelectionSubTool),
    BucketFill
}

impl Tools {
    pub fn index(&self) -> usize {
        match self {
            Tools::Pencil => 0,
            Tools::Line => 1,
            Tools::Rectangle => 2,
            Tools::Circle => 3,
            Tools::Selection(_) => 4,
            Tools::BucketFill => 5
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionSubTool {
    Select,
    MovePixels
}

pub fn create_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(PencilDrawTool::new()),
        Box::new(LineDrawTool::new()),
        Box::new(RectangleDrawTool::new()),
        Box::new(CircleDrawTool::new()),
        Box::new(SelectionTool::new()),
        Box::new(BucketFillDrawTool::new())
    ]
}

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