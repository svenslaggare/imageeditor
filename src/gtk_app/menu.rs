use std::rc::Rc;
use std::str::FromStr;
use std::iter::FromIterator;
use std::ops::{ Deref};
use std::cell::RefCell;
use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{GLArea, gio, Application, ApplicationWindow, glib, FileChooserAction, ResponseType, Orientation};
use gtk::glib::translate::{from_glib_none};

use crate::gtk_app::{GTKProgram, GTKProgramRef};
use crate::gtk_app::helpers::{create_entry, create_file_dialog};
use crate::command_buffer::{Command, BackgroundType};
use crate::program::{ProgramAction, ProgramActionData};
use crate::editor::editor::ImageFormat;


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
    // New
    add_new_image_dialog(app, window, gtk_program.clone(), menu);

    // Open
    menu.append(Some("Open"), Some("app.open_file"));
    let open_file = gio::SimpleAction::new("open_file", None);

    let gl_area_clone = gl_area.clone();
    let open_file_dialog = create_file_dialog(
        window,
        gtk_program.clone(),
        "Open image",
        FileChooserAction::Open,
        move |gtk_program, path| {
            match image::open(&path) {
                Ok(image) => {
                    if let Some(program) = gtk_program.program.borrow_mut().as_mut() {
                        program.command_buffer.push(Command::SwitchImage(path, image.into_rgba()));
                        gl_area_clone.queue_render();
                    }
                }
                Err(err) => {
                    println!("Failed to open file due to: {:?}.", err);
                }
            }

            true
        }
    );

    let open_file_dialog_clone = open_file_dialog.clone();
    gtk_program.actions.borrow_mut().insert(
        ProgramAction::OpenImage,
        Box::new(move |_| {
            open_file_dialog_clone.show();
        })
    );

    let open_file_dialog_clone = open_file_dialog.clone();
    open_file.connect_activate(glib::clone!(@weak window => move |_, _| {
        open_file_dialog_clone.show();
    }));
    app.add_action(&open_file);

    // Save
    menu.append(Some("Save"), Some("app.save_file"));
    let save_file = gio::SimpleAction::new("save_file", None);
    let gtk_program_clone = gtk_program.clone();
    save_file.connect_activate(glib::clone!(@weak window => move |_, _| {
        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
            if let (Some(path), Some(image_format)) = (program.editor.image().path(), program.editor.image().image_format()) {
               if let Err(err) = program.editor.image().save(path, &image_format) {
                    println!("Failed to save file due to: {:?}.", err);
               }
            }
        }
    }));
    app.add_action(&save_file);

    // Save as
    add_save_as_dialog(app, window, gtk_program.clone(), menu);;

    // Quit
    menu.append(Some("Quit"), Some("app.quit"));
    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(glib::clone!(@weak window => move |_, _| {
        window.close();
    }));
    app.add_action(&quit);
}

fn add_new_image_dialog(app: &Application,
                        window: &ApplicationWindow,
                        gtk_program: GTKProgramRef,
                        menu: &gio::Menu) {
    menu.append(Some("New"), Some("app.new_image"));
    let new_image = gio::SimpleAction::new("new_image", None);

    let new_image_dialog = create_dialog(window, "New image");
    get_action_area(&new_image_dialog).set_property("halign", gtk::Align::Center).unwrap();

    new_image_dialog.add_buttons(&[
        ("Create", gtk::ResponseType::Ok),
        ("Cancel", gtk::ResponseType::Cancel)
    ]);

    let entry_width = create_entry(&new_image_dialog.content_area(), "Width: ", "1280");
    let entry_height = create_entry(&new_image_dialog.content_area(), "Height:", "800");

    let background_group = gtk::Box::new(gtk::Orientation::Vertical, 2);

    let background_label = gtk::LabelBuilder::new()
        .label("Background:")
        .build();

    background_label.set_xalign(0.0);
    background_group.add(&background_label);

    let background_transparent = gtk::RadioButtonBuilder::new()
        .label("Transparent")
        .build();
    background_group.add(&background_transparent);

    let background_white = gtk::RadioButtonBuilder::new()
        .label("White")
        .build();
    background_group.add(&background_white);
    background_white.join_group(Some(&background_transparent));
    new_image_dialog.content_area().add(&background_group);

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
                        let background = if background_transparent.is_active() {
                            BackgroundType::Transparent
                        } else if background_white.is_active() {
                            BackgroundType::Color(image::Rgba([255, 255, 255, 255]))
                        } else {
                            BackgroundType::Transparent
                        };

                        if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                            program.command_buffer.push(Command::NewImage(width, height, background));
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
}

