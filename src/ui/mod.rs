pub mod manager;
pub mod button;
pub mod layout;

pub use manager::Manager;
pub use button::Button;

use itertools::Itertools;

use crate::command_buffer::{Command};
use crate::rendering::prelude::Position;
use crate::editor::draw_tools::DrawTools;

fn generate_draw_tools() -> Vec<Button> {
    let mut buttons = Vec::new();
    let mut layout = layout::adaptive_rows(
        Position::new(0.0, 0.0),
        (40.0, 40.0),
        40.0,
        5
    );

    buttons.push(Button::new(
        &image::open("content/ui/pencil.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Pencil));
        })
    ));

    buttons.push(Button::new(
        &image::open("content/ui/line.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Line));
        })
    ));

    buttons.push(Button::new(
        &image::open("content/ui/rectangle.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Rectangle));
        })
    ));

    buttons.push(Button::new(
        &image::open("content/ui/select.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Selection));
        })
    ));

    buttons.push(Button::new(
        &image::open("content/ui/effect.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Box::new(|command_buffer| {
            command_buffer.push(Command::SetDrawTool(DrawTools::Effect));
        })
    ));

    buttons
}

fn generate_color_palette() -> Vec<Button> {
    let mut buttons = Vec::new();

    let colors = [[0, 127, 255], [0, 127, 255], [0, 127, 255]]
        .iter()
        .multi_cartesian_product()
        .collect::<Vec<_>>();

    let cell_size = (16, 16);

    let layout = layout::adaptive_rows(
        Position::new(0.0, 200.0),
        (cell_size.0 as f32, cell_size.1 as f32),
        48.0,
        colors.len()
    );

    for (color, position) in colors.iter().zip_eq(layout) {
        let mut image = image::RgbaImage::new(cell_size.0, cell_size.1);
        let color = image::Rgba([*color[0], *color[1], *color[2], 255]);

        for pixel in image.pixels_mut() {
            *pixel = color;
        }

        buttons.push(Button::new(
            &image,
            position,
            Box::new(move |command_buffer| {
                command_buffer.push(Command::SetColor(color));
            })
        ));
    }

    buttons
}

pub fn create() -> Manager {
    let mut buttons = Vec::new();
    buttons.append(&mut generate_draw_tools());
    buttons.append(&mut generate_color_palette());

    Manager::new(buttons)
}