use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, GLArea, Box, Orientation, EventBox, gdk, glib, Menu, AccelGroup, MenuBar, MenuItem, Image, Label, CheckMenuItem, IconSize, AccelFlags, gio, FileChooserAction, ResponseType};
use gtk::gio::ApplicationFlags;
use gtk::gdk::keys::Key;
use gtk::gdk::ScrollDirection;

use crate::program::{Program, LEFT_SIDE_PANEL_WIDTH, TOP_PANEL_HEIGHT, SIDE_PANELS_WIDTH};
use crate::editor::tools::EditorWindow;
use crate::{ui, editor};
use crate::command_buffer::Command;
use std::str::FromStr;

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

        add_menu(app, &window, gtk_program.clone(), gl_area.clone());
        add_input_support(gtk_program.clone(), gl_area.clone(), event_box.clone());

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

            gtk_program.program.render(&transform);

            Inhibit(true)
        });

        window.show_all();
    });

    application.run();
}

struct GTKProgram {
    program: Program,
    editor_window: GTKEditorWindow,
    event_queue: VecDeque<glfw::WindowEvent>
}

impl GTKProgram {
    pub fn new(view_width: u32, view_height: u32, image_to_edit: image::RgbaImage) -> GTKProgram {
        let mut program = Program::new(
            view_width,
            view_height,
            editor::Editor::new(editor::Image::new(image_to_edit)),
            ui::create(),
        );

        GTKProgram {
            program,
            editor_window: GTKEditorWindow {
                mouse_position: (0.0, 0.0),
                width: view_width,
                height: view_height
            },
            event_queue: VecDeque::new()
        }
    }

    pub fn change_size(&mut self, width: u32, height: u32) {
        self.event_queue.push_back(glfw::WindowEvent::FramebufferSize(width as i32, height as i32));
        self.editor_window.width = width;
        self.editor_window.height = height;
    }
}

struct GTKEditorWindow {
    mouse_position: (f64, f64),
    width: u32,
    height: u32
}

