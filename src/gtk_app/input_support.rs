use std::collections::HashMap;
use std::iter::FromIterator;
use std::rc::Rc;
use std::cell::RefCell;

use gtk::{gdk, EventBox, GLArea};
use gtk::prelude::*;

use crate::gtk_app::GTKProgram;

pub fn add(gtk_program: Rc<RefCell<Option<GTKProgram>>>,
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

            if let Some((key, _)) = get_glfw_key(event.keyval(), event.state()) {
                match key {
                    glfw::Key::LeftShift | glfw::Key::RightShift => {
                        program.editor_window.shift_down = true;
                    }
                    _ => {}
                }
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

            if let Some((key, _)) = get_glfw_key(event.keyval(), event.state()) {
                match key {
                    glfw::Key::LeftShift | glfw::Key::RightShift => {
                        program.editor_window.shift_down = false;
                    }
                    _ => {}
                }
            }
        }

        gl_area_clone.queue_render();
        Inhibit(true)
    });
}

pub fn get_glfw_key(key: gdk::keys::Key, state: gdk::ModifierType) -> Option<(glfw::Key, glfw::Modifiers)> {
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

pub fn get_glfw_mouse_button(mouse_button: u32) -> glfw::MouseButton {
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
            (gdk::keys::constants::Shift_L, glfw::Key::LeftShift),
            (gdk::keys::constants::Shift_R, glfw::Key::RightShift),
        ].into_iter()
    );
}