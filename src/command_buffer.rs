use std::collections::VecDeque;

use crate::editor::image_operation::ImageOperation;
use crate::editor::tools::Tools;
use crate::rendering::prelude::Position;

#[derive(Debug, Clone)]
pub struct Selection {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32
}

impl Selection {
    pub fn start_position(&self) -> Position {
        Position::new(self.start_x as f32, self.start_y as f32)
    }

    pub fn end_position(&self) -> Position {
        Position::new(self.end_x as f32, self.end_y as f32)
    }
}

#[derive(Debug)]
pub enum Command {
    SetImageSize(u32, u32),
    SetTool(Tools),
    SetColor(image::Rgba<u8>),
    SetAlternativeColor(image::Rgba<u8>),
    SetSelection(Option<Selection>),
    ApplyImageOp(ImageOperation),
    UndoImageOp,
    RedoImageOp,
}

pub struct CommandBuffer {
    queue: VecDeque<Command>
}

impl CommandBuffer {
    pub fn new() -> CommandBuffer {
        CommandBuffer {
            queue: VecDeque::new()
        }
    }

    pub fn pop(&mut self) -> Option<Command> {
        self.queue.pop_front()
    }

    pub fn push(&mut self, command: Command) {
        self.queue.push_back(command);
    }
}
