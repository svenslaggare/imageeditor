use std::sync::mpsc::Receiver;

use glfw::{Context, Key, Action, Glfw, Window, WindowEvent, Modifiers};
use gl::types::*;

use crate::program::{Program, LEFT_SIDE_PANEL_WIDTH, TOP_PANEL_HEIGHT, SIDE_PANELS_WIDTH};
use crate::{editor, ui};
use crate::editor::tools::EditorWindow;

pub fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: ./imageeditor <filename>");
        return;
    }

    let image_to_edit = image::open(&args[1]).unwrap().into_rgba();
    // let image_to_edit = image::open("/home/antjans/Bilder/TestImage.JPG").unwrap().into_rgba();
    let width = image_to_edit.width();
    let height = image_to_edit.height();

    // let width = 1280;
    // let height = 800;
    // let mut image_to_edit: image::RgbaImage = image::RgbaImage::new(width, height);

    // let image_to_edit = image::open("/home/antjans/Bilder/TestImage.JPG").unwrap().into_rgba();
    // let width = 1280;
    // let height = 800;

    let width = width + SIDE_PANELS_WIDTH;
    let height = height + TOP_PANEL_HEIGHT;
    let (mut glfw, mut window, mut events) = setup_window(width, height);

    let mut program = Program::new(
        width,
        height,
        editor::Editor::new(editor::Image::new(image_to_edit)),
        ui::create(),
    );

    let target_fps = 60.0;

    while !window.should_close() {
        let frame_start_draw = std::time::Instant::now();

        glfw.poll_events();
        let mut events = glfw::flush_messages(&mut events).map(|(_, event)| event);
        program.update(&mut window, &mut events);

        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let transform = cgmath::ortho(
            0.0,
            window.get_size().0 as f32,
            window.get_size().1 as f32,
            0.0,
            0.0,
            1.0
        );

        program.render(&mut window, &transform);
        window.swap_buffers();

        let duration = (std::time::Instant::now() - frame_start_draw).as_millis() as f32 / 1000.0;
        let mut wait_time = (1.0 / target_fps - duration) as f32;
        if wait_time < 0.0 {
            wait_time = 0.0;
        }

        std::thread::sleep(std::time::Duration::from_nanos((1.0E9 * wait_time) as u64));
    }
}

impl EditorWindow for glfw::Window {
    fn get_cursor_pos(&self) -> (f64, f64) {
        self.get_cursor_pos()
    }

    fn width(&self) -> u32 {
        self.get_size().0 as u32
    }

    fn height(&self) -> u32 {
        self.get_size().1 as u32
    }

    fn set_should_close(&mut self, value: bool) {
        self.set_should_close(value);
    }
}

fn setup_window(width: u32, height: u32) -> (Glfw, Window, Receiver<(f64, WindowEvent)>) {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "linux")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    let (mut window, events) = glfw.create_window(
        width,
        height,
        "ImageEditor",
        glfw::WindowMode::Windowed
    ).expect("Failed to create GLFW window");

    window.make_current();

    window.set_key_polling(true);
    window.set_sticky_keys(true);
    window.set_char_polling(true);
    window.set_scroll_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_focus_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    unsafe {
        gl::Enable(gl::MULTISAMPLE);
    }

    (glfw, window, events)
}