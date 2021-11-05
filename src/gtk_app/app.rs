use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, GLArea, Orientation, EventBox};
use gtk::gio::ApplicationFlags;

use crate::gtk_app::{GTKProgram, menu, input_support};
use crate::program::{SIDE_PANELS_WIDTH, TOP_PANEL_HEIGHT};

pub fn main() {
    let application = Application::builder()
        .application_id("imageeditor")
        .build();

    application.set_flags(ApplicationFlags::HANDLES_OPEN);
    application.connect_open(|app, _files, _file| {
        app.activate();
    });

    application.connect_activate(|app| {
        let program_args = std::env::args().collect::<Vec<_>>();
        let image_to_edit = image::open(&program_args[1]).unwrap().into_rgba();

        let width = (image_to_edit.width() + SIDE_PANELS_WIDTH) as i32;
        let height = (image_to_edit.height() + TOP_PANEL_HEIGHT + 27) as i32;

        let window = ApplicationWindow::builder()
            .application(app)
            .title("ImageEditor")
            .default_width(width)
            .default_height(height)
            .build();

        let gtk_program = Rc::new(GTKProgram::new());

        let layout = gtk::Box::new(Orientation::Vertical, 6);
        window.add(&layout);

        let gl_area = Rc::new(GLArea::new());
        let gtk_program_clone = gtk_program.clone();
        gl_area.connect_resize(move |_, width, height| {
            gtk_program_clone.change_size(width as u32, height as u32);
        });

        let event_box = Rc::new(EventBox::new());
        event_box.add(gl_area.deref());
        layout.pack_start(event_box.deref(), true, true, 0);

        menu::add(app, &window, gtk_program.clone(), gl_area.clone());
        input_support::add(gtk_program.clone(), gl_area.clone(), event_box.clone());

        gl_loader::init_gl();
        gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

        let program_clone = gtk_program.clone();
        let image_to_edit = Rc::new(RefCell::new(Some(image_to_edit)));
        gl_area.connect_realize(move |area| {
            area.context().unwrap().make_current();
            program_clone.initialize(width as u32, height as u32, image_to_edit.borrow_mut().take().unwrap());
        });

        gl_area.connect_render(move |area, context| {
            context.make_current();

            unsafe {
                gl::ClearColor(214.0 / 255.0, 214.0 / 255.0, 214.0 / 255.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            let transform = cgmath::ortho(
                0.0,
                area.window().unwrap().width() as f32,
                area.window().unwrap().height() as f32,
                0.0,
                0.0,
                1.0
            );

            if let (Some(program), Some(editor_window)) = (gtk_program.program.borrow_mut().as_mut(),
                                                           gtk_program.editor_window.borrow_mut().as_mut()) {
                program.update(
                    editor_window,
                    std::mem::take(gtk_program.event_queue.borrow_mut().deref_mut()).into_iter()
                );

                program.render(
                    editor_window,
                    &transform
                );
            }

            for (action, callback) in gtk_program.actions.borrow_mut().iter() {
                let triggered = match gtk_program.program.borrow_mut().as_mut() {
                    Some(program) => program.actions.is_triggered(action),
                    _ => false
                };

                if triggered {
                    callback();
                }
            }

            Inhibit(true)
        });

        window.show_all();
    });

    application.run();
}