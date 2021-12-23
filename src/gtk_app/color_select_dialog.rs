use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Deref;
use itertools::Itertools;
use std::iter::FromIterator;
use std::collections::VecDeque;

use gtk::{Application, ApplicationWindow, GLArea, Orientation, Align, EventBox, gdk, ResponseType};

use gtk::prelude::*;

use crate::gtk_app::GTKProgramRef;
use crate::gtk_app::helpers::{create_dialog, create_spin_button, create_entry};
use crate::ui::color_wheel::ColorWheel;
use crate::ui::button::GenericButton;
use crate::program::{Renders, ProgramAction, ProgramActionData};
use crate::gtk_app::input_support::get_glfw_mouse_button;
use crate::editor::tools::{EditorWindow, SelectColorMode};
use crate::command_buffer::{CommandBuffer, Command};
use crate::editor;

pub fn add(_app: &Application,
           window: &ApplicationWindow,
           gtk_program: GTKProgramRef) {
    let color_select_dialog = Rc::new(RefCell::new(None));

    let dialog = Rc::new(create_dialog(window, "Select primary color"));
    dialog.set_width_request(400);
    dialog.set_height_request(200);

    let color_select_mode = Rc::new(RefCell::new(SelectColorMode::PrimaryColor));

    dialog.add_buttons(&[
        ("Ok", gtk::ResponseType::Ok),
        ("Cancel", gtk::ResponseType::Cancel)
    ]);

    let container = gtk::Box::new(Orientation::Horizontal, 10);
    dialog.content_area().add(&container);

    let gl_area = GLArea::new();
    gl_area.set_width_request(200);
    gl_area.set_height_request(200);

    let event_box = Rc::new(EventBox::new());
    event_box.add(&gl_area);
    container.add(event_box.deref());

    let color_selector_container = gtk::Box::new(Orientation::Vertical, 6);
    container.add(&color_selector_container);

    let mut color_selector = ColorSelector::new(&color_selector_container);

    color_selector_container.add(&gtk::Separator::new(Orientation::Horizontal));

    let opacity_box = gtk::Box::new(Orientation::Horizontal, 4);
    color_selector_container.add(&opacity_box);

    let opacity_label = gtk::Label::new(Some("Opacity:"));
    opacity_box.add(&opacity_label);

    let opacity_scale = gtk::Scale::with_range(
        Orientation::Horizontal,
        0.0,
        255.0,
        1.0
    );
    opacity_scale.set_width_request(200);
    opacity_scale.set_value(255.0);
    opacity_box.add(&opacity_scale);

    let current_color_box = gtk::Box::new(Orientation::Horizontal, 5);

    let color_code = gtk::Entry::builder()
        .text("#000000")
        .build();

    current_color_box.add(&gtk::Label::new(Some("Color:")));
    current_color_box.add(&color_code);

    let current_color_view = gtk::Label::new(None);
    current_color_view.set_markup(&generate_current_color(color_code.text().as_str()));
    current_color_box.add(&current_color_view);

    color_selector_container.add(&current_color_box);

    color_selector.initialize(opacity_scale, color_code, current_color_view);
    let color_selector = Rc::new(color_selector);

    let color_selector_clone = color_selector.clone();
    color_selector.red_selector.connect_changed(move |selector| color_selector_clone.update());

    let color_selector_clone = color_selector.clone();
    color_selector.blue_selector.connect_changed(move |selector| color_selector_clone.update());

    let color_selector_clone = color_selector.clone();
    color_selector.blue_selector.connect_changed(move |selector| color_selector_clone.update());

    let color_selector_clone = color_selector.clone();
    color_selector.color_code.as_ref().unwrap().connect_changed(move |entry| {
        if color_selector_clone.is_color_code_change_suppressed() {
            return;
        }

        if entry.text().starts_with("#") {
            let chars = entry.text().chars().skip(1).collect::<Vec<_>>();
            let mut parts = chars.chunks(2);
            let red = parts.next().map(|chars| String::from_iter(chars)).map(|str| u8::from_str_radix(&str, 16).ok()).flatten();
            let green = parts.next().map(|chars| String::from_iter(chars)).map(|str| u8::from_str_radix(&str, 16).ok()).flatten();
            let blue = parts.next().map(|chars| String::from_iter(chars)).map(|str| u8::from_str_radix(&str, 16).ok()).flatten();

            match (red, green, blue) {
                (Some(red), Some(green), Some(blue)) => {
                    color_selector_clone.set_rgb(red, green, blue);
                }
                _ => {}
            }
        }
    });

    let gtk_program_clone = gtk_program.clone();
    let color_selector_clone = color_selector.clone();
    let dialog_clone = dialog.clone();
    let color_select_mode_clone = color_select_mode.clone();
    gtk_program.actions.borrow_mut().insert(
        ProgramAction::OpenSelectPrimaryColorDialog,
        Box::new(move |_| {
            if dialog_clone.is_visible() {
                return;
            }

            if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                let color = program.primary_color();
                color_selector_clone.set_rgba(color[0], color[1], color[2], color[3]);
            }

            dialog_clone.set_title("Select primary color");
            *color_select_mode_clone.borrow_mut() = SelectColorMode::PrimaryColor;
            dialog_clone.show_all();
        })
    );

    let gtk_program_clone = gtk_program.clone();
    let color_selector_clone = color_selector.clone();
    let dialog_clone = dialog.clone();
    let color_select_mode_clone = color_select_mode.clone();
    gtk_program.actions.borrow_mut().insert(
        ProgramAction::OpenSelectSecondaryColorDialog,
        Box::new(move |_| {
            if dialog_clone.is_visible() {
                return;
            }

            if let Some(program) = gtk_program_clone.program.borrow_mut().as_mut() {
                let color = program.secondary_color();
                color_selector_clone.set_rgba(color[0], color[1], color[2], color[3]);
            }

            dialog_clone.set_title("Select secondary color");
            *color_select_mode_clone.borrow_mut() = SelectColorMode::SecondaryColor;
            dialog_clone.show_all();
        })
    );

    let color_selector_clone = color_selector.clone();
    dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Ok => {
                if let Some(program) = gtk_program.program.borrow_mut().as_mut() {
                    let color = color_selector_clone.selected_color();
                    match *color_select_mode.borrow_mut() {
                        SelectColorMode::PrimaryColor => {
                            program.command_buffer.push(Command::SetPrimaryColor(color));
                        }
                        SelectColorMode::SecondaryColor => {
                            program.command_buffer.push(Command::SetSecondaryColor(color));
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

    let color_select_dialog_clone = color_select_dialog.clone();
    gl_area.connect_realize(move |area| {
        area.context().unwrap().make_current();
        *color_select_dialog_clone.borrow_mut() = Some(ColorSelectDialog::new());
    });

    event_box.add_events(gdk::EventMask::POINTER_MOTION_MASK | gdk::EventMask::SCROLL_MASK);

    event_box.set_can_focus(true);
    event_box.grab_focus();

    let gl_area_clone = gl_area.clone();
    let event_box_clone = event_box.clone();
    let color_select_dialog_clone = color_select_dialog.clone();
    event_box.connect_button_press_event(move |_, event| {
        event_box_clone.grab_focus();

        if let Some(color_select_dialog) = color_select_dialog_clone.borrow_mut().as_mut() {
            color_select_dialog.mouse_position = event.coords().unwrap();
            color_select_dialog.event_queue.push_back(glfw::WindowEvent::MouseButton(
                get_glfw_mouse_button(event.button()),
                glfw::Action::Press,
                glfw::Modifiers::empty()
            ));
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });

    let gl_area_clone = gl_area.clone();
    let color_select_dialog_clone = color_select_dialog.clone();
    event_box.connect_button_release_event(move |_, event| {
        if let Some(color_select_dialog) = color_select_dialog_clone.borrow_mut().as_mut() {
            color_select_dialog.mouse_position = event.coords().unwrap();
            color_select_dialog.event_queue.push_back(glfw::WindowEvent::MouseButton(
                get_glfw_mouse_button(event.button()),
                glfw::Action::Release,
                glfw::Modifiers::empty()
            ));
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });

    let gl_area_clone = gl_area.clone();
    let color_select_dialog_clone = color_select_dialog.clone();
    event_box.connect_motion_notify_event(move |_, event| {
        let mouse_position = event.coords().unwrap();

        if let Some(color_select_dialog) = color_select_dialog_clone.borrow_mut().as_mut() {
            color_select_dialog.mouse_position = event.coords().unwrap();
            color_select_dialog.event_queue.push_back(glfw::WindowEvent::CursorPos(
                mouse_position.0,
                mouse_position.1
            ));
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });

    let color_selector_clone = color_selector.clone();
    gl_area.connect_render(move |area, context| {
        context.make_current();

        unsafe {
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let transform = cgmath::ortho(
            0.0,
            area.width_request() as f32,
            area.height_request() as f32,
            0.0,
            0.0,
            1.0
        );

        let mut color_select_dialog = color_select_dialog.borrow_mut();
        let color_select_dialog = color_select_dialog.as_mut().unwrap();

        let events = std::mem::take(&mut color_select_dialog.event_queue);
        let editor_window = EditorWindowFixed::new(color_select_dialog.mouse_position, 200, 200, false);
        for event in events.into_iter() {
            let color = color_select_dialog.color_wheel.select_color(&editor_window, &event);
            if let Some(color) = color {
                color_selector_clone.set_rgb(color[0], color[1], color[2]);
            }
        }

        color_select_dialog.color_wheel.render(&color_select_dialog.renders, &transform);

        Inhibit(true)
    });
}

struct ColorSelectDialog {
    renders: Renders,
    color_wheel: ColorWheel,
    mouse_position: (f64, f64),
    event_queue: VecDeque<glfw::WindowEvent>
}

impl ColorSelectDialog {
    pub fn new() -> ColorSelectDialog {
        ColorSelectDialog {
            renders: Renders::new(),
            color_wheel: ColorWheel::new(),
            mouse_position: (0.0, 0.0),
            event_queue: VecDeque::new()
        }
    }
}

struct ColorSelector {
    red_selector: gtk::SpinButton,
    green_selector: gtk::SpinButton,
    blue_selector: gtk::SpinButton,
    opacity_selector: Option<gtk::Scale>,
    color_code: Option<gtk::Entry>,
    current_color_view: Option<gtk::Label>,
    suppress_color_code_change: RefCell<bool>,
}

impl ColorSelector {
    pub fn new(container: &gtk::Box) -> ColorSelector {
        ColorSelector {
            red_selector: create_spin_button(&container, "Red:", 0.0, 0.0, 255.0, 1.0),
            green_selector: create_spin_button(&container, "Green:", 0.0, 0.0, 255.0, 1.0),
            blue_selector: create_spin_button(&container, "Blue:", 0.0, 0.0, 255.0, 1.0),
            opacity_selector: None,
            color_code: None,
            current_color_view: None,
            suppress_color_code_change: RefCell::new(false)
        }
    }

    pub fn initialize(&mut self,
                      opacity_selector: gtk::Scale,
                      color_code: gtk::Entry,
                      current_color_view: gtk::Label) {
        self.opacity_selector = Some(opacity_selector);
        self.color_code = Some(color_code);
        self.current_color_view = Some(current_color_view);
    }

    pub fn opacity_selector(&self) -> &gtk::Scale {
        self.opacity_selector.as_ref().unwrap()
    }

    pub fn color_code(&self) -> &gtk::Entry {
        self.color_code.as_ref().unwrap()
    }

    pub fn current_color_view(&self) -> &gtk::Label {
        self.current_color_view.as_ref().unwrap()
    }

    pub fn update(&self) {
        let red = self.red_selector.value() as u8;
        let green = self.green_selector.value() as u8;
        let blue = self.blue_selector.value() as u8;
        *self.suppress_color_code_change.borrow_mut() = true;

        let color_str = format!("#{:02X}{:02X}{:02X}", red, green, blue);
        self.color_code.as_ref().unwrap().set_text(&color_str);
        self.current_color_view.as_ref().unwrap().set_markup(&generate_current_color(&color_str));
    }

    pub fn selected_color(&self) -> editor::Color {
        let red = self.red_selector.value();
        let green = self.green_selector.value();
        let blue = self.blue_selector.value();

        image::Rgba([
            red as u8,
            green as u8,
            blue as u8,
            self.opacity_selector().value() as u8
        ])
    }

    pub fn set_rgb(&self, red: u8, green: u8, blue: u8) {
        self.red_selector.set_value(red as f64);
        self.green_selector.set_value(green as f64);
        self.blue_selector.set_value(blue as f64);
    }

    pub fn set_rgba(&self, red: u8, green: u8, blue: u8, alpha: u8) {
        self.set_rgb(red, green, blue);
        self.opacity_selector().set_value(alpha as f64);
    }

    pub fn is_color_code_change_suppressed(&self) -> bool {
        if *self.suppress_color_code_change.borrow_mut() {
            *self.suppress_color_code_change.borrow_mut() = false;
            true
        } else {
            false
        }
    }
}

fn generate_current_color(color_str: &str) -> String {
    format!("<span font='14' background='{}'>             </span>", color_str)
}

struct EditorWindowFixed {
    mouse_position: (f64, f64),
    width: u32,
    height: u32,
    is_shift_down: bool
}

impl EditorWindowFixed {
    pub fn new(mouse_position: (f64, f64),
               width: u32,
               height: u32,
               is_shift_down: bool) -> EditorWindowFixed {
        EditorWindowFixed {
            mouse_position,
            width,
            height,
            is_shift_down
        }
    }
}

impl EditorWindow for EditorWindowFixed {
    fn get_cursor_pos(&self) -> (f64, f64) {
        self.mouse_position
    }

    fn is_shift_down(&self) -> bool {
        self.is_shift_down
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}