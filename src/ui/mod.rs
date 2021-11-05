use itertools::Itertools;

pub mod manager;
pub mod button;
pub mod layout;
pub mod color_wheel;
pub mod layers;

pub use manager::Manager;
pub use button::TextureButton;

use crate::command_buffer::{Command, CommandBuffer};
use crate::rendering::prelude::{Position, Rectangle};
use crate::editor::tools::{Tools, SelectionSubTool, ColorWheelMode};
use crate::editor::image_operation_helpers::hsv_to_rgb;
use crate::ui::button::{SolidColorButton};
use crate::ui::manager::BoxGenericButton;
use crate::program::LEFT_SIDE_PANEL_WIDTH;

pub fn create() -> Manager {
    let mut buttons = Vec::<BoxGenericButton>::new();

    generate_draw_tools(&mut buttons);
    generate_color_palette(&mut buttons);

    Manager::new(buttons)
}

fn generate_draw_tools(buttons: &mut Vec<BoxGenericButton>) {
    let mut layout = layout::adaptive_rows(
        Position::new(10.0, 10.0),
        (35.0, 35.0),
        LEFT_SIDE_PANEL_WIDTH as f32,
        13
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/pencil.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::Pencil));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/block_pencil.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::BlockPencil));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/eraser.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::Eraser));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/line.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::Line));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/rectangle.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::Rectangle));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/circle.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::Circle));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/fill.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::BucketFill));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/color_picker.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::ColorPicker));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/color_gradient.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::ColorGradient));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/selection.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::Selection(SelectionSubTool::Select)));
            })),
            None,
            None
        ))
    );

    buttons.push(
        Box::new(TextureButton::<CommandBuffer>::new(
            &image::open("content/ui/move.png").unwrap().into_rgba(),
            layout.next().unwrap(),
            Some(Box::new(|command_buffer| {
                command_buffer.push(Command::SetTool(Tools::Selection(SelectionSubTool::MovePixels)));
            })),
            None,
            None
        ))
    );

    buttons.push(Box::new(TextureButton::<CommandBuffer>::new(
        &image::open("content/ui/resize.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Selection(SelectionSubTool::ResizePixels)));
        })),
        None,
        None
    )));

    buttons.push(Box::new(TextureButton::<CommandBuffer>::new(
        &image::open("content/ui/rotate.png").unwrap().into_rgba(),
        layout.next().unwrap(),
        Some(Box::new(|command_buffer| {
            command_buffer.push(Command::SetTool(Tools::Selection(SelectionSubTool::RotatePixels)));
        })),
        None,
        None
    )));
}

fn generate_color_palette(buttons: &mut Vec<BoxGenericButton>) {
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

    let start_x = 10.0;
    let start_y = 280.0;
    let selected_color_width = 32.0;
    let selected_color_height = 32.0;

    buttons.push(
        Box::new(SolidColorButton::<CommandBuffer>::new(
            image::Rgba([0, 0, 0, 255]),
            Rectangle::new(
                start_x + selected_color_width / 2.0,
                start_y + selected_color_height / 2.0,
                selected_color_width,
                selected_color_height
            ),
            Some(Box::new(move |command_buffer| {
                command_buffer.push(Command::SetTool(Tools::ColorWheel(ColorWheelMode::SelectAlternativeColor)));
            })),
            None,
            Some(Box::new(move |button, command| {
                if let Command::SetAlternativeColor(color) = command {
                    button.set_color(*color);
                }
            }))
        ))
    );

    buttons.push(
        Box::new(SolidColorButton::<CommandBuffer>::new(
            image::Rgba([255, 0, 0, 255]),
            Rectangle::new(start_x, start_y, selected_color_width, selected_color_height),
            Some(Box::new(move |command_buffer| {
                command_buffer.push(Command::SetTool(Tools::ColorWheel(ColorWheelMode::SelectColor)));
            })),
            None,
            Some(Box::new(move |button, command| {
                if let Command::SetColor(color) = command {
                    button.set_color(*color);
                }
            }))
        ))
    );

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

        buttons.push(
            Box::new(TextureButton::<CommandBuffer>::new(
                &image,
                position,
                Some(Box::new(move |command_buffer| {
                    command_buffer.push(Command::SetColor(color));
                })),
                Some(Box::new(move |command_buffer| {
                    command_buffer.push(Command::SetAlternativeColor(color));
                })),
                None
            ))
        );
    }
}