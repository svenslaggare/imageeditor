use std::rc::Rc;
use std::str::FromStr;
use std::iter::FromIterator;
use std::ops::{ Deref};

use gtk::prelude::*;
use gtk::{GLArea, gio, Application, ApplicationWindow, glib, FileChooserAction, ResponseType};

use crate::gtk_app::{GTKProgram, GTKProgramRef};
use crate::gtk_app::helpers::{create_entry, create_file_dialog};
use crate::command_buffer::Command;
use crate::program::ProgramActions;

pub fn add(app: &Application,
           window: &ApplicationWindow,
           gtk_program: GTKProgramRef,
           gl_area: Rc<GLArea>) {
    let menu = gio::Menu::new();
    let menu_bar = gio::Menu::new();

    app.set_app_menu(Some(&menu));
    app.set_menubar(Some(&menu_bar));

    add_program_menu(app, window, gtk_program.clone(), gl_area.clone(), &menu);
    add_edit_menu(app, window, gtk_program.clone(), gl_area.clone(), &menu_bar);
    add_image_menu(app, window, gtk_program.clone(), gl_area.clone(), &menu_bar);
    add_layers_menu(app, window, gtk_program.clone(), gl_area.clone(), &menu_bar);
}

fn add_program_menu(app: &Application,
                    window: &ApplicationWindow,
                    gtk_program: GTKProgramRef,
                    gl_area: Rc<GLArea>,
                    menu: &gio::Menu) {
    menu.append(Some("New"), Some("app.new_image"));
    let new_image = gio::SimpleAction::new("new_image", None);

    let new_image_dialog = create_dialog(window, "New image");

    new_image_dialog.add_buttons(&[
        ("Cancel", gtk::ResponseType::Cancel),
        ("Create", gtk::ResponseType::Ok)
    ]);

    let entry_width = create_entry(&new_image_dialog.content_area(), "Width: ", "1280");
    let entry_height = create_entry(&new_image_dialog.content_area(), "Height:", "800");

    let new_file_dialog = Rc::new(new_image_dialog);

    let new_file_dialog_clone = new_file_dialog.clone();
    new_image.connect_activate(glib::clone!(@weak window => move |_, _| {
        new_file_dialog_clone.show_all();
    }));

    let gtk_program_clone = gtk_program.clone();
    new_file_dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Ok => {
                match (u32::from_str(entry_width.text().as_ref()), u32::from_str(entry_height.text().as_ref())) {
                    (Ok(width), Ok(height)) => {
                        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                            program.command_buffer.push(Command::NewImage(width, height));
                        }

                        dialog.hide();
                    }
                    _ => {}
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
        "Open image",
        FileChooserAction::Open,
        move |gtk_program, filename| {
            match image::open(&filename) {
                Ok(image) => {
                    if let Some(program) = gtk_program.program.borrow_mut().as_mut() {
                        program.command_buffer.push(Command::SwitchImage(image.into_rgba()));
                        gl_area_clone.queue_render();
                    }
                }
                Err(err) => {
                    println!("Failed to open file due to: {:?}.", err);
                }
            }
        }
    );

    let open_file_dialog_clone = open_file_dialog.clone();
    gtk_program.actions.borrow_mut().insert(
        ProgramActions::OpenImage,
        Box::new(move || {
            open_file_dialog_clone.show();
        })
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
        "Save as",
        FileChooserAction::Save,
        |gtk_program, filename| {
            if let Some(program) = gtk_program.program.borrow_mut().as_mut() {
                if let Err(err) = program.editor.image().save(&filename) {
                    println!("Failed to save file due to: {:?}.", err);
                }
            }
        }
    );

    let save_file_as_dialog_clone = save_file_as_dialog.clone();
    gtk_program.actions.borrow_mut().insert(
        ProgramActions::SaveImageAs,
        Box::new(move || {
            save_file_as_dialog_clone.show();
        })
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
}

fn add_edit_menu(app: &Application,
                 window: &ApplicationWindow,
                 gtk_program: GTKProgramRef,
                 gl_area: Rc<GLArea>,
                 menu_bar: &gio::Menu) {
    let edit_menu = gio::Menu::new();
    menu_bar.append_submenu(Some("_Edit"), &edit_menu);

    // Undo
    edit_menu.append(Some("Undo"), Some("app.undo"));
    let undo = gio::SimpleAction::new("undo", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    undo.connect_activate(glib::clone!(@weak window => move |_, _| {
        gtk_program_clone.program.borrow_mut().as_mut().unwrap().command_buffer.push(Command::UndoImageOp);
        gl_area_clone.queue_render();
    }));
    app.add_action(&undo);

    // Redo
    edit_menu.append(Some("Redo"), Some("app.redo"));
    let redo = gio::SimpleAction::new("redo", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    redo.connect_activate(glib::clone!(@weak window => move |_, _| {
        gtk_program_clone.program.borrow_mut().as_mut().unwrap().command_buffer.push(Command::RedoImageOp);
        gl_area_clone.queue_render();
    }));
    app.add_action(&redo);

    edit_menu.append(Some("Select all"), Some("app.select_all"));
    let select_all = gio::SimpleAction::new("select_all", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    select_all.connect_activate(glib::clone!(@weak window => move |_, _| {
        gtk_program_clone.program.borrow_mut().as_mut().unwrap().command_buffer.push(Command::SelectAll);
        gl_area_clone.queue_render();
    }));
    app.add_action(&select_all);
}

fn add_image_menu(app: &Application,
                  window: &ApplicationWindow,
                  gtk_program: GTKProgramRef,
                  _gl_area: Rc<GLArea>,
                  menu_bar: &gio::Menu) {
    let layer_menu = gio::Menu::new();
    menu_bar.append_submenu(Some("_Image"), &layer_menu);

    layer_menu.append(Some("Resize image"), Some("app.resize_image"));
    let resize_image = gio::SimpleAction::new("resize_image", None);
    let resize_image_dialog = create_dialog(window, "Resize image");

    resize_image_dialog.add_buttons(&[
        ("Cancel", gtk::ResponseType::Cancel),
        ("Ok", gtk::ResponseType::Ok)
    ]);

    let entry_width = Rc::new(create_entry(&resize_image_dialog.content_area(), "New width: ", "0"));
    let entry_height = Rc::new(create_entry(&resize_image_dialog.content_area(), "New height:", "0"));

    let resize_image_dialog = Rc::new(resize_image_dialog);

    let gtk_program_clone = gtk_program.clone();
    let resize_image_dialog_clone = resize_image_dialog.clone();
    let entry_width_clone = entry_width.clone();
    let entry_height_clone = entry_height.clone();

    resize_image.connect_activate(glib::clone!(@weak window => move |_, _| {
        entry_width_clone.set_text(&format!("{}", gtk_program_clone.program.borrow_mut().as_mut().unwrap().editor.image().width()));
        entry_height_clone.set_text(&format!("{}", gtk_program_clone.program.borrow_mut().as_mut().unwrap().editor.image().height()));
        resize_image_dialog_clone.show_all();
    }));

    let gtk_program_clone = gtk_program.clone();
    let resize_image_dialog_clone = resize_image_dialog.clone();
    let entry_width_clone = entry_width.clone();
    let entry_height_clone = entry_height.clone();
    gtk_program.actions.borrow_mut().insert(
        ProgramActions::ResizeImage,
        Box::new(move || {
            if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                entry_width_clone.set_text(&format!("{}", program.editor.image().width()));
                entry_height_clone.set_text(&format!("{}", program.editor.image().height()));
                resize_image_dialog_clone.show_all();
            }
        })
    );

    let gtk_program_clone = gtk_program.clone();
    let entry_width_clone = entry_width.clone();
    let entry_height_clone = entry_height.clone();
    resize_image_dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Ok => {
                match parse_new_size(gtk_program_clone.deref(), entry_width_clone.as_ref(), entry_height_clone.as_ref()) {
                    Some((width, height)) => {
                        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                            program.command_buffer.push(Command::ResizeImage(width, height));
                        }

                        dialog.hide();
                    }
                    _ => {}
                }
            }
            _ => {
                dialog.hide();
            }
        }
    });
    app.add_action(&resize_image);

    // Resize canvas
    layer_menu.append(Some("Resize canvas"), Some("app.resize_canvas"));
    let resize_canvas = gio::SimpleAction::new("resize_canvas", None);
    let resize_canvas_dialog = create_dialog(window, "Resize canvas");

    resize_canvas_dialog.add_buttons(&[
        ("Cancel", gtk::ResponseType::Cancel),
        ("Ok", gtk::ResponseType::Ok)
    ]);

    let entry_width = Rc::new(create_entry(&resize_canvas_dialog.content_area(), "New width: ", "0"));
    let entry_height = Rc::new(create_entry(&resize_canvas_dialog.content_area(), "New height:", "0"));

    let resize_canvas_dialog = Rc::new(resize_canvas_dialog);

    let gtk_program_clone = gtk_program.clone();
    let resize_canvas_dialog_clone = resize_canvas_dialog.clone();
    let entry_width_clone = entry_width.clone();
    let entry_height_clone = entry_height.clone();
    resize_canvas.connect_activate(glib::clone!(@weak window => move |_, _| {
        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
            entry_width_clone.set_text(&format!("{}", program.editor.image().width()));
            entry_height_clone.set_text(&format!("{}", program.editor.image().height()));
            resize_canvas_dialog_clone.show_all();
        }
    }));

    let gtk_program_clone = gtk_program.clone();
    let resize_canvas_dialog_clone = resize_canvas_dialog.clone();
    let entry_width_clone = entry_width.clone();
    let entry_height_clone = entry_height.clone();
    gtk_program.actions.borrow_mut().insert(
        ProgramActions::ResizeCanvas,
        Box::new(move || {
            if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                entry_width_clone.set_text(&format!("{}", program.editor.image().width()));
                entry_height_clone.set_text(&format!("{}", program.editor.image().height()));
                resize_canvas_dialog_clone.show_all();
            }
        })
    );

    let gtk_program_clone = gtk_program.clone();
    let entry_width_clone = entry_width.clone();
    let entry_height_clone = entry_height.clone();
    resize_canvas_dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Ok => {
                match parse_new_size(gtk_program_clone.deref(), entry_width_clone.as_ref(), entry_height_clone.as_ref()) {
                    Some((width, height)) => {
                        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                            program.command_buffer.push(Command::ResizeCanvas(width, height));
                        }

                        dialog.hide();
                    }
                    _ => {}
                }
            }
            _ => {
                dialog.hide();
            }
        }
    });
    app.add_action(&resize_canvas);
}

