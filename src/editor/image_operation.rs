use std::collections::HashMap;

use crate::editor::image::{Color};
use crate::editor::image_operation_helpers::{sub_image, draw_block, draw_line, draw_circle, fill_rectangle, bucket_fill};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ImageOperationMarker {
    BeginDraw,
    EndDraw
}

pub type SparseImage = HashMap<(u32, u32), Color>;

#[derive(Debug)]
pub enum ImageOperation {
    Empty,
    Marker(ImageOperationMarker),
    Sequential(Vec<ImageOperation>),
    SetImageSparse { image: SparseImage },
    SetImage { start_x: i32, start_y: i32, image: image::RgbaImage },
    SetPixel { x: i32, y: i32, color: Color },
    DrawBlock { x: i32, y: i32, color: Color, side_half_width: i32 },
    DrawLine { start_x: i32, start_y: i32, end_x: i32, end_y: i32, color: Color, side_half_width: i32 },
    FillRectangle { start_x: i32, start_y: i32, end_x: i32, end_y: i32, color: Color },
    DrawRectangle { start_x: i32, start_y: i32, end_x: i32, end_y: i32, color: Color },
    DrawCircle { center_x: i32, center_y: i32, radius: i32, border_side_half_width: i32, color: Color },
    FillCircle { center_x: i32, center_y: i32, radius: i32, color: Color },
    BucketFill { start_x: i32, start_y: i32, fill_color: Color }
}

pub trait ImageSource {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn get_pixel(&self, x: u32, y: u32) -> Color;
}

pub trait ImageOperationSource : ImageSource {
    fn put_pixel(&mut self, x: u32, y: u32, pixel: Color);
}

