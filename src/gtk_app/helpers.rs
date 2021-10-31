use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{FileChooserAction, ApplicationWindow, Inhibit, Orientation, ResponseType};

use crate::gtk_app::GTKProgram;

pub fn create_file_dialog<F: Fn(&mut GTKProgram, PathBuf) + 'static>(window: &ApplicationWindow,
                                                                     gtk_program: Rc<RefCell<Option<GTKProgram>>>,
                                                                     action: FileChooserAction,
                                                                     on_file: F) -> Rc<gtk::FileChooserDialog> {
    let file_dialog = gtk::FileChooserDialogBuilder::new()
        .transient_for(window)
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

    file_dialog.connect_delete_event(|dialog, event| {
        Inhibit(true)
    });

    let file_dialog_clone = file_dialog.clone();
    let gtk_program_clone = gtk_program.clone();
    file_dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Ok => {
                if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
                    if let Some(filename) = file_dialog_clone.filename() {
                        on_file(program, filename);
                    }
                }

                dialog.hide();
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