use std::rc::Rc;
use std::path::PathBuf;
use std::ops::{Deref};

use gtk::prelude::*;
use gtk::{FileChooserAction, ApplicationWindow, Inhibit, Orientation, ResponseType};

use crate::gtk_app::{GTKProgram, GTKProgramRef};

pub fn create_file_dialog<F: Fn(&GTKProgram, PathBuf) -> bool + 'static>(window: &ApplicationWindow,
                                                                         gtk_program: GTKProgramRef,
                                                                         title: &str,
                                                                         action: FileChooserAction,
                                                                         on_file: F) -> Rc<gtk::FileChooserDialog> {
    let file_dialog = gtk::FileChooserDialogBuilder::new()
        .transient_for(window)
        .title(title)
        .modal(true)
        .action(action)
        .build();

    let action_name = match action {
        FileChooserAction::Open => "Open",
        FileChooserAction::Save => "Save",
        FileChooserAction::SelectFolder => "Select",
        FileChooserAction::CreateFolder => "Create",
        _ => "Unknown"
    };

    file_dialog.add_buttons(&[
        (action_name, gtk::ResponseType::Ok),
        ("Cancel", gtk::ResponseType::Cancel),
    ]);

    let file_dialog = Rc::new(file_dialog);

    file_dialog.connect_delete_event(|_, _| {
        Inhibit(true)
    });

    let file_dialog_clone = file_dialog.clone();
    let gtk_program_clone = gtk_program.clone();
    file_dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Ok => {
                let hide = if let Some(path) = file_dialog_clone.filename() {
                    on_file(gtk_program_clone.deref(), path)
                } else {
                    true
                };

                if hide {
                    dialog.hide();
                }
            }
            _ => {
                dialog.hide();
            }
        }
    });

    file_dialog
}

pub fn create_entry(container: &gtk::Box, label: &str, default_value: &str) -> gtk::Entry {
    let box_widget = gtk::Box::new(Orientation::Horizontal, 5);

    let entry_widget = gtk::Entry::builder()
        .text(default_value)
        .build();

    let label_widget = gtk::Label::builder()
        .label(label)
        .build();

    box_widget.add(&label_widget);
    box_widget.add(&entry_widget);

    container.add(&box_widget);
    entry_widget
}