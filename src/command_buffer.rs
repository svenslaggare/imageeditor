use std::collections::VecDeque;
use std::path::PathBuf;

use crate::editor::image_operation::ImageOperation;
use crate::editor::tools::Tools;

#[derive(Debug)]
pub enum Command {
    SetImageSize(u32, u32),
    NewImage(u32, u32),
    SwitchImage(PathBuf, image::RgbaImage),
    SetTool(Tools),
    SwitchToPrevTool,
    SetColor(image::Rgba<u8>),
    SetAlternativeColor(image::Rgba<u8>),
    ApplyImageOp(ImageOperation),
    UndoImageOp,
    RedoImageOp,
    NewLayer,
    DuplicateLayer,
    DeleteLayer,
    SelectAll,
    ResizeImage(u32, u32),
    ResizeCanvas(u32, u32),
    RequestResizeCanvas(u32, u32),
    AbortedResizeCanvas
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
