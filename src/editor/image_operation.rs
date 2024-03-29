use std::collections::HashMap;
use std::fmt::Display;

use itertools::Itertools;

use image::{Pixel, FilterType};

use crate::editor::image::{Color};
use crate::editor::image_operation_helpers::{sub_image, draw_block, draw_line, draw_circle, fill_rectangle, bucket_fill, draw_line_anti_aliased_thick, draw_circle_anti_aliased_thick, color_gradient, pencil_stroke_anti_aliased, rotate_image, draw_line_thick, draw_pixel};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ImageOperationMarker {
    BeginDraw,
    EndDraw
}

#[derive(Debug, Clone)]
pub enum ColorGradientType {
    Linear,
    Radial
}

#[derive(Debug, Clone)]
pub enum ImageOperation {
    Empty,
    Marker(ImageOperationMarker, Option<String>),
    Sequential(Option<String>, Vec<ImageOperation>),
    SetImage { start_x: i32, start_y: i32, image: image::RgbaImage, blend: bool },
    SetSparseImage { image: SparseImage },
    SetOptionalImage { image: OptionalImage },
    SetScaledImage { image: image::RgbaImage, start_x: i32, start_y: i32, scale_x: f32, scale_y: f32 },
    SetRotatedImage { image: image::RgbaImage, start_x: i32, start_y: i32, end_x: i32, end_y: i32, rotation: f32 },
    SetPixel { x: i32, y: i32, color: Color },
    Block { x: i32, y: i32, color: Color, blend: bool, side_half_width: i32 },
    Line { start_x: i32, start_y: i32, end_x: i32, end_y: i32, color: Color, blend: bool, anti_aliased: Option<bool>, side_half_width: i32 },
    PencilStroke { start_x: i32, start_y: i32, end_x: i32, end_y: i32, prev_start_x: Option<i32>, prev_start_y: Option<i32>, color: Color, blend: bool, anti_aliased: Option<bool>, side_half_width: i32 },
    Rectangle { start_x: i32, start_y: i32, end_x: i32, end_y: i32, border_half_width: i32, color: Color, blend: bool },
    FillRectangle { start_x: i32, start_y: i32, end_x: i32, end_y: i32, color: Color, blend: bool },
    Circle { center_x: i32, center_y: i32, radius: i32, border_half_width: i32, color: Color, blend: bool, anti_aliased: Option<bool> },
    FillCircle { center_x: i32, center_y: i32, radius: i32, color: Color, blend: bool },
    BucketFill { start_x: i32, start_y: i32, fill_color: Color, tolerance: f32 },
    ColorGradient { start_x: i32, start_y: i32, end_x: i32, end_y: i32, first_color: Color, second_color: Color, gradient_type: ColorGradientType }
}

pub trait ImageSource {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn get_pixel(&self, x: u32, y: u32) -> Color;
}

pub trait ImageOperationSource : ImageSource {
    fn put_pixel(&mut self, x: u32, y: u32, pixel: Color);

    fn put_pixel_with_blend(&mut self, x: u32, y: u32, pixel: Color) {
        let mut current = self.get_pixel(x, y);
        current.blend(&pixel);
        self.put_pixel(x, y, current);
    }
}

