use std::collections::VecDeque;

use crate::editor::image_operation::ImageOperation;
use crate::editor::draw_tools::DrawTools;

#[derive(Debug)]
pub enum Command {
    SetDrawTool(DrawTools),
    SetColor(image::Rgba<u8>),
    SetAlternativeColor(image::Rgba<u8>),
    SetImageSize(u32, u32),
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