impl ImageOperation {
    pub fn apply<T: ImageOperationSource>(&self, update_op: &mut T, undo: bool) -> Option<ImageOperation> {
        match self {
            ImageOperation::Empty => {
                None
            }
            ImageOperation::Marker(_) => {
                None
            }
            ImageOperation::Sequential(ops) => {
                let mut undo_ops = Vec::new();

                for op in ops {
                    if let Some(undo_op) = op.apply(update_op, undo) {
                        undo_ops.push(undo_op);
                    }
                }

                if !undo_ops.is_empty() {
                    undo_ops.reverse();
                    Some(ImageOperation::Sequential(undo_ops))
                } else {
                    None
                }
            }
            ImageOperation::SetImageSparse { image: changes } => {
                for ((x, y), pixel) in changes {
                    update_op.put_pixel(*x, *y, *pixel);
                }

                None
            }
            ImageOperation::SetImage { start_x, start_y, image } => {
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

                for y in 0.. image.height() {
                    for x in 0..image.width() {
                        let image_x = *start_x + x as i32;
                        let image_y = *start_y + y as i32;

                        if image_x >= 0 && image_x < update_op.width() as i32 && image_y >= 0 && image_y < update_op.height() as i32 {
                            update_op.put_pixel(image_x as u32, image_y as u32, *image.get_pixel(x, y));
                        }
                    }
                }

                undo_image.map(|image| ImageOperation::SetImage { start_x: *start_x, start_y: *start_y, image })
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
            ImageOperation::DrawBlock { x, y, color, side_half_width } => {
                let mut undo_image = SparseImage::new();
                draw_block(update_op, *x, *y, *side_half_width, *color, undo, &mut undo_image);

                if undo {
                    Some(ImageOperation::SetImageSparse { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::DrawLine { start_x, start_y, end_x, end_y, color, side_half_width } => {
                let mut undo_image = SparseImage::new();
                draw_line(
                    *start_x,
                    *start_y,
                    *end_x,
                    *end_y,
                    |center_x: i32, center_y: i32| {
                        draw_block(update_op, center_x, center_y, *side_half_width, *color, undo, &mut undo_image);
                    }
                );

                if undo {
                    Some(ImageOperation::SetImageSparse { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::FillRectangle { start_x, start_y, end_x, end_y, color } => {
                let width = update_op.width() as i32;
                let height = update_op.height() as i32;

                let min_x = std::cmp::max(0, *start_x);
                let min_y = std::cmp::max(0, *start_y);
                let max_x = std::cmp::min(width, *end_x + 1);
                let max_y = std::cmp::min(height, *end_y + 1);

                let undo_image = if undo {
                    Some(sub_image(update_op, min_x, min_y, max_x, max_y))
                } else {
                    None
                };

                fill_rectangle(
                    min_x, min_y,
                    max_x, max_y,
                    |x, y| update_op.put_pixel(x as u32, y as u32, *color)
                );

                undo_image.map(|image| ImageOperation::SetImage { start_x: min_x, start_y: min_y, image })
            }
            ImageOperation::DrawRectangle { start_x, start_y, end_x, end_y, color } => {
                let mut undo_ops = Vec::new();

                let side_half_width = 0;
                undo_ops.push(ImageOperation::DrawLine {
                    start_x: start_x.clone(),
                    start_y: start_y.clone(),
                    end_x: end_x.clone(),
                    end_y: start_y.clone(),
                    color: color.clone(),
                    side_half_width
                }.apply(update_op, undo));

                undo_ops.push(ImageOperation::DrawLine {
                    start_x: end_x.clone(),
                    start_y: start_y.clone(),
                    end_x: end_x.clone(),
                    end_y: end_y.clone(),
                    color: color.clone(),
                    side_half_width
                }.apply(update_op, undo));

                undo_ops.push(ImageOperation::DrawLine {
                    start_x: end_x.clone(),
                    start_y: end_y.clone(),
                    end_x: start_x.clone(),
                    end_y: end_y.clone(),
                    color: color.clone(),
                    side_half_width
                }.apply(update_op, undo));

                undo_ops.push(ImageOperation::DrawLine {
                    start_x: start_x.clone(),
                    start_y: end_y.clone(),
                    end_x: start_x.clone(),
                    end_y: start_y.clone(),
                    color: color.clone(),
                    side_half_width
                }.apply(update_op, undo));

                let mut undo_ops = undo_ops.into_iter().flatten().collect::<Vec<_>>();
                if !undo_ops.is_empty() {
                    undo_ops.reverse();
                    Some(ImageOperation::Sequential(undo_ops))
                } else {
                    None
                }
            }
            ImageOperation::DrawCircle { center_x, center_y, radius, border_side_half_width, color } => {
                let mut undo_image = SparseImage::new();

                draw_circle(
                    *center_x,
                    *center_y,
                    *radius,
                    false,
                    |center_x: i32, center_y: i32| {
                        draw_block(update_op, center_x, center_y, *border_side_half_width, *color, undo, &mut undo_image);
                    }
                );

                if undo {
                    Some(ImageOperation::SetImageSparse { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::FillCircle { center_x, center_y, radius, color } => {
                let mut undo_image = SparseImage::new();

                draw_circle(
                    *center_x,
                    *center_y,
                    *radius,
                    true,
                    |center_x: i32, center_y: i32| {
                        draw_block(update_op, center_x, center_y, 0, *color, undo, &mut undo_image);
                    }
                );

                if undo {
                    Some(ImageOperation::SetImageSparse { image: undo_image })
                } else {
                    None
                }
            }
            ImageOperation::BucketFill { start_x, start_y, fill_color } => {
                let mut undo_image = SparseImage::new();

                bucket_fill(
                    update_op,
                    *start_x,
                    *start_y,
                    *fill_color,
                    undo,
                    &mut undo_image
                );

                if undo {
                    Some(ImageOperation::SetImageSparse { image: undo_image })
                } else {
                    None
                }
            }
        }
    }

    pub fn is_marker(&self, compare_marker: ImageOperationMarker) -> bool {
        return match self {
            ImageOperation::Marker(marker) => { marker == &compare_marker },
            ImageOperation::Sequential(ops) => { ops.iter().any(|x| x.is_marker(compare_marker)) },
            _ => { false }
        }
    }

    pub fn remove_markers(self) -> Self {
        match self {
            ImageOperation::Marker(_) => {
                ImageOperation::Empty
            },
            ImageOperation::Sequential(ops) => {
                ImageOperation::Sequential(ops.into_iter().map(|x| x.remove_markers()).collect())
            },
            _ => self
        }
    }
}