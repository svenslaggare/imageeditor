use std::collections::{VecDeque, HashMap};
use std::rc::Rc;
use std::cell::{RefCell};

use crate::{editor, ui};
use crate::program::{Program, ProgramActions};
use crate::editor::tools::EditorWindow;

pub mod app;
pub mod helpers;
pub mod input_support;
pub mod menu;

pub type GTKProgramRef = Rc<GTKProgram>;

pub struct GTKProgram {
    pub program: RefCell<Option<Program>>,
    pub editor_window: RefCell<Option<GTKEditorWindow>>,
    pub event_queue: RefCell<VecDeque<glfw::WindowEvent>>,
    pub actions: RefCell<HashMap<ProgramActions, Box<dyn Fn()>>>
}

impl GTKProgram {
    pub fn new() -> GTKProgram {
        GTKProgram {
            program: RefCell::new(None),
            editor_window: RefCell::new(None),
            event_queue: RefCell::new(VecDeque::new()),
            actions: RefCell::new(HashMap::new())
        }
    }

    pub fn initialize(&self, view_width: u32, view_height: u32, image_to_edit: image::RgbaImage) {
        *self.program.borrow_mut() = Some(
            Program::new(
                view_width,
                view_height,
                editor::Editor::new(editor::Image::new(image_to_edit)),
                ui::create(),
            )
        );

        *self.editor_window.borrow_mut() = Some(
            GTKEditorWindow {
                mouse_position: (0.0, 0.0),
                shift_down: false,
                width: view_width,
                height: view_height
            }
        );
    }

    pub fn change_size(&self, width: u32, height: u32) {
        if let Some(editor_window) = self.editor_window.borrow_mut().as_mut() {
            self.event_queue.borrow_mut().push_back(glfw::WindowEvent::FramebufferSize(width as i32, height as i32));
            editor_window.width = width;
            editor_window.height = height;
        }
    }
}

pub struct GTKEditorWindow {
    pub mouse_position: (f64, f64),
    pub shift_down: bool,
    pub width: u32,
    pub height: u32
}

impl EditorWindow for GTKEditorWindow {
    fn get_cursor_pos(&self) -> (f64, f64) {
        self.mouse_position
    }

    fn is_shift_down(&self) -> bool {
        self.shift_down
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}