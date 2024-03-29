use std::collections::VecDeque;
use std::path::PathBuf;

use crate::editor::image_operation::ImageOperation;
use crate::editor::tools::Tools;
use crate::editor::tools::selection::Selection;
use crate::program::{ProgramAction, ProgramActionData};

#[derive(Debug)]
pub enum BackgroundType {
    Transparent,
    Color(image::Rgba<u8>)
}

#[derive(Debug)]
pub enum Command {
    SetImageSize(u32, u32),
    NewImage(u32, u32, BackgroundType),
    SwitchImage(PathBuf, image::RgbaImage),
    SetTool(Tools),
    SwitchToPrevTool,
    SwitchedTool(Tools),
    SetPrimaryColor(image::Rgba<u8>),
    SetSecondaryColor(image::Rgba<u8>),
    SetSelection(Option<Selection>),
    SetClipboard(image::RgbaImage),
    SetCopiedImage(image::RgbaImage),
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
    AbortedResizeCanvas,
    TriggerProgramAction(ProgramAction, ProgramActionData)
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