fn add_save_as_dialog(app: &Application,
                      window: &ApplicationWindow,
                      gtk_program: GTKProgramRef,
                      menu: &gio::Menu) {
    menu.append(Some("Save as"), Some("app.save_file_as"));
    let save_file_as = gio::SimpleAction::new("save_file_as", None);

    // JPEG quality dialog
    let jpeg_quality_dialog = create_dialog(window, "JPEG Quality");
    jpeg_quality_dialog.content_area().set_spacing(4);
    jpeg_quality_dialog.set_width_request(200);

    jpeg_quality_dialog.add_buttons(&[
        ("Ok", gtk::ResponseType::Ok)
    ]);

    let jpeg_quality_label = gtk::Label::new(Some("Quality:"));
    jpeg_quality_label.set_xalign(0.0);
    jpeg_quality_dialog.content_area().add(&jpeg_quality_label);

    let jpeg_quality_scale = gtk::Scale::with_range(
        Orientation::Horizontal,
        1.0,
        100.0,
        1.0
    );
    jpeg_quality_dialog.content_area().add(&jpeg_quality_scale);

    get_action_area(&jpeg_quality_dialog).set_property("halign", gtk::Align::Center).unwrap();

    let current_save_path = Rc::new(RefCell::new(Option::<PathBuf>::None));

    let gtk_program_clone = gtk_program.clone();
    let current_save_path_clone = current_save_path.clone();
    let jpeg_quality_scale_clone = jpeg_quality_scale.clone();
    jpeg_quality_dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Ok => {
                if let Some(path) = current_save_path_clone.borrow_mut().clone() {
                    if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                        if let Err(err) = program.editor.image_mut().save_as(&path, &ImageFormat::Jpeg(jpeg_quality_scale_clone.value() as u8)) {
                            println!("Failed to save file due to: {:?}.", err);
                        }
                    }
                }

                dialog.hide();
            }
            _ => {
                dialog.hide();
            }
        }
    });

    // Main save as dialog
    let current_save_path_clone = current_save_path.clone();
    let save_file_as_dialog = create_file_dialog(
        window,
        gtk_program.clone(),
        "Save as",
        FileChooserAction::Save,
        move |gtk_program, path| {
            if let Some(program) = gtk_program.program.borrow_mut().as_mut() {
                *current_save_path_clone.borrow_mut() = Some(path.clone());

                let image_format = path
                    .extension()
                    .map(|ext| ext.to_str()).flatten()
                    .map(|extension| ImageFormat::from_extension(extension)).flatten();

                if let Some(image_format) = image_format {
                    match image_format {
                        ImageFormat::Jpeg(quality) => {
                            jpeg_quality_scale.set_value(quality as f64);
                            jpeg_quality_dialog.show_all();
                        }
                        image_format => {
                            if let Err(err) = program.editor.image_mut().save_as(&path, &image_format) {
                                println!("Failed to save file due to: {:?}.", err);
                            }
                        }
                    }
                } else {
                    return false;
                }
            }

            true
        }
    );

    let save_file_as_dialog_clone = save_file_as_dialog.clone();
    gtk_program.actions.borrow_mut().insert(
        ProgramAction::SaveImageAs,
        Box::new(move |_| {
            save_file_as_dialog_clone.show();
        })
    );

    let save_file_as_dialog_clone = save_file_as_dialog.clone();
    save_file_as.connect_activate(glib::clone!(@weak window => move |_, _| {
        save_file_as_dialog_clone.show();
    }));
    app.add_action(&save_file_as);
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

    get_action_area(&resize_image_dialog).set_property("halign", gtk::Align::Center).unwrap();

    resize_image_dialog.add_buttons(&[
        ("Ok", gtk::ResponseType::Ok),
        ("Cancel", gtk::ResponseType::Cancel),
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
        ProgramAction::ResizeImage,
        Box::new(move |requested_size| {
            if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                let (width, height) = match requested_size {
                    ProgramActionData::Size(width, height, _) => (width, height),
                    _ => (program.editor.image().width(), program.editor.image().height())
                };

                entry_width_clone.set_text(&format!("{}", width));
                entry_height_clone.set_text(&format!("{}", height));
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

    resize_canvas_dialog.set_width_request(200);
    get_action_area(&resize_canvas_dialog).set_property("halign", gtk::Align::Center).unwrap();

    resize_canvas_dialog.add_buttons(&[
        ("Ok", gtk::ResponseType::Ok),
        ("Cancel", gtk::ResponseType::Cancel),
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
            resize_canvas_dialog_clone.set_title("Resize canvas");
            resize_canvas_dialog_clone.show_all();
        }
    }));

    let gtk_program_clone = gtk_program.clone();
    let resize_canvas_dialog_clone = resize_canvas_dialog.clone();
    let entry_width_clone = entry_width.clone();
    let entry_height_clone = entry_height.clone();
    gtk_program.actions.borrow_mut().insert(
        ProgramAction::ResizeCanvas,
        Box::new(move |requested_size| {
            if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                let (width, height, message) = match requested_size {
                    ProgramActionData::Size(width, height, message) => (width, height, message),
                    _ => (program.editor.image().width(), program.editor.image().height(), None)
                };

                entry_width_clone.set_text(&format!("{}", width));
                entry_height_clone.set_text(&format!("{}", height));
                if let Some(message) = message {
                    resize_canvas_dialog_clone.set_title(&message);
                } else {
                    resize_canvas_dialog_clone.set_title("Resize canvas");
                }

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
                if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                    program.command_buffer.push(Command::AbortedResizeCanvas);
                }

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

fn get_action_area(dialog: &gtk::Dialog) -> gtk::Box {
    unsafe {
        from_glib_none(gtk::ffi::gtk_dialog_get_action_area(dialog.as_ptr()))
    }
}