impl EditorWindow for GTKEditorWindow {
    fn get_cursor_pos(&self) -> (f64, f64) {
        self.mouse_position
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn set_should_close(&mut self, _value: bool) {

    }
}

fn add_menu(app: &Application,
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

fn add_input_support(gtk_program: Rc<RefCell<Option<GTKProgram>>>,
                     gl_area: Rc<GLArea>,
                     event_box: Rc<EventBox>) {
    event_box.add_events(
        gdk::EventMask::KEY_PRESS_MASK
            | gdk::EventMask::KEY_RELEASE_MASK
            | gdk::EventMask::POINTER_MOTION_MASK
            | gdk::EventMask::SCROLL_MASK
    );

    event_box.set_can_focus(true);
    event_box.grab_focus();

    let gl_area_clone = gl_area.clone();
    let event_box_clone = event_box.clone();
    let gtk_program_clone = gtk_program.clone();
    event_box.connect_button_press_event(move |_, event| {
        event_box_clone.grab_focus();

        if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
            program.editor_window.mouse_position = event.coords().unwrap();

            program.event_queue.push_back(glfw::WindowEvent::MouseButton(
                get_glfw_mouse_button(event.button()),
                glfw::Action::Press,
                glfw::Modifiers::empty()
            ));
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });

    let gl_area_clone = gl_area.clone();
    let gtk_program_clone = gtk_program.clone();
    event_box.connect_button_release_event(move |_, event| {
        if let Some(program) = gtk_program_clone.borrow_mut().as_mut() {
            program.editor_window.mouse_position = event.coords().unwrap();

            program.event_queue.push_back(glfw::WindowEvent::MouseButton(
                get_glfw_mouse_button(event.button()),
                glfw::Action::Release,
                glfw::Modifiers::empty()
            ));
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });

    let gl_area_clone = gl_area.clone();
    let program_clone = gtk_program.clone();
    event_box.connect_motion_notify_event(move |_, event| {
        if let Some(program) = program_clone.borrow_mut().as_mut() {
            let mouse_position = event.coords().unwrap();
            program.editor_window.mouse_position = mouse_position;

            program.event_queue.push_back(glfw::WindowEvent::CursorPos(
                mouse_position.0,
                mouse_position.1
            ));
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });

    let gl_area_clone = gl_area.clone();
    let program_clone = gtk_program.clone();
    event_box.connect_scroll_event(move |_, event| {
        if let Some(program) = program_clone.borrow_mut().as_mut() {
            match event.scroll_direction() {
                Some(gdk::ScrollDirection::Down) => {
                    program.event_queue.push_back(glfw::WindowEvent::Scroll(0.0, -1.0));
                }
                Some(gdk::ScrollDirection::Up) => {
                    program.event_queue.push_back(glfw::WindowEvent::Scroll(0.0, 1.0));
                }
                Some(gdk::ScrollDirection::Right) => {
                    program.event_queue.push_back(glfw::WindowEvent::Scroll(1.0, 0.0));
                }
                Some(gdk::ScrollDirection::Left) => {
                    program.event_queue.push_back(glfw::WindowEvent::Scroll(-1.0, 0.0));
                }
                _ => {}
            }
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });

    let gl_area_clone = gl_area.clone();
    let program_clone = gtk_program.clone();
    event_box.connect_key_press_event(move |_, event| {
        if let Some(program) = program_clone.borrow_mut().as_mut() {
            if let Some((key, modifier)) = get_glfw_key(event.keyval(), event.state()) {
                program.event_queue.push_back(glfw::WindowEvent::Key(
                    key,
                    0,
                    glfw::Action::Press,
                    modifier
                ));
            }
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });

    let gl_area_clone = gl_area.clone();
    let program_clone = gtk_program.clone();
    event_box.connect_key_release_event(move |_, event| {
        if let Some(program) = program_clone.borrow_mut().as_mut() {
            if let Some((key, modifier)) = get_glfw_key(event.keyval(), event.state()) {
                program.event_queue.push_back(glfw::WindowEvent::Key(
                    key,
                    0,
                    glfw::Action::Release,
                    modifier
                ));
            }
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });
}

fn create_file_dialog<F: Fn(&mut GTKProgram, PathBuf) + 'static>(window: &ApplicationWindow,
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

fn create_entry(container: &gtk::Box, label: &str, default_value: &str) -> gtk::Entry {
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

fn get_glfw_key(key: gdk::keys::Key, state: gdk::ModifierType) -> Option<(glfw::Key, glfw::Modifiers)> {
    if let Some(key) = KEYS_MAPPING.get(&key).cloned() {
        let mut modifiers = glfw::Modifiers::empty();
        if (state & gdk::ModifierType::SHIFT_MASK) == gdk::ModifierType::SHIFT_MASK {
            modifiers |= glfw::Modifiers::Shift;
        }

        if (state & gdk::ModifierType::CONTROL_MASK) == gdk::ModifierType::CONTROL_MASK {
            modifiers |= glfw::Modifiers::Control;
        }

        Some((key, modifiers))
    } else {
        None
    }
}

fn get_glfw_mouse_button(mouse_button: u32) -> glfw::MouseButton {
    match mouse_button {
        1 => glfw::MouseButton::Button1,
        2 => glfw::MouseButton::Button3,
        3 => glfw::MouseButton::Button2,
        _ => panic!("Unsupported")
    }
}

lazy_static::lazy_static! {
    static ref KEYS_MAPPING: HashMap<gdk::keys::Key, glfw::Key> = HashMap::from_iter(
        vec![
            (gdk::keys::constants::a, glfw::Key::A), (gdk::keys::constants::A, glfw::Key::A),
            (gdk::keys::constants::b, glfw::Key::B), (gdk::keys::constants::B, glfw::Key::B),
            (gdk::keys::constants::c, glfw::Key::C), (gdk::keys::constants::C, glfw::Key::C),
            (gdk::keys::constants::d, glfw::Key::D), (gdk::keys::constants::D, glfw::Key::D),
            (gdk::keys::constants::e, glfw::Key::E), (gdk::keys::constants::E, glfw::Key::E),
            (gdk::keys::constants::f, glfw::Key::F), (gdk::keys::constants::F, glfw::Key::F),
            (gdk::keys::constants::g, glfw::Key::G), (gdk::keys::constants::G, glfw::Key::G),
            (gdk::keys::constants::h, glfw::Key::H), (gdk::keys::constants::H, glfw::Key::H),
            (gdk::keys::constants::i, glfw::Key::I), (gdk::keys::constants::I, glfw::Key::I),
            (gdk::keys::constants::j, glfw::Key::J), (gdk::keys::constants::J, glfw::Key::J),
            (gdk::keys::constants::k, glfw::Key::K), (gdk::keys::constants::K, glfw::Key::K),
            (gdk::keys::constants::l, glfw::Key::L), (gdk::keys::constants::L, glfw::Key::L),
            (gdk::keys::constants::m, glfw::Key::M), (gdk::keys::constants::M, glfw::Key::M),
            (gdk::keys::constants::n, glfw::Key::N), (gdk::keys::constants::N, glfw::Key::N),
            (gdk::keys::constants::o, glfw::Key::O), (gdk::keys::constants::O, glfw::Key::O),
            (gdk::keys::constants::p, glfw::Key::P), (gdk::keys::constants::P, glfw::Key::P),
            (gdk::keys::constants::q, glfw::Key::Q), (gdk::keys::constants::Q, glfw::Key::Q),
            (gdk::keys::constants::r, glfw::Key::R), (gdk::keys::constants::R, glfw::Key::R),
            (gdk::keys::constants::s, glfw::Key::S), (gdk::keys::constants::S, glfw::Key::S),
            (gdk::keys::constants::t, glfw::Key::T), (gdk::keys::constants::T, glfw::Key::T),
            (gdk::keys::constants::u, glfw::Key::U), (gdk::keys::constants::U, glfw::Key::U),
            (gdk::keys::constants::v, glfw::Key::V), (gdk::keys::constants::V, glfw::Key::V),
            (gdk::keys::constants::x, glfw::Key::X), (gdk::keys::constants::X, glfw::Key::X),
            (gdk::keys::constants::y, glfw::Key::Y), (gdk::keys::constants::Y, glfw::Key::Y),
            (gdk::keys::constants::z, glfw::Key::Z), (gdk::keys::constants::X, glfw::Key::Z),
            (gdk::keys::constants::_0, glfw::Key::Num0),
            (gdk::keys::constants::_1, glfw::Key::Num1),
            (gdk::keys::constants::_2, glfw::Key::Num2),
            (gdk::keys::constants::_3, glfw::Key::Num3),
            (gdk::keys::constants::_4, glfw::Key::Num4),
            (gdk::keys::constants::_5, glfw::Key::Num5),
            (gdk::keys::constants::_6, glfw::Key::Num6),
            (gdk::keys::constants::_7, glfw::Key::Num7),
            (gdk::keys::constants::_8, glfw::Key::Num8),
            (gdk::keys::constants::_9, glfw::Key::Num9),
            (gdk::keys::constants::Left, glfw::Key::Left),
            (gdk::keys::constants::Right, glfw::Key::Right),
            (gdk::keys::constants::Up, glfw::Key::Up),
            (gdk::keys::constants::Down, glfw::Key::Down),
            (gdk::keys::constants::F1, glfw::Key::F1),
            (gdk::keys::constants::F2, glfw::Key::F2),
            (gdk::keys::constants::F3, glfw::Key::F3),
            (gdk::keys::constants::F4, glfw::Key::F4),
            (gdk::keys::constants::F5, glfw::Key::F5),
            (gdk::keys::constants::F6, glfw::Key::F6),
            (gdk::keys::constants::F7, glfw::Key::F7),
            (gdk::keys::constants::F8, glfw::Key::F8),
            (gdk::keys::constants::F9, glfw::Key::F9),
            (gdk::keys::constants::F10, glfw::Key::F10),
            (gdk::keys::constants::F11, glfw::Key::F11),
            (gdk::keys::constants::F12, glfw::Key::F12),
            (gdk::keys::constants::Return, glfw::Key::Enter),
            (gdk::keys::constants::Escape, glfw::Key::Escape),
            (gdk::keys::constants::space, glfw::Key::Space),
            (gdk::keys::constants::Delete, glfw::Key::Delete),
            (gdk::keys::constants::BackSpace, glfw::Key::Backspace),
            (gdk::keys::constants::Insert, glfw::Key::Insert),
            (gdk::keys::constants::Home, glfw::Key::Home),
            (gdk::keys::constants::End, glfw::Key::End),
            (gdk::keys::constants::Page_Up, glfw::Key::PageUp),
            (gdk::keys::constants::Page_Down, glfw::Key::PageDown),
        ].into_iter()
    );
}