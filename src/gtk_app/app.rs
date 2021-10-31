use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, GLArea, Box, Orientation, EventBox, gdk, glib, Menu, AccelGroup, MenuBar, MenuItem, Image, Label, CheckMenuItem, IconSize, AccelFlags, gio, FileChooserAction, ResponseType};
use gtk::gio::ApplicationFlags;
use gtk::gdk::keys::Key;
use gtk::gdk::ScrollDirection;

use crate::program::{Program, LEFT_SIDE_PANEL_WIDTH, TOP_PANEL_HEIGHT, SIDE_PANELS_WIDTH};
use crate::editor::tools::EditorWindow;
use crate::{ui, editor};
use crate::command_buffer::Command;
use crate::gtk_app::helpers::{create_entry, create_file_dialog};
use crate::gtk_app::{GTKProgram, menu, input_support};

pub fn main() {
    let application = Application::builder()
        .application_id("imageeditor")
        .build();

    application.set_flags(ApplicationFlags::HANDLES_OPEN);
    application.connect_open(|app, files, file| {
        app.activate();
    });

    application.connect_activate(|app| {
        let program_args = std::env::args().collect::<Vec<_>>();
        let mut image_to_edit = image::open(&program_args[1]).unwrap().into_rgba();

        let width = (image_to_edit.width() + SIDE_PANELS_WIDTH) as i32;
        let height = (image_to_edit.height() + TOP_PANEL_HEIGHT + 27) as i32;

        let window = ApplicationWindow::builder()
            .application(app)
            .title("ImageEditor")
            .default_width(width)
            .default_height(height)
            .build();

        let gtk_program = Rc::new(RefCell::new(Option::<GTKProgram>::None));

        let layout = Box::new(Orientation::Vertical, 6);
        window.add(&layout);

        let gl_area = Rc::new(GLArea::new());
        let gtk_program_clone = gtk_program.clone();
        gl_area.connect_resize(move |gl_area, width, height| {
            if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
                program.change_size(width as u32, height as u32);
            }
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
            *program_clone.borrow_mut() = Some(GTKProgram::new(width as u32, height as u32, image_to_edit.borrow_mut().take().unwrap()));
        });

        gl_area.connect_render(move |area, context| {
            let mut gtk_program = gtk_program.borrow_mut();
            let gtk_program = gtk_program.as_mut().unwrap();

            context.make_current();

            unsafe {
                gl::ClearColor(1.0, 1.0, 1.0, 1.0);
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

            gtk_program.program.update(
                &mut gtk_program.editor_window,
                std::mem::take(&mut gtk_program.event_queue).into_iter()
            );

            gtk_program.program.render(
                &mut gtk_program.editor_window,
                &transform
            );

            Inhibit(true)
        });

        window.show_all();
    });

    application.run();
}