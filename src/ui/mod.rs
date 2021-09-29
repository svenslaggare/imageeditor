use std::cell::RefCell;
use std::rc::Rc;

use itertools::Itertools;

pub mod manager;
pub mod button;
pub mod layout;

pub use manager::Manager;
pub use button::Button;

use crate::command_buffer::{Command};
use crate::rendering::prelude::Position;
use crate::editor::draw_tools::DrawTools;
use crate::editor::image_operation_helpers::hsv_to_rgb;
use crate::ui::button::TextButton;
use crate::rendering::font::{FontRef, Font};

pub fn create() -> Manager {
    let mut buttons = Vec::new();
    let mut text_buttons = Vec::new();

    // buttons.append(&mut generate_draw_tools());
    text_buttons.append(&mut generate_draw_tools2());
    buttons.append(&mut generate_color_palette());

    Manager::new(buttons, text_buttons)
}

fn generate_draw_tools() -> Vec<Button> {
    let mut buttons = Vec::new();
    let mut layout = layout::adaptive_rows(
        Position::new(0.0, 0.0),
        (40.0, 40.0),
        40.0,
        6
    );

    buttons.push(Button::new(
        &image::open("content/ui/pencil.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Pencil));
        })),
        None
    ));

    buttons.push(Button::new(
        &image::open("content/ui/line.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Line));
        })),
        None
    ));

    buttons.push(Button::new(
        &image::open("content/ui/rectangle.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Rectangle));
        })),
        None
    ));

    buttons.push(Button::new(
        &image::open("content/ui/circle.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Circle));
        })),
        None
    ));

    buttons.push(Button::new(
        &image::open("content/ui/select.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Selection));
        })),
        None
    ));

    buttons.push(Button::new(
        &image::open("content/ui/effect.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Effect));
        })),
        None
    ));

    buttons
}

fn generate_draw_tools2() -> Vec<TextButton> {
    let font = Rc::new(RefCell::new(Font::new("content/fonts/NotoMono-Regular.ttf", 24).unwrap()));
    let line_height = font.borrow_mut().line_height();

    let mut buttons = Vec::new();
    let mut layout = layout::adaptive_rows(
        Position::new(5.0, 5.0),
        (40.0, line_height + 5.0),
        40.0,
        6
    );

    buttons.push(TextButton::new(
        font.clone(),
        "P".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Pencil));
        })),
        None
    ));

    buttons.push(TextButton::new(
        font.clone(),
        "L".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Line));
        })),
        None
    ));

    buttons.push(TextButton::new(
        font.clone(),
        "R".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Rectangle));
        })),
        None
    ));

    buttons.push(TextButton::new(
        font.clone(),
        "C".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Circle));
        })),
        None
    ));

    buttons.push(TextButton::new(
        font.clone(),
        "S".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Selection));
        })),
        None
    ));

    buttons.push(TextButton::new(
        font.clone(),
        "E".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Effect));
        })),
        None
    ));

    buttons
}

fn generate_color_palette() -> Vec<Button> {
    let mut buttons = Vec::new();

    let mut colors = Vec::new();
    colors.push(image::Rgba([255, 255, 255, 255]));
    colors.push(image::Rgba([0, 0, 0, 255]));
    colors.push(image::Rgba([160, 160, 160, 255]));
    for h in (0..360).step_by(20) {
        if let Some(color) = hsv_to_rgb(h as f64, 100.0, 50.0) {
            colors.push(color);
        }

        if let Some(color) = hsv_to_rgb(h as f64, 100.0, 100.0) {
            colors.push(color);
        }

        if let Some(color) = hsv_to_rgb(h as f64, 50.0, 100.0) {
            colors.push(color);
        }
    }

    let cell_size = (16, 16);

    let layout = layout::adaptive_rows(
        Position::new(0.0, 240.0),
        (cell_size.0 as f32, cell_size.1 as f32),
        48.0,
        colors.len()
    );

    for (color, position) in colors.iter().zip_eq(layout) {
        let mut image = image::RgbaImage::new(cell_size.0, cell_size.1);
        // let color = image::Rgba([*color[0], *color[1], *color[2], 255]);
        let color = *color;

        for pixel in image.pixels_mut() {
            *pixel = color;
        }

        buttons.push(Button::new(
            &image,
            position,
            Some(Box::new(move |command_buffer| {
                command_buffer.push(Command::SetColor(color));
            })),
            Some(Box::new(move |command_buffer| {
                command_buffer.push(Command::SetAlternativeColor(color));
            }))
        ));
    }

    buttons
}