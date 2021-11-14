use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, GLArea, Orientation, EventBox, gdk, gdk_pixbuf};
use gtk::gio::ApplicationFlags;

use crate::gtk_app::{GTKProgram, menu, input_support};
use crate::program::{SIDE_PANELS_WIDTH, TOP_PANEL_HEIGHT, ProgramActionData, ProgramAction};
use crate::editor::EditorImage;
use crate::command_buffer::Command;
use gtk::gdk_pixbuf::Colorspace;


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

        let (image_to_edit_path, image_to_edit) = if program_args.len() >= 2 {
            let image_to_edit_path = Path::new(&program_args[1]).to_path_buf();
            let image_to_edit = image::open(&image_to_edit_path).unwrap().into_rgba();
            (Some(image_to_edit_path), image_to_edit)
        } else {
            (None, image::RgbaImage::new(1280, 800))
        };

        let width = (image_to_edit.width() + SIDE_PANELS_WIDTH) as i32;
        let height = (image_to_edit.height() + TOP_PANEL_HEIGHT + 27) as i32;

        let window = ApplicationWindow::builder()
            .application(app)
            .title("ImageEditor")
            .default_width(width)
            .default_height(height)
            .build();

        let gtk_program = Rc::new(GTKProgram::new());
        let clipboard = Rc::new(gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD));

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

        let gtk_program_clone = gtk_program.clone();
        let image_to_edit = Rc::new(RefCell::new(Some(image_to_edit)));
        let clipboard_clone = clipboard.clone();
        gl_area.connect_realize(move |area| {
            area.context().unwrap().make_current();
            gtk_program_clone.initialize(
                width as u32,
                height as u32,
                EditorImage::from_rgba(
                    image_to_edit_path.clone(),
                    image_to_edit.borrow_mut().take().unwrap()
                )
            );

            let gtk_program_clone = gtk_program_clone.clone();
            clipboard_clone.request_contents(
                &gdk::Atom::intern("image/png"),
                move |_, data| {
                    if data.data_type().name() == "image/png" {
                        let image = image::load_from_memory_with_format(data.data().as_slice(), image::ImageFormat::PNG).unwrap().into_rgba();
                        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                            program.command_buffer.push(Command::SetClipboard(image));
                        }
                    }
                }
            );
        });

        let clipboard_clone = clipboard.clone();
        gtk_program.actions.borrow_mut().insert(
            ProgramAction::SetCopiedImage,
            Box::new(move |data| {
                if let ProgramActionData::Image(image) = data {
                    let gtk_image = gdk_pixbuf::Pixbuf::new(
                        Colorspace::Rgb,
                        true,
                        8,
                        image.width() as i32,
                        image.height() as i32,
                    ).unwrap();

                    for y in 0..image.height() {
                        for x in 0..image.width() {
                            let pixel = image.get_pixel(x, y);
                            gtk_image.put_pixel(x, y, pixel[0], pixel[1], pixel[2], pixel[3]);
                        }
                    }

                    clipboard_clone.set_image(&gtk_image);
                }
            })
        );

        let window = Rc::new(window);
        let window_clone = window.clone();

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
                window_clone.set_title(&format!(
                    "ImageEditor - {}",
                    program.editor.image().path().map(|path| path.file_name().unwrap().to_str().unwrap()).unwrap_or("Untitled")
                ));

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
                    _ => ProgramActionData::None
                };

                if triggered.is_triggered() {
                    callback(triggered);
                }
            }

            Inhibit(true)
        });

        window.show_all();
    });

    application.run();
}