fn add_layers_menu(app: &Application,
                   window: &ApplicationWindow,
                   gtk_program: GTKProgramRef,
                   gl_area: Rc<GLArea>,
                   menu_bar: &gio::Menu) {
    let layer_menu = gio::Menu::new();
    menu_bar.append_submenu(Some("_Layer"), &layer_menu);

    // New layer
    layer_menu.append(Some("New layer"), Some("app.new_layer"));
    let new_layer = gio::SimpleAction::new("new_layer", None);
    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    new_layer.connect_activate(glib::clone!(@weak window => move |_, _| {
        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
            program.command_buffer.push(Command::NewLayer);
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
        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
            program.command_buffer.push(Command::DuplicateLayer);
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
        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
            program.command_buffer.push(Command::DeleteLayer);
            gl_area_clone.queue_render();
        }
    }));
    app.add_action(&delete_layer);
}

fn create_dialog(window: &ApplicationWindow, title: &str) -> gtk::Dialog {
    let dialog = gtk::DialogBuilder::new()
        .transient_for(window)
        .title(title)
        .resizable(false)
        .modal(true)
        .build();

    dialog.connect_delete_event(|_, _| {
        Inhibit(true)
    });

    dialog.content_area().set_spacing(8);
    dialog
}

fn parse_new_size(gtk_program: &GTKProgram, entry_width: &gtk::Entry, entry_height: &gtk::Entry) -> Option<(u32, u32)> {
    let parse_entry = |entry: &gtk::Entry, current: u32| {
        let text = entry.text();
        let text: &str = text.as_str();
        let mut chars = text.chars().collect::<Vec<_>>();
        if chars.last() == Some(&'%') {
            chars.remove(chars.len() - 1);
            let value = f32::from_str(&String::from_iter(chars)).ok()? / 100.0;
            Some((value * current as f32).round() as u32)
        } else {
            u32::from_str(text).ok()
        }
    };

    if let Some(program) = gtk_program.program.borrow_mut().as_mut() {
        match (parse_entry(entry_width, program.editor.image().width()),
               parse_entry(entry_height, program.editor.image().height())) {
            (Some(width), Some(height)) => {
                Some((width, height))
            }
            _ => None
        }
    } else {
        None
    }
}