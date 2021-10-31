use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use gtk::prelude::*;
use gtk::{GLArea, gio, Application, ApplicationWindow, glib, FileChooserAction, ResponseType};

use crate::gtk_app::GTKProgram;
use crate::gtk_app::helpers::{create_entry, create_file_dialog};
use crate::command_buffer::Command;

pub fn add(app: &Application,
           window: &ApplicationWindow,
           gtk_program: Rc<RefCell<Option<GTKProgram>>>,
           gl_area: Rc<GLArea>) {
    let menu = gio::Menu::new();
    let menu_bar = gio::Menu::new();

    app.set_app_menu(Some(&menu));
    app.set_menubar(Some(&menu_bar));

    // New image
    menu.append(Some("New"), Some("app.new_image"));
    let new_image = gio::SimpleAction::new("new_image", None);

    let new_file_dialog = gtk::DialogBuilder::new()
        .transient_for(window)
        .title("New image")
        .resizable(false)
        .modal(true)
        .build();

    new_file_dialog.content_area().set_spacing(8);

    new_file_dialog.add_buttons(&[
        ("Cancel", gtk::ResponseType::Cancel),
        ("Create", gtk::ResponseType::Ok)
    ]);

    let entry_width = create_entry(&new_file_dialog.content_area(), "Width: ", "1280");
    let entry_height = create_entry(&new_file_dialog.content_area(), "Height:", "800");

    let new_file_dialog = Rc::new(new_file_dialog);
    new_file_dialog.connect_delete_event(|dialog, event| {
        Inhibit(true)
    });

    let new_file_dialog_clone = new_file_dialog.clone();
    new_image.connect_activate(glib::clone!(@weak window => move |_, _| {
        new_file_dialog_clone.show_all();
    }));

    let gtk_program_clone = gtk_program.clone();
    new_file_dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Ok => {
                if let Some(gtk_program) = gtk_program_clone.borrow_mut().as_mut() {
                    match (u32::from_str(entry_width.text().as_ref()), u32::from_str(entry_height.text().as_ref())) {
                        (Ok(width), Ok(height)) => {
                            gtk_program.program.command_buffer.push(Command::NewImage(width, height));
                            dialog.hide();
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                dialog.hide();
            }
        }
    });
    app.add_action(&new_image);

    // Open
    menu.append(Some("Open"), Some("app.open_file"));
    let open_file = gio::SimpleAction::new("open_file", None);

    let gl_area_clone = gl_area.clone();
    let open_file_dialog = create_file_dialog(
        window,
        gtk_program.clone(),
        FileChooserAction::Open,
        move |program, filename| {
            match image::open(&filename) {
                Ok(image) => {
                    program.program.command_buffer.push(Command::SwitchImage(image.into_rgba()));
                    gl_area_clone.queue_render();
                }
                Err(err) => {
                    println!("Failed to open file due to: {:?}.", err);
                }
            }
        }
    );

    let open_file_dialog_clone = open_file_dialog.clone();
    open_file.connect_activate(glib::clone!(@weak window => move |_, _| {
        open_file_dialog_clone.show();
    }));
    app.add_action(&open_file);

    // Save as
    menu.append(Some("Save as"), Some("app.save_file_as"));
    let save_file_as = gio::SimpleAction::new("save_file_as", None);

    let save_file_as_dialog = create_file_dialog(
        window,
        gtk_program.clone(),
        FileChooserAction::Save,
        |program, filename| {
            if let Err(err) = program.program.editor.image().save(&filename) {
                println!("Failed to save file due to: {:?}.", err);
            }
        }
    );

    let save_file_as_dialog_clone = save_file_as_dialog.clone();
    save_file_as.connect_activate(glib::clone!(@weak window => move |_, _| {
        save_file_as_dialog_clone.show();
    }));
    app.add_action(&save_file_as);

    // Quit
    menu.append(Some("Quit"), Some("app.quit"));
    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(glib::clone!(@weak window => move |_, _| {
        window.close();
    }));
    app.add_action(&quit);

    let edit_menu = gio::Menu::new();
    menu_bar.append_submenu(Some("_Edit"), &edit_menu);

    // Undo
    edit_menu.append(Some("Undo"), Some("app.undo"));
    let undo = gio::SimpleAction::new("undo", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    undo.connect_activate(glib::clone!(@weak window => move |_, _| {
        if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
            program.program.command_buffer.push(Command::UndoImageOp);
            gl_area_clone.queue_render();
        }
    }));
    app.add_action(&undo);

    // Redo
    edit_menu.append(Some("Redo"), Some("app.redo"));
    let redo = gio::SimpleAction::new("redo", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    redo.connect_activate(glib::clone!(@weak window => move |_, _| {
        if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
            program.program.command_buffer.push(Command::RedoImageOp);
            gl_area_clone.queue_render();
        }
    }));
    app.add_action(&redo);

    edit_menu.append(Some("Select all"), Some("app.select_all"));
    let select_all = gio::SimpleAction::new("select_all", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    select_all.connect_activate(glib::clone!(@weak window => move |_, _| {
        if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
            program.program.command_buffer.push(Command::SelectAll);
            gl_area_clone.queue_render();
        }
    }));
    app.add_action(&select_all);

    let layer_menu = gio::Menu::new();
    menu_bar.append_submenu(Some("_Layer"), &layer_menu);

    // New layer
    layer_menu.append(Some("New layer"), Some("app.new_layer"));
    let new_layer = gio::SimpleAction::new("new_layer", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    new_layer.connect_activate(glib::clone!(@weak window => move |_, _| {
        if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
            program.program.command_buffer.push(Command::NewLayer);
            gl_area_clone.queue_render();
        }
    }));
    app.add_action(&new_layer);

    // Duplicate layer
    layer_menu.append(Some("Duplicate layer"), Some("app.duplicate_layer"));
    let duplicate_layer = gio::SimpleAction::new("duplicate_layer", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    duplicate_layer.connect_activate(glib::clone!(@weak window => move |_, _| {
        if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
            program.program.command_buffer.push(Command::DuplicateLayer);
            gl_area_clone.queue_render();
        }
    }));
    app.add_action(&duplicate_layer);

    // Delete layer
    layer_menu.append(Some("Delete layer"), Some("app.delete_layer"));
    let delete_layer = gio::SimpleAction::new("delete_layer", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    delete_layer.connect_activate(glib::clone!(@weak window => move |_, _| {
        if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
            program.program.command_buffer.push(Command::DeleteLayer);
            gl_area_clone.queue_render();
        }
    }));
    app.add_action(&delete_layer);
}