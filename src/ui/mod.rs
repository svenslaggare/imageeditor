use std::cell::RefCell;
use std::rc::Rc;

use itertools::Itertools;

pub mod manager;
pub mod button;
pub mod layout;

pub use manager::Manager;
pub use button::TextureButton;

use crate::command_buffer::{Command, CommandBuffer};
use crate::rendering::prelude::{Position, Rectangle};
use crate::editor::tools::{Tools, SelectionSubTool};
use crate::editor::image_operation_helpers::hsv_to_rgb;
use crate::ui::button::{TextButton, SolidColorButton};
use crate::rendering::font::{Font};

pub fn create() -> Manager {
    let mut texture_buttons = Vec::new();
    let mut solid_color_buttons = Vec::new();
    let mut text_buttons = Vec::new();

    generate_draw_tools(&mut text_buttons);
    generate_color_palette(&mut texture_buttons, &mut solid_color_buttons);

    Manager::new(texture_buttons, solid_color_buttons, text_buttons)
}

fn generate_draw_tools(texture_buttons: &mut Vec<TextButton<CommandBuffer>>) {
    let font = Rc::new(RefCell::new(Font::new("content/fonts/NotoMono-Regular.ttf", 24).unwrap()));
    let line_height = font.borrow_mut().line_height();

    let mut layout = layout::adaptive_rows(
        Position::new(5.0, 5.0),
        (35.0, line_height + 5.0),
        70.0,
        10
    );

    texture_buttons.push(TextButton::new(
        font.clone(),
        "P".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Pencil));
        })),
        None,
        None
    ));

    texture_buttons.push(TextButton::new(
        font.clone(),
        "E".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Eraser));
        })),
        None,
        None
    ));

    texture_buttons.push(TextButton::new(
        font.clone(),
        "L".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Line));
        })),
        None,
        None
    ));

    texture_buttons.push(TextButton::new(
        font.clone(),
        "R".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Rectangle));
        })),
        None,
        None
    ));

    texture_buttons.push(TextButton::new(
        font.clone(),
        "C".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Circle));
        })),
        None,
        None
    ));

    texture_buttons.push(TextButton::new(
        font.clone(),
        "BF".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::BucketFill));
        })),
        None,
        None
    ));

    texture_buttons.push(TextButton::new(
        font.clone(),
        "CP".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::ColorPicker));
        })),
        None,
        None
    ));

    texture_buttons.push(TextButton::new(
        font.clone(),
        "S".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Selection(SelectionSubTool::Select)));
        })),
        None,
        None
    ));

    texture_buttons.push(TextButton::new(
        font.clone(),
        "M".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Selection(SelectionSubTool::MovePixels)));
        })),
        None,
        None
    ));

    texture_buttons.push(TextButton::new(
        font.clone(),
        "RS".to_owned(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Selection(SelectionSubTool::ResizePixels)));
        })),
        None,
        None
    ));
}

fn generate_color_palette(buttons: &mut Vec<TextureButton<CommandBuffer>>,
                          solid_color_buttons: &mut Vec<SolidColorButton<CommandBuffer>>) {
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

    let start_x = 0.0;
    let start_y = 280.0;
    let selected_color_width = 32.0;
    let selected_color_height = 32.0;

    solid_color_buttons.push(SolidColorButton::new(
        image::Rgba([255, 0, 0, 255]),
        Rectangle::new(start_x, start_y, selected_color_width, selected_color_height),
        None,
        None,
        Some(Box::new(move |button, command| {
            if let Command::SetColor(color) = command {
                button.set_color(*color);
            }
        }))
    ));

    solid_color_buttons.push(SolidColorButton::new(
        image::Rgba([0, 0, 0, 255]),
        Rectangle::new(selected_color_width / 2.0, start_y + selected_color_height / 2.0, selected_color_width, selected_color_height),
        None,
        None,
        Some(Box::new(move |button, command| {
            if let Command::SetAlternativeColor(color) = command {
                button.set_color(*color);
            }
        }))
    ));

    let cell_size = (16, 16);

    let layout = layout::adaptive_rows(
        Position::new(start_x, start_y + selected_color_height * 1.5 + 5.0),
        (cell_size.0 as f32, cell_size.1 as f32),
        48.0,
        colors.len()
    );

    for (color, position) in colors.iter().zip_eq(layout) {
        let mut image = image::RgbaImage::new(cell_size.0, cell_size.1);
        let color = *color;

        for pixel in image.pixels_mut() {
            *pixel = color;
        }

        buttons.push(TextureButton::new(
            &image,
            position,
            Some(Box::new(move |command_buffer| {
                command_buffer.push(Command::SetColor(color));
            })),
            Some(Box::new(move |command_buffer| {
                command_buffer.push(Command::SetAlternativeColor(color));
            })),
            None
        ));
    }
}