use std::collections::VecDeque;

use crate::{editor, ui};
use crate::program::Program;
use crate::editor::tools::EditorWindow;

pub mod app;
pub mod helpers;
pub mod input_support;
pub mod menu;

pub struct GTKProgram {
    pub program: Program,
    pub editor_window: GTKEditorWindow,
    pub event_queue: VecDeque<glfw::WindowEvent>
}

impl GTKProgram {
    pub fn new(view_width: u32, view_height: u32, image_to_edit: image::RgbaImage) -> GTKProgram {
        let mut program = Program::new(
            view_width,
            view_height,
            editor::Editor::new(editor::Image::new(image_to_edit)),
            ui::create(),
        );

        GTKProgram {
            program,
            editor_window: GTKEditorWindow {
                mouse_position: (0.0, 0.0),
                width: view_width,
                height: view_height
            },
            event_queue: VecDeque::new()
        }
    }

    pub fn change_size(&mut self, width: u32, height: u32) {
        self.event_queue.push_back(glfw::WindowEvent::FramebufferSize(width as i32, height as i32));
        self.editor_window.width = width;
        self.editor_window.height = height;
    }
}

pub struct GTKEditorWindow {
    pub mouse_position: (f64, f64),
    pub width: u32,
    pub height: u32
}

impl EditorWindow for GTKEditorWindow {
    fn get_cursor_pos(&self) -> (f64, f64) {
        self.mouse_position
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn set_should_close(&mut self, _value: bool) {

    }
}