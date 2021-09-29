use std::sync::mpsc::Receiver;

use glfw::{Context, Key, Action, Glfw, Window, WindowEvent, Modifiers};
use glfw::WindowEvent::MouseButton;
use gl::types::*;

use image::{GenericImageView, DynamicImage, RgbaImage};

mod program;
mod helpers;
mod command_buffer;
mod rendering;
mod ui;
mod editor;

use crate::program::Program;

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

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: ./imageeditor <filename>")
    }

    // let image_to_edit = image::open(&args[1]).unwrap().into_rgba();

    let width = 1280;
    let height = 800;
    let image_to_edit = image::RgbaImage::new(width, height);

    let (mut glfw, mut window, mut events) = setup_window(image_to_edit.width(), image_to_edit.height());

    let mut program = Program::new(
        editor::Editor::new(
            editor::Image::new(
                image_to_edit
            )
        ),
        ui::create(),
    );

    let target_fps = 60.0;

    while !window.should_close() {
        let frame_start_draw = std::time::Instant::now();

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

        program.render(&transform);

        window.swap_buffers();
        glfw.poll_events();

        let duration = (std::time::Instant::now() - frame_start_draw).as_millis() as f32 / 1000.0;
        let mut wait_time = (1.0 / target_fps - duration) as f32;
        if wait_time < 0.0 {
            wait_time = 0.0;
        }

        std::thread::sleep(std::time::Duration::from_nanos((1.0E9 * wait_time) as u64));
    }
}