impl ImageOperation {
    pub fn apply<T: ImageOperationSource>(&self, update_op: &mut T, undo: bool) -> Option<ImageOperation> {
        match self {
            ImageOperation::Empty => {
                None
            }
            ImageOperation::Marker(_, _) => {
                None
            }
            ImageOperation::Sequential(message, ops) => {
                let mut undo_ops = Vec::new();

                for op in ops {
                    if let Some(undo_op) = op.apply(update_op, undo) {
                        undo_ops.push(undo_op);
                    }
                }

                if !undo_ops.is_empty() {
                    undo_ops.reverse();
                    Some(ImageOperation::Sequential(message.clone(), undo_ops))
                } else {
                    None
                }
            }
            ImageOperation::SetImage { start_x, start_y, image, blend } => {
                let undo_image = if undo {
                    Some(
                        sub_image(
                            update_op,
                            *start_x,
                            *start_y,
                            *start_x + image.width() as i32,
                            *start_y + image.height() as i32
                        )
                    )
                } else {
                    None
                };

                for y in 0..image.height() {
                    for x in 0..image.width() {
                        let image_x = *start_x + x as i32;
                        let image_y = *start_y + y as i32;

                        if image_x >= 0 && image_x < update_op.width() as i32 && image_y >= 0 && image_y < update_op.height() as i32 {
                            if *blend {
                                update_op.put_pixel_with_blend(image_x as u32, image_y as u32, *image.get_pixel(x, y));
                            } else {
                                update_op.put_pixel(image_x as u32, image_y as u32, *image.get_pixel(x, y));
                            }
                        }
                    }
                }

                undo_image.map(|image| ImageOperation::SetImage { start_x: *start_x, start_y: *start_y, image, blend: false })
            }
            ImageOperation::SetSparseImage { image: changes } => {
                for ((x, y), pixel) in changes {
                    update_op.put_pixel(*x, *y, *pixel);
                }

                None
            }
            ImageOperation::SetOptionalImage { image } => {
                for y in 0..image.height {
                    for x in 0..image.width {
                        if let Some(color) = image.data[(y * image.width + x) as usize] {
                            update_op.put_pixel(x, y, color);
                        }
                    }
                }

                None
            }
            ImageOperation::SetScaledImage { image, start_x, start_y, scale_x, scale_y } => {
                let new_width = (image.width() as f32 * scale_x).round() as u32;
                let new_height = (image.height() as f32 * scale_y).round() as u32;

                let resized_image = image::imageops::resize(
                    image,
                    new_width,
                    new_height,
                    FilterType::Triangle
                );

                ImageOperation::SetImage {
                    start_x: *start_x,
                    start_y: *start_y,
                    image: resized_image,
                    blend: false
                }.apply(update_op, undo)
            }
            ImageOperation::SetRotatedImage { image, start_x, start_y, end_x, end_y, rotation } => {
                let rotated_image = rotate_image(image, *rotation, FilterType::Triangle);
                let start_fx = ((start_x + end_x) as f32 * 0.5).floor() - (rotated_image.width() as f32 * 0.5).floor();
                let start_fy = ((start_y + end_y) as f32 * 0.5).floor() - (rotated_image.height() as f32 * 0.5).floor();
                let start_x = start_fx as i32;
                let start_y = start_fy as i32;

                ImageOperation::SetImage {
                    start_x,
                    start_y,
                    image: rotated_image,
                    blend: true
                }.apply(update_op, undo)
            }
            ImageOperation::SetPixel { x, y, color } => {
                let width = update_op.width();
                let height = update_op.height();
                let original_color = if *x >= 0 && *x < width as i32 && *y >= 0 && *y < height as i32 {
                    let original_color = if undo {
                        Some(update_op.get_pixel(*x as u32, *y as u32))
                    } else {
                        None
                    };

                    update_op.put_pixel(*x as u32, *y as u32, *color);
                    original_color
                } else {
                    None
                };

                original_color.map(|original_color| ImageOperation::SetPixel { x: *x, y: *y, color: original_color })
            }
            ImageOperation::Block { x, y, color, blend, side_half_width } => {
                let mut undo_image = SparseImage::new();
                draw_block(update_op, *x, *y, *side_half_width, *color, *blend, undo, &mut undo_image);

                if undo {
                    Some(ImageOperation::SetSparseImage { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::Line { start_x, start_y, end_x, end_y, color, blend, anti_aliased, side_half_width } => {
                let mut undo_image = SparseImage::new();

                if anti_aliased.unwrap_or(true) {
                    draw_line_anti_aliased_thick(
                        update_op,
                        *start_x,
                        *start_y,
                        *end_x,
                        *end_y,
                        *side_half_width,
                        *color,
                        undo,
                        &mut undo_image
                    );
                } else {
                    draw_line_thick(
                        update_op,
                        *start_x,
                        *start_y,
                        *end_x,
                        *end_y,
                        *side_half_width,
                        *color,
                        *blend,
                        undo,
                        &mut undo_image
                    );
                }

                if undo {
                    Some(ImageOperation::SetSparseImage { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::PencilStroke { start_x, start_y, end_x, end_y, prev_start_x, prev_start_y, color, blend, anti_aliased, side_half_width } => {
                let mut undo_image = SparseImage::new();

                if anti_aliased.unwrap_or(true) {
                    pencil_stroke_anti_aliased(
                        update_op,
                        *start_x,
                        *start_y,
                        *end_x,
                        *end_y,
                        *prev_start_x,
                        *prev_start_y,
                        *side_half_width,
                        *color,
                        undo,
                        &mut undo_image
                    );
                } else {
                    draw_line(
                        *start_x,
                        *start_y,
                        *end_x,
                        *end_y,
                        |center_x: i32, center_y: i32, _| {
                            draw_circle(
                                center_x,
                                center_y,
                                *side_half_width,
                                true,
                                |center_x: i32, center_y: i32| {
                                    draw_block(update_op, center_x, center_y, 0, *color, *blend, undo, &mut undo_image);
                                }
                            );
                        }
                    );
                }

                if undo {
                    Some(ImageOperation::SetSparseImage { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::FillRectangle { start_x, start_y, end_x, end_y, color, blend } => {
                let width = update_op.width() as i32;
                let height = update_op.height() as i32;

                let min_x = std::cmp::max(0, *start_x);
                let min_y = std::cmp::max(0, *start_y);
                let max_x = std::cmp::min(width, *end_x);
                let max_y = std::cmp::min(height, *end_y);

                let undo_image = if undo {
                    Some(sub_image(update_op, min_x, min_y, max_x + 1, max_y + 1))
                } else {
                    None
                };

                fill_rectangle(
                    min_x, min_y,
                    max_x, max_y,
                    |x, y| {
                        if *blend {
                            update_op.put_pixel_with_blend(x as u32, y as u32, *color)
                        } else {
                            update_op.put_pixel(x as u32, y as u32, *color)
                        }
                    }
                );

                undo_image.map(|image| ImageOperation::SetImage { start_x: min_x, start_y: min_y, image, blend: false })
            }
            ImageOperation::Rectangle { start_x, start_y, end_x, end_y, border_half_width: side_half_width, color, blend } => {
                let mut undo_ops = Vec::new();

                undo_ops.push(
                    ImageOperation::Line {
                        start_x: start_x.clone() - (side_half_width * 2 + 1),
                        start_y: start_y.clone() - side_half_width - 1,
                        end_x: end_x.clone() + (side_half_width * 2 + 1),
                        end_y: start_y.clone() - side_half_width - 1,
                        color: color.clone(),
                        blend: *blend,
                        anti_aliased: Some(false),
                        side_half_width: *side_half_width
                    }.apply(update_op, undo)
                );

                undo_ops.push(
                    ImageOperation::Line {
                        start_x: end_x.clone() + side_half_width + 1,
                        start_y: start_y.clone(),
                        end_x: end_x.clone() + side_half_width + 1,
                        end_y: end_y.clone(),
                        color: color.clone(),
                        blend: *blend,
                        anti_aliased: Some(false),
                        side_half_width: *side_half_width
                    }.apply(update_op, undo)
                );

                undo_ops.push(
                    ImageOperation::Line {
                        start_x: start_x.clone() - side_half_width - 1,
                        start_y: end_y.clone(),
                        end_x: start_x.clone() - side_half_width - 1,
                        end_y: start_y.clone(),
                        color: color.clone(),
                        blend: *blend,
                        anti_aliased: Some(false),
                        side_half_width: *side_half_width
                    }.apply(update_op, undo)
                );

                undo_ops.push(
                    ImageOperation::Line {
                        start_x: end_x.clone() + (side_half_width * 2 + 1),
                        start_y: end_y.clone() + side_half_width + 1,
                        end_x: start_x.clone() - (side_half_width * 2 + 1),
                        end_y: end_y.clone() + side_half_width + 1,
                        color: color.clone(),
                        blend: *blend,
                        anti_aliased: Some(false),
                        side_half_width: *side_half_width
                    }.apply(update_op, undo)
                );

                let mut undo_ops = undo_ops.into_iter().flatten().collect::<Vec<_>>();
                if !undo_ops.is_empty() {
                    undo_ops.reverse();
                    Some(ImageOperation::Sequential(None, undo_ops))
                } else {
                    None
                }
            }
            ImageOperation::Circle { center_x, center_y, radius, border_half_width, color, blend, anti_aliased} => {
                let mut undo_image = SparseImage::new();

                if anti_aliased.unwrap_or(true) {
                    draw_circle_anti_aliased_thick(
                        update_op,
                        *center_x,
                        *center_y,
                        *radius,
                        *border_half_width,
                        *color,
                        undo,
                        &mut undo_image
                    );
                } else {
                    draw_circle(
                        *center_x,
                        *center_y,
                        *radius,
                        false,
                        |center_x: i32, center_y: i32| {
                            draw_block(update_op, center_x, center_y, *border_half_width, *color, *blend, undo, &mut undo_image);
                        }
                    );
                }

                if undo {
                    Some(ImageOperation::SetSparseImage { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::FillCircle { center_x, center_y, radius, color, blend } => {
                let mut undo_image = SparseImage::new();

                draw_circle(
                    *center_x,
                    *center_y,
                    *radius,
                    true,
                    |center_x: i32, center_y: i32| {
                        draw_pixel(update_op, center_x, center_y, *color, *blend, undo, &mut undo_image);
                    }
                );

                if undo {
                    Some(ImageOperation::SetSparseImage { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::BucketFill { start_x, start_y, fill_color, tolerance } => {
                let mut undo_image = OptionalImage::new(update_op.width(), update_op.height());

                bucket_fill(
                    update_op,
                    *start_x,
                    *start_y,
                    *fill_color,
                    *tolerance,
                    undo,
                    &mut undo_image
                );

                if undo {
                    Some(ImageOperation::SetOptionalImage { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::ColorGradient { start_x, start_y, end_x, end_y, first_color, second_color, gradient_type } => {
                let undo_image = if undo {
                    Some(
                        sub_image(
                            update_op,
                            0,
                            0,
                            update_op.width() as i32,
                            update_op.height() as i32
                        )
                    )
                } else {
                    None
                };

                color_gradient(
                    update_op,
                    *start_x,
                    *start_y,
                    *end_x,
                    *end_y,
                    *first_color,
                    *second_color,
                    gradient_type.clone()
                );

                undo_image.map(|image| ImageOperation::SetImage { start_x: 0, start_y: 0, image, blend: false })
            }
        }
    }

    pub fn is_marker(&self, compare_marker: ImageOperationMarker) -> bool {
        return match self {
            ImageOperation::Marker(marker, _) => { marker == &compare_marker },
            ImageOperation::Sequential(_, ops) => { ops.iter().any(|x| x.is_marker(compare_marker.clone())) },
            _ => { false }
        }
    }

    pub fn is_any_marker(&self) -> bool {
        return match self {
            ImageOperation::Marker(_, _) => true,
            ImageOperation::Sequential(_, ops) => { ops.iter().any(|x| x.is_any_marker()) },
            _ => { false }
        }
    }

    pub fn get_first_marker_message(&self) -> Option<&String> {
        match self {
            ImageOperation::Marker(_, message) => message.as_ref(),
            ImageOperation::Sequential(_, ops) => ops.iter().map(|op| op.get_first_marker_message()).flatten().next(),
            _ => None
        }
    }

    pub fn remove_markers(self) -> Self {
        match self {
            ImageOperation::Marker(_, _) => {
                ImageOperation::Empty
            },
            ImageOperation::Sequential(message, ops) => {
                ImageOperation::Sequential(message, ops.into_iter().map(|x| x.remove_markers()).collect())
            },
            _ => self
        }
    }
}

impl Display for ImageOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageOperation::Empty => write!(f, ""),
            ImageOperation::Marker(_, message) => write!(f, "{}", message.as_ref().unwrap_or(&String::new())),
            ImageOperation::Sequential(message, ops) => {
                match message {
                    Some(message) => write!(f, "{}", message),
                    None => write!(f, "{}", ops.iter().map(|op| format!("{}", op)).filter(|x| !x.is_empty()).join(", ")),
                }
            }
            ImageOperation::SetImage { .. } => write!(f, "Set image"),
            ImageOperation::SetSparseImage { .. } => write!(f, "Set image"),
            ImageOperation::SetOptionalImage { .. } => write!(f, "Set image"),
            ImageOperation::SetScaledImage { .. } => write!(f, "Scale pixels"),
            ImageOperation::SetRotatedImage { .. } => write!(f, "Rotate pixels"),
            ImageOperation::SetPixel { .. } => write!(f, "Set pixel"),
            ImageOperation::Block { .. } => write!(f, "Block"),
            ImageOperation::Line { .. } => write!(f, "Line"),
            ImageOperation::PencilStroke { .. } => write!(f, "Pencil stroke"),
            ImageOperation::Rectangle { .. } => write!(f, "Rectangle"),
            ImageOperation::FillRectangle { .. } => write!(f, "Rectangle"),
            ImageOperation::Circle { .. } => write!(f, "Circle"),
            ImageOperation::FillCircle { .. } => write!(f, "Circle"),
            ImageOperation::BucketFill { .. } => write!(f, "Bucket fill"),
            ImageOperation::ColorGradient { .. } => write!(f, "Color gradient"),
        }
    }
}

pub type SparseImage = HashMap<(u32, u32), Color>;

#[derive(Debug, Clone)]
pub struct OptionalImage {
    width: u32,
    height: u32,
    data: Vec<Option<Color>>
}

impl OptionalImage {
    pub fn new(width: u32, height: u32) -> OptionalImage {
        OptionalImage {
            width,
            height,
            data: vec![None; (width * height) as usize]
        }
    }

    pub fn contains_key(&self, key: &(u32, u32)) -> bool {
        let (x, y) = key;
        if *x < self.width && *y < self.height {
            self.data[(y * self.width + x) as usize].is_some()
        } else {
            false
        }
    }

    pub fn insert(&mut self, key: (u32, u32), color: Color) {
        let (x, y) = key;
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = Some(color);
        }
    }
}

pub fn add_op_sequential(op: &mut Option<ImageOperation>, new_op: Option<ImageOperation>) {
    if let Some(new_op) = new_op {
        match op {
            Some(ImageOperation::Sequential(_, ops)) => {
                ops.push(new_op);
            }
            Some(current_op) => {
                let mut current_op_stolen = ImageOperation::Empty;
                std::mem::swap(&mut current_op_stolen, current_op);
                *current_op = ImageOperation::Sequential(None, vec![current_op_stolen, new_op]);
            }
            None => {
                *op = Some(new_op);
            }
        }
    }
}

pub fn select_latest<const N: usize>(ops: [Option<ImageOperation>; N]) -> Option<ImageOperation> {
    let mut last = None;
    for op in ops {
        if op.is_some() {
            last = op;
        }
    }

    last
}