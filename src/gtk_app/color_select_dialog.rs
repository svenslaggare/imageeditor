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

    let red_color_selector = Rc::new(create_spin_button(&color_selector_container, "Red:", 0.0, 255.0, 1.0));
    let green_color_selector = Rc::new(create_spin_button(&color_selector_container, "Green:", 0.0, 255.0, 1.0));
    let blue_color_selector = Rc::new(create_spin_button(&color_selector_container, "Blue:", 0.0, 255.0, 1.0));

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

    let color_code_entry = gtk::Entry::builder()
        .text("#000000")
        .build();

    current_color_box.add(&gtk::Label::new(Some("Color:")));
    current_color_box.add(&color_code_entry);

    fn generate_current_color(color_str: &str) -> String {
        format!("<span font='14' background='{}'>             </span>", color_str)
    }

    let current_color_view = gtk::Label::new(None);
    current_color_view.set_markup(&generate_current_color(color_code_entry.text().as_str()));
    current_color_box.add(&current_color_view);
    let current_color_view = Rc::new(current_color_view);

    color_selector_container.add(&current_color_box);
    let color_code_entry = Rc::new(color_code_entry);

    let suppress_color_code_change = Rc::new(RefCell::new(false));
    fn update_color_code(red_color_selector: &gtk::SpinButton,
                         green_color_selector: &gtk::SpinButton,
                         blue_color_selector: &gtk::SpinButton,
                         color_code_entry: &gtk::Entry,
                         current_color_view: &gtk::Label,
                         suppress_color_code_change: &RefCell<bool>) {
        let red = red_color_selector.value() as u8;
        let green = green_color_selector.value() as u8;
        let blue = blue_color_selector.value() as u8;
        *suppress_color_code_change.borrow_mut() = true;

        let color_str = format!("#{:02X}{:02X}{:02X}", red, green, blue);
        color_code_entry.set_text(&color_str);
        current_color_view.set_markup(&generate_current_color(&color_str));
    }

    let red_color_selector_clone = red_color_selector.clone();
    let green_color_selector_clone = green_color_selector.clone();
    let blue_color_selector_clone = blue_color_selector.clone();
    let color_code_entry_clone = color_code_entry.clone();
    let current_color_view_clone = current_color_view.clone();
    let suppress_color_code_change_clone = suppress_color_code_change.clone();
    red_color_selector.connect_changed(move |selector| {
        update_color_code(
            red_color_selector_clone.deref(),
            green_color_selector_clone.deref(),
            blue_color_selector_clone.deref(),
            color_code_entry_clone.deref(),
            current_color_view_clone.deref(),
            suppress_color_code_change_clone.deref()
        );
    });

    let red_color_selector_clone = red_color_selector.clone();
    let green_color_selector_clone = green_color_selector.clone();
    let blue_color_selector_clone = blue_color_selector.clone();
    let color_code_entry_clone = color_code_entry.clone();
    let current_color_view_clone = current_color_view.clone();
    let suppress_color_code_change_clone = suppress_color_code_change.clone();
    green_color_selector.connect_changed(move |selector| {
        update_color_code(
            red_color_selector_clone.deref(),
            green_color_selector_clone.deref(),
            blue_color_selector_clone.deref(),
            color_code_entry_clone.deref(),
            current_color_view_clone.deref(),
            suppress_color_code_change_clone.deref()
        );
    });

    let red_color_selector_clone = red_color_selector.clone();
    let green_color_selector_clone = green_color_selector.clone();
    let blue_color_selector_clone = blue_color_selector.clone();
    let color_code_entry_clone = color_code_entry.clone();
    let current_color_view_clone = current_color_view.clone();
    let suppress_color_code_change_clone = suppress_color_code_change.clone();
    blue_color_selector.connect_changed(move |selector| {
        update_color_code(
            red_color_selector_clone.deref(),
            green_color_selector_clone.deref(),
            blue_color_selector_clone.deref(),
            color_code_entry_clone.deref(),
            current_color_view_clone.deref(),
            suppress_color_code_change_clone.deref()
        );
    });

    let red_color_selector_clone = red_color_selector.clone();
    let green_color_selector_clone = green_color_selector.clone();
    let blue_color_selector_clone = blue_color_selector.clone();
    let suppress_color_code_change_clone = suppress_color_code_change.clone();
    color_code_entry.connect_changed(move |entry| {
        if *suppress_color_code_change_clone.borrow_mut() {
            *suppress_color_code_change_clone.borrow_mut() = false;
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
                    red_color_selector_clone.set_value(red as f64);
                    green_color_selector_clone.set_value(green as f64);
                    blue_color_selector_clone.set_value(blue as f64);
                }
                _ => {}
            }
        }
    });

    let gtk_program_clone = gtk_program.clone();
    let red_color_selector_clone = red_color_selector.clone();
    let green_color_selector_clone = green_color_selector.clone();
    let blue_color_selector_clone = blue_color_selector.clone();
    let opacity_scale_clone = opacity_scale.clone();
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
                red_color_selector_clone.set_value(color[0] as f64);
                green_color_selector_clone.set_value(color[1] as f64);
                blue_color_selector_clone.set_value(color[2] as f64);
                opacity_scale_clone.set_value(color[3] as f64);
            }

            dialog_clone.set_title("Select primary color");
            *color_select_mode_clone.borrow_mut() = SelectColorMode::PrimaryColor;
            dialog_clone.show_all();
        })
    );

    let gtk_program_clone = gtk_program.clone();
    let red_color_selector_clone = red_color_selector.clone();
    let green_color_selector_clone = green_color_selector.clone();
    let blue_color_selector_clone = blue_color_selector.clone();
    let opacity_scale_clone = opacity_scale.clone();
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
                red_color_selector_clone.set_value(color[0] as f64);
                green_color_selector_clone.set_value(color[1] as f64);
                blue_color_selector_clone.set_value(color[2] as f64);
                opacity_scale_clone.set_value(color[3] as f64);
            }

            dialog_clone.set_title("Select secondary color");
            *color_select_mode_clone.borrow_mut() = SelectColorMode::SecondaryColor;
            dialog_clone.show_all();
        })
    );

    let red_color_selector_clone = red_color_selector.clone();
    let green_color_selector_clone = green_color_selector.clone();
    let blue_color_selector_clone = blue_color_selector.clone();
    let opacity_scale_clone = opacity_scale.clone();
    dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Ok => {
                let red = red_color_selector_clone.value();
                let green = green_color_selector_clone.value();
                let blue = blue_color_selector_clone.value();

                if let Some(program) = gtk_program.program.borrow_mut().as_mut() {
                    let color = image::Rgba([
                        red as u8,
                        green as u8,
                        blue as u8,
                        opacity_scale_clone.value() as u8
                    ]);

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

    let red_color_selector_clone = red_color_selector.clone();
    let green_color_selector_clone = green_color_selector.clone();
    let blue_color_selector_clone = blue_color_selector.clone();
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
            let color = color_select_dialog.color_wheel.select_color(
                &editor_window,
                &event,
            );

            if let Some(color) = color {
                red_color_selector_clone.set_value(color[0] as f64);
                green_color_selector_clone.set_value(color[1] as f64);
                blue_color_selector_clone.set_value(color[2] as f64);
            }
        }

        color_select_dialog.color_wheel.render(&color_select_dialog.renders, &transform);

        Inhibit(true)
    });

    // dialog.show_all();
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