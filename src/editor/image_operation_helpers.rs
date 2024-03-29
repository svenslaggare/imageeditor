use std::collections::HashSet;

use image::{Pixel, FilterType};

use cgmath::{ElementWise, Vector4, Point2, Matrix3, Transform};

use crate::editor::image_operation::{ImageSource, ImageOperationSource, SparseImage, OptionalImage, ColorGradientType};
use crate::editor::Color;

pub fn draw_pixel<T: ImageOperationSource>(update_op: &mut T,
                                           x: i32,
                                           y: i32,
                                           color: Color,
                                           blend: bool,
                                           undo: bool,
                                           undo_image: &mut SparseImage) {
    if x >= 0 && x < update_op.width() as i32 && y >= 0 && y < update_op.height() as i32 {
        if undo && !undo_image.contains_key(&(x as u32, y as u32)) {
            undo_image.insert((x as u32, y as u32), update_op.get_pixel(x as u32, y as u32));
        }

        if blend {
            update_op.put_pixel_with_blend(x as u32, y as u32, color);
        } else {
            update_op.put_pixel(x as u32, y as u32, color);
        }
    }
}

pub fn draw_block<T: ImageOperationSource>(update_op: &mut T,
                                           center_x: i32,
                                           center_y: i32,
                                           side_half_width: i32,
                                           color: Color,
                                           blend: bool,
                                           undo: bool,
                                           undo_image: &mut SparseImage) {
    for y in (center_y - side_half_width)..(center_y + side_half_width + 1) {
        for x in (center_x - side_half_width)..(center_x + side_half_width + 1) {
            draw_pixel(
                update_op,
                x,
                y,
                color,
                blend,
                undo,
                undo_image
            );
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LineSegmentPart {
    Start,
    Middle,
    End
}

pub fn draw_line<F: FnMut(i32, i32, LineSegmentPart)>(x1: i32, y1: i32, x2: i32, y2: i32, mut set_pixel: F) {
    // using Bresenham's algorithm
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dx1 = dx.abs();
    let dy1 = dy.abs();

    let mut px = 2 * dy1 - dx1;
    let mut py = 2 * dx1 - dy1;

    let mut x;
    let mut y;

    if dy1 <= dx1 {
        let end_x;
        if dx >= 0 {
            x = x1;
            y = y1;
            end_x = x2;
        } else {
            x = x2;
            y = y2;
            end_x = x1;
        }

        set_pixel(x, y, LineSegmentPart::Start);

        while x < end_x {
            x += 1;
            if px < 0 {
                px += 2 * dy1;
            } else {
                if (dx < 0 && dy < 0) || (dx > 0 && dy > 0) {
                    y += 1;
                } else {
                    y -= 1;
                }
                px += 2 * (dy1 - dx1);
            }

            set_pixel(x, y, if x < end_x {LineSegmentPart::Middle} else {LineSegmentPart::End});
        }
    } else {
        let end_y;
        if dy >= 0 {
            x = x1;
            y = y1;
            end_y = y2;
        } else {
            x = x2;
            y = y2;
            end_y = y1;
        }

        set_pixel(x, y, LineSegmentPart::Start);

        while y < end_y {
            y += 1;
            if py <= 0 {
                py = py + 2 * dx1;
            } else {
                if dx < 0 && dy < 0 || (dx > 0 && dy > 0) {
                    x += 1;
                } else {
                    x -= 1;
                }
                py += 2 * (dx1 - dy1);
            }

            set_pixel(x, y, if y < end_y {LineSegmentPart::Middle} else {LineSegmentPart::End});
        }
    }
}

pub fn draw_line_thick<T: ImageOperationSource>(update_op: &mut T,
                                                x1: i32, y1: i32, x2: i32, y2: i32,
                                                side_half_width: i32,
                                                color: Color,
                                                blend: bool,
                                                undo: bool,
                                                undo_image: &mut SparseImage) {
    if side_half_width > 0 {
        let x1_f = x1 as f32;
        let y1_f = y1 as f32;
        let x2_f = x2 as f32;
        let y2_f = y2 as f32;

        let dx = x2_f - x1_f;
        let dy = y2_f - y1_f;
        let norm = ((dx * dx + dy * dy) as f32).sqrt();
        let (dx, dy) = if norm > 1E-6 {
            (dx / norm, dy / norm)
        } else {
            (0.0, 0.0)
        };

        let dx_perp = dy;
        let dy_perp = -dx;

        let mut pixels_draw = HashSet::new();
        let mut draw_pixel_guard = |x: i32, y: i32| {
            if pixels_draw.insert((x, y)) {
                draw_pixel(
                    update_op,
                    x, y,
                    color, blend,
                    undo, undo_image
                );
            }
        };

        draw_line(
            x1,
            y1,
            x2,
            y2,
            |long_center_x: i32, long_center_y: i32, long_part| {
                let long_center_x = long_center_x as f32;
                let long_center_y = long_center_y as f32;
                let width = side_half_width as f32;

                draw_line(
                    (long_center_x - dx_perp * width).round() as i32,
                    (long_center_y - dy_perp * width).round() as i32,
                    (long_center_x + dx_perp * width).round() as i32,
                    (long_center_y + dy_perp * width).round() as i32,
                    |short_center_x: i32, short_center_y: i32, short_part| {
                        if short_part == LineSegmentPart::Middle && long_part == LineSegmentPart::Middle {
                            draw_pixel_guard(short_center_x, short_center_y);
                            draw_pixel_guard(short_center_x + 1, short_center_y);
                            draw_pixel_guard(short_center_x - 1, short_center_y);
                            draw_pixel_guard(short_center_x, short_center_y + 1);
                            draw_pixel_guard(short_center_x, short_center_y - 1);
                        } else {
                            draw_pixel_guard(short_center_x, short_center_y);
                        }
                    }
                );
            }
        );
    } else {
        draw_line(
            x1,
            y1,
            x2,
            y2,
            |center_x: i32, center_y: i32, _| {
                draw_pixel(update_op, center_x, center_y, color, blend, undo, undo_image);
            }
        );
    }
}

pub fn draw_line_anti_aliased<T: ImageOperationSource>(update_op: &mut T,
                                                       mut x1: i32, mut y1: i32,
                                                       mut x2: i32, mut y2: i32,
                                                       color: Color,
                                                       undo: bool,
                                                       undo_image: &mut SparseImage) {
    let fpart = |x: f32| x - x.floor();
    let rfpart = |x: f32| 1.0 - fpart(x);
    let ipart = |x: f32| x.floor() as i32;

    let mut plot = |x: i32, y: i32, c: f32| {
        if !(x >= 0 && x < update_op.width() as i32 && y >= 0 && y < update_op.height() as i32) {
            return;
        }

        let pixel = update_op.get_pixel(x as u32, y as u32);
        if undo && !undo_image.contains_key(&(x as u32, y as u32)) {
            undo_image.insert((x as u32, y as u32), pixel);
        }

        let mut color = color;
        color[3] = (color[3] as f32 * c).clamp(0.0, 255.0) as u8;

        update_op.put_pixel(x as u32, y as u32, alpha_blend(color, pixel));
    };

    let steep = (y2 - y1).abs() > (x2 - x1).abs();

    if steep {
        std::mem::swap(&mut x1, &mut y1);
        std::mem::swap(&mut x2, &mut y2);
    }

    if x1 > x2 {
        std::mem::swap(&mut x1, &mut x2);
        std::mem::swap(&mut y1, &mut y2);
    }

    let dx = x2 - x1;
    let dy = y2 - y1;
    let gradient = if dx == 0 {
        1.0
    } else {
        dy as f32 / dx as f32
    };

    // handle first endpoint
    let x_end = x1;
    let y_end = y1 as f32 + gradient * (x_end - x1) as f32;
    let x_gap = rfpart(x1 as f32 + 0.5);
    let x_pixel1 = x_end;
    let y_pixel1 = ipart(y_end);

    if steep {
        plot(y_pixel1, x_pixel1, rfpart(y_end) * x_gap);
        plot(y_pixel1 + 1, x_pixel1, fpart(y_end) * x_gap);
    } else {
        plot(x_pixel1, y_pixel1, rfpart(y_end) * x_gap);
        plot(x_pixel1, y_pixel1 + 1, fpart(y_end) * x_gap);
    }

    let mut intercept_y = y_end + gradient; // first y-intersection for the main loop

    // handle second endpoint
    let x_end = x2;
    let y_end = y2 as f32 + gradient * (x_end - x2) as f32;
    let x_gap = fpart(x2 as f32 + 0.5);
    let x_pixel2 = x_end;
    let y_pixel2 = ipart(y_end);

    if steep {
        plot(y_pixel2, x_pixel2, rfpart(y_end) * x_gap);
        plot(y_pixel2 + 1, x_pixel2, fpart(y_end) * x_gap);
    } else {
        plot(x_pixel2, y_pixel2, rfpart(y_end) * x_gap);
        plot(x_pixel2, y_pixel2 + 1, fpart(y_end) * x_gap);
    }

    // Main loop
    if steep {
        for x in (x_pixel1 + 1)..x_pixel2 {
            plot(ipart(intercept_y), x, rfpart(intercept_y));
            plot(ipart(intercept_y) + 1, x, fpart(intercept_y));
            intercept_y = intercept_y + gradient;
        }
    } else {
        for x in (x_pixel1 + 1)..x_pixel2 {
            plot(x, ipart(intercept_y), rfpart(intercept_y));
            plot(x, ipart(intercept_y) + 1, fpart(intercept_y));
            intercept_y = intercept_y + gradient;
        }
    }
}

pub fn draw_line_anti_aliased_f32<T: ImageOperationSource>(update_op: &mut T,
                                                           mut x1: f32, mut y1: f32,
                                                           mut x2: f32, mut y2: f32,
                                                           color: Color,
                                                           set_pixel: &mut impl FnMut(i32, i32, LineSegmentPart) -> (bool, bool),
                                                           undo: bool,
                                                           undo_image: &mut SparseImage) {
    let fpart = |x: f32| x - x.floor();
    let rfpart = |x: f32| 1.0 - fpart(x);
    let ipart = |x: f32| x.floor() as i32;

    let mut plot = |part: LineSegmentPart, x: i32, y: i32, c: f32| {
        if !(x >= 0 && x < update_op.width() as i32 && y >= 0 && y < update_op.height() as i32) {
            return;
        }

        let (draw, use_alpha) = set_pixel(x, y, part);
        if !draw {
            return;
        }

        let pixel = update_op.get_pixel(x as u32, y as u32);
        if undo && !undo_image.contains_key(&(x as u32, y as u32)) {
            undo_image.insert((x as u32, y as u32), pixel);
        }

        let mut color = color;

        if use_alpha {
            color[3] = (color[3] as f32 * c).clamp(0.0, 255.0) as u8;
        }

        color = alpha_blend(color, pixel);

        update_op.put_pixel(x as u32, y as u32, color);
    };

    let steep = (y2 - y1).abs() > (x2 - x1).abs();

    if steep {
        std::mem::swap(&mut x1, &mut y1);
        std::mem::swap(&mut x2, &mut y2);
    }

    if x1 > x2 {
        std::mem::swap(&mut x1, &mut x2);
        std::mem::swap(&mut y1, &mut y2);
    }

    let dx = x2 - x1;
    let dy = y2 - y1;
    let gradient = if dx == 0.0 {
        1.0
    } else {
        dy as f32 / dx as f32
    };

    // handle first endpoint
    let x_end = x1.round();
    let y_end = y1 as f32 + gradient * (x_end - x1) as f32;
    let x_gap = rfpart(x1 as f32 + 0.5);
    let x_pixel1 = x_end;
    let y_pixel1 = ipart(y_end);

    if steep {
        plot(LineSegmentPart::Start, y_pixel1, x_pixel1 as i32, rfpart(y_end) * x_gap);
        plot(LineSegmentPart::Start, y_pixel1 + 1, x_pixel1 as i32, fpart(y_end) * x_gap);
    } else {
        plot(LineSegmentPart::Start, x_pixel1 as i32, y_pixel1, rfpart(y_end) * x_gap);
        plot(LineSegmentPart::Start, x_pixel1 as i32, y_pixel1 + 1, fpart(y_end) * x_gap);
    }

    let mut intercept_y = y_end + gradient; // first y-intersection for the main loop

    // handle second endpoint
    let x_end = x2.round();
    let y_end = y2 as f32 + gradient * (x_end - x2) as f32;
    let x_gap = fpart(x2 as f32 + 0.5);
    let x_pixel2 = x_end;
    let y_pixel2 = ipart(y_end);

    if steep {
        plot(LineSegmentPart::End, y_pixel2, x_pixel2 as i32, rfpart(y_end) * x_gap);
        plot(LineSegmentPart::End, y_pixel2 + 1, x_pixel2 as i32, fpart(y_end) * x_gap);
    } else {
        plot(LineSegmentPart::End, x_pixel2 as i32, y_pixel2, rfpart(y_end) * x_gap);
        plot(LineSegmentPart::End, x_pixel2 as i32, y_pixel2 + 1, fpart(y_end) * x_gap);
    }

    // Main loop
    if steep {
        for x in (x_pixel1 + 1.0) as i32..x_pixel2 as i32 {
            plot(LineSegmentPart::Middle, ipart(intercept_y), x, rfpart(intercept_y));
            plot(LineSegmentPart::Middle, ipart(intercept_y) + 1, x, fpart(intercept_y));
            intercept_y = intercept_y + gradient;
        }
    } else {
        for x in (x_pixel1 + 1.0) as i32..x_pixel2 as i32 {
            plot(LineSegmentPart::Middle, x, ipart(intercept_y), rfpart(intercept_y));
            plot(LineSegmentPart::Middle, x, ipart(intercept_y) + 1, fpart(intercept_y));
            intercept_y = intercept_y + gradient;
        }
    }
}

pub fn draw_line_anti_aliased_thick<T: ImageOperationSource>(update_op: &mut T,
                                                             x1: i32, y1: i32,
                                                             x2: i32, y2: i32,
                                                             side_half_width: i32,
                                                             color: Color,
                                                             undo: bool,
                                                             undo_image: &mut SparseImage) {
    if side_half_width > 0 {
        let x1 = x1 as f32;
        let y1 = y1 as f32;
        let x2 = x2 as f32;
        let y2 = y2 as f32;

        let dx = x2 - x1;
        let dy = y2 - y1;
        let norm = ((dx * dx + dy * dy) as f32).sqrt();
        let dx = dx / norm;
        let dy = dy / norm;

        let dx_perp = dy;
        let dy_perp = -dx;

        let mut drawn_pixels = HashSet::new();
        for width in 0..(side_half_width + 1) {
            let mut set_pixel = |x, y, part: LineSegmentPart| {
                match part {
                    LineSegmentPart::Start | LineSegmentPart::End => (true, true),
                    LineSegmentPart::Middle => {
                        if drawn_pixels.insert((x, y)) {
                            (true, width == side_half_width)
                        } else {
                            (false, false)
                        }
                    }
                }
            };

            if width != 0 {
                let width = width as f32;
                draw_line_anti_aliased_f32(update_op, x1 - dx_perp * width, y1 - dy_perp * width, x2 - dx_perp * width, y2 - dy_perp * width, color, &mut set_pixel, undo, undo_image);
                draw_line_anti_aliased_f32(update_op, x1 + dx_perp * width, y1 + dy_perp * width, x2 + dx_perp * width, y2 + dy_perp * width, color, &mut set_pixel, undo, undo_image);
            } else {
                draw_line_anti_aliased_f32(update_op, x1, y1, x2, y2, color, &mut set_pixel, undo, undo_image);
            }
        }
    } else {
        draw_line_anti_aliased(update_op, x1, y1, x2, y2, color, undo, undo_image);
    }
}

pub fn pencil_stroke_anti_aliased<T: ImageOperationSource>(update_op: &mut T,
                                                           x1: i32, y1: i32,
                                                           x2: i32, y2: i32,
                                                           prev_x1: Option<i32>, prev_y1: Option<i32>,
                                                           side_half_width: i32,
                                                           color: Color,
                                                           undo: bool,
                                                           undo_image: &mut SparseImage) {
    if side_half_width > 0 {
        fn calculate_gradient(x1: f32, y1: f32, x2: f32, y2: f32) -> (f32, f32) {
            let dx = x2 - x1;
            let dy = y2 - y1;
            let norm = ((dx * dx + dy * dy) as f32).sqrt();
            if norm < 1E-6 {
                return (0.0, 0.0);
            }

            let dx = dx / norm;
            let dy = dy / norm;
            (dx, dy)
        }

        let x1 = x1 as f32;
        let y1 = y1 as f32;
        let x2 = x2 as f32;
        let y2 = y2 as f32;

        let (dx, dy) = calculate_gradient(x1, y1, x2, y2);
        let (mut prev_dx, mut prev_dy) = match (prev_x1, prev_y1) {
            (Some(prev_x1), Some(prev_y1)) => calculate_gradient(prev_x1 as f32, prev_y1 as f32, x1, y1),
            _ => (dx, dy)
        };

        if prev_dx == 0.0 && prev_dy == 0.0 {
            prev_dx = dx;
            prev_dy = dy;
        }

        let dx_perp = dy;
        let dy_perp = -dx;
        let prev_dx_perp = prev_dy;
        let prev_dy_perp = -prev_dx;

        for width in 0..(side_half_width + 1) {
            let mut set_pixel = |_, _, _| (true, width == side_half_width);

            if width != 0 {
                let width = width as f32;
                draw_line_anti_aliased_f32(update_op, x1 - prev_dx_perp * width, y1 - prev_dy_perp * width, x2 - dx_perp * width, y2 - dy_perp * width, color, &mut set_pixel, undo, undo_image);
                draw_line_anti_aliased_f32(update_op, x1 + prev_dx_perp * width, y1 + prev_dy_perp * width, x2 + dx_perp * width, y2 + dy_perp * width, color, &mut set_pixel, undo, undo_image);
            } else {
                draw_line_anti_aliased_f32(update_op, x1, y1, x2, y2, color, &mut set_pixel, undo, undo_image);
            }
        }
    } else {
        draw_line_anti_aliased(update_op, x1, y1, x2, y2, color, undo, undo_image);
    }
}

pub fn draw_circle<F: FnMut(i32, i32)>(center_x: i32, center_y: i32, radius: i32, filled: bool, mut set_pixel: F) {
    let mut line_drawn = HashSet::new();
    let mut draw_line_guard = |set_pixel: &mut F, x1: i32, y1: i32, x2: i32, y2: i32| {
        if line_drawn.insert(y1) {
            draw_line(x1, y1, x2, y2, |x, y, _| set_pixel(x, y));
        }
    };

    let mut draw = |x: i32, y: i32| {
        if filled {
            draw_line_guard(&mut set_pixel, center_x - x, center_y + y, center_x + x, center_y + y);
            draw_line_guard(&mut set_pixel, center_x - x, center_y - y, center_x + x, center_y - y);
            draw_line_guard(&mut set_pixel, center_x - y, center_y + x, center_x + y, center_y + x);
            draw_line_guard(&mut set_pixel, center_x - y, center_y - x, center_x + y, center_y - x);
        } else {
            set_pixel(center_x - x, center_y + y);
            set_pixel(center_x + x, center_y + y);

            set_pixel(center_x - x, center_y - y);
            set_pixel(center_x + x, center_y - y);

            set_pixel(center_x - y, center_y + x);
            set_pixel(center_x + y, center_y + x);

            set_pixel(center_x - y, center_y - x);
            set_pixel(center_x + y, center_y - x);
        }
    };

    let mut x = 0;
    let mut y = radius;
    let mut d = 3 - 2 * radius;
    draw(x, y);
    while y >= x {
        x += 1;

        if d > 0 {
            y -= 1;
            d += 4 * (x - y) + 10;
        } else {
            d += 4 * x + 6;
        }

        draw(x, y);
    }
}

pub fn draw_circle_anti_aliased<T: ImageOperationSource>(update_op: &mut T,
                                                         center_x: i32, center_y: i32,
                                                         radius: i32,
                                                         color: Color,
                                                         blend: bool,
                                                         undo: bool,
                                                         undo_image: &mut SparseImage) {
    let mut set_pixel = |x: i32, y: i32, color: Color| {
        if !(x >= 0 && x < update_op.width() as i32 && y >= 0 && y < update_op.height() as i32) {
            return;
        }

        let pixel = update_op.get_pixel(x as u32, y as u32);
        if undo && !undo_image.contains_key(&(x as u32, y as u32)) {
            undo_image.insert((x as u32, y as u32), pixel);
        }

        update_op.put_pixel_with_blend(x as u32, y as u32, color);
    };

    let mut draw = |x: i32, y: i32, alpha: f32| {
        let color = if blend {
            let mut color = color;
            color[3] = alpha.clamp(0.0, 255.0) as u8;
            color
        } else {
            color
        };

        set_pixel(center_x - x, center_y + y, color);
        set_pixel(center_x + x, center_y + y, color);

        set_pixel(center_x - x, center_y - y, color);
        set_pixel(center_x + x, center_y - y, color);

        set_pixel(center_x - y, center_y + x, color);
        set_pixel(center_x + y, center_y + x, color);

        set_pixel(center_x - y, center_y - x, color);
        set_pixel(center_x + y, center_y - x, color);
    };

    let mut i = 0;
    let mut j = radius;
    let mut last_fade_amount = 0.0;

    while i < j {
        let height = ((radius * radius - (i * i)).max(0) as f32).sqrt();
        let fade_amount = 255.0 * (height.ceil() - height);

        if fade_amount < last_fade_amount {
            j -= 1;
        }
        last_fade_amount = fade_amount;

        let fade_amount = fade_amount.floor();
        draw(i, j, 255.0 - fade_amount);
        draw(i, j - 1, fade_amount);
        i += 1;
    }
}

pub fn draw_circle_anti_aliased_thick<T: ImageOperationSource>(update_op: &mut T,
                                                               center_x: i32, center_y: i32,
                                                               radius: i32,
                                                               border_half_width: i32,
                                                               color: Color,
                                                               undo: bool,
                                                               undo_image: &mut SparseImage) {
    if border_half_width > 0 {
        for radius_offset in -border_half_width..(border_half_width + 1) {
            draw_circle_anti_aliased(
                update_op,
                center_x,
                center_y,
                radius + (border_half_width - 1) + radius_offset,
                color,
                radius_offset.abs() == border_half_width,
                undo,
                undo_image
            );
        }
    } else {
        draw_circle_anti_aliased(
            update_op,
            center_x,
            center_y,
            radius,
            color,
            true,
            undo,
            undo_image
        );
    }
}

pub fn fill_rectangle<F: FnMut(i32, i32)>(min_x: i32, min_y: i32, max_x: i32, max_y: i32, mut set_pixel: F) {
    for y in min_y..max_y + 1 {
        for x in min_x..max_x + 1 {
            set_pixel(x, y);
        }
    }
}

pub fn bucket_fill<T: ImageOperationSource>(update_op: &mut T,
                                            start_x: i32, start_y: i32,
                                            fill_color: Color,
                                            tolerance: f32,
                                            undo: bool,
                                            undo_image: &mut OptionalImage) {
    let width = update_op.width() as i32;
    let height = update_op.height() as i32;

    if start_x >= 0 && start_x < width && start_y >= 0 && start_y < height {
        let ref_color = update_op.get_pixel(start_x as u32, start_y as u32);

        let mut stack = Vec::new();
        stack.push((start_x, start_y, ref_color));

        let mut visited = vec![false; (update_op.width() * update_op.height()) as usize];
        while let Some((x, y, color)) = stack.pop() {
            if undo && !undo_image.contains_key(&(x as u32, y as u32)) {
                undo_image.insert((x as u32, y as u32), color);
            }

            update_op.put_pixel_with_blend(x as u32, y as u32, fill_color);
            visited[(y * width + x) as usize] = true;

            for ny in (y - 1)..(y + 2) {
                for nx in (x - 1)..(x + 2) {
                    if nx >= 0 && nx < width && ny >= 0 && ny < height {
                        if !visited[(ny * width + nx) as usize] {
                            let color = update_op.get_pixel(nx as u32, ny as u32);
                            if color_within_tolerance(&ref_color, tolerance, &color) {
                                stack.push((nx, ny, color));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn color_gradient<T: ImageOperationSource>(update_op: &mut T,
                                               start_x: i32, start_y: i32,
                                               end_x: i32, end_y: i32,
                                               first_color: Color,
                                               second_color: Color,
                                               gradient_type: ColorGradientType) {
    let first_color = Vector4::new(first_color[0] as f32, first_color[1] as f32, first_color[2] as f32, first_color[3] as f32);
    let second_color = Vector4::new(second_color[0] as f32, second_color[1] as f32, second_color[2] as f32, second_color[3] as f32);

    let calc_distance = |x: i32, y: i32| {
        match gradient_type {
            ColorGradientType::Linear => {
                (-(end_y - start_y) * (start_y - y) - (start_x - x) * (end_x - start_x)).abs() as f32
                / (((end_x - start_x).pow(2) + (end_y - start_y).pow(2)) as f32).sqrt()
            }
            ColorGradientType::Radial => {
                (((x - start_x).pow(2) + (y - start_y).pow(2)) as f32).sqrt()
            }
        }
    };

    let max_distance = match gradient_type {
        ColorGradientType::Linear => (((end_x - start_x).pow(2) + (end_y - start_y).pow(2)) as f32).sqrt(),
        ColorGradientType::Radial => calc_distance(end_x, end_y)
    };

    for y in 0..update_op.height() {
        for x in 0..update_op.width() {
            let distance = calc_distance(x as i32, y as i32);

            let factor = distance / max_distance;
            let color = factor * first_color + (1.0 - factor) * second_color;
            update_op.put_pixel_with_blend(
                x,
                y,
                image::Rgba([
                    color.x.clamp(0.0, 255.0) as u8,
                    color.y.clamp(0.0, 255.0) as u8,
                    color.z.clamp(0.0, 255.0) as u8,
                    color.w.clamp(0.0, 255.0) as u8
                ])
            );
        }
    }
}

fn color_within_tolerance(ref_color: &Color, tolerance: f32, color: &Color) -> bool {
    if color == &image::Rgba([0, 0, 0, 0]) {
        return true;
    }

    let ref_color = cgmath::Vector3::new(ref_color[0], ref_color[1], ref_color[2]);
    let color = cgmath::Vector3::new(color[0], color[1], color[2]);

    if tolerance == 0.0 {
        return ref_color == color;
    }

    let ref_color = cgmath::Vector3::new(ref_color[0] as f32, ref_color[1] as f32, ref_color[2] as f32);
    let color = cgmath::Vector3::new(color[0] as f32, color[1] as f32, color[2] as f32);

    let diff = (ref_color - color).div_element_wise(cgmath::Vector3::new(255.0, 255.0, 255.0));
    ((diff.x.abs() + diff.y.abs() + diff.z.abs()) / 3.0) <= tolerance
}

pub fn sub_image<T: ImageSource>(image: &T, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> image::RgbaImage {
    let mut sub_image: image::RgbaImage = image::RgbaImage::new(
        (max_x - min_x).max(0) as u32,
        (max_y - min_y).max(0) as u32
    );

    for y in min_y..max_y {
        for x in min_x..max_x {
            if x >= 0 && x < image.width() as i32 && y >= 0 && y < image.height() as i32 {
                let sub_x = x - min_x;
                let sub_y = y - min_y;

                if sub_x >= 0 && sub_x < sub_image.width() as i32 && sub_y >= 0 && sub_y < sub_image.height() as i32 {
                    sub_image.put_pixel(sub_x as u32, sub_y as u32, image.get_pixel(x as u32, y as u32));
                }
            }
        }
    }

    sub_image
}

pub fn rotate_image(image: &image::RgbaImage, rotation: f32, filter_type: FilterType) -> image::RgbaImage {
    let rotated_image_size = image.width() + image.height();
    let mut rotated_image: image::RgbaImage = image::RgbaImage::new(
        rotated_image_size,
        rotated_image_size
    );

    let center_x = (image.width() as f32 * 0.5).floor();
    let center_y = (image.height() as f32 * 0.5).floor();
    let target_center_x = (rotated_image.width() as f32 * 0.5).floor();
    let target_center_y = (rotated_image.height() as f32 * 0.5).floor();

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    let mut transform = Matrix3::<f32>::from_angle_z(cgmath::Rad(-rotation));
    transform.z.x = center_x;
    transform.z.y = center_y;

    for y in 0..rotated_image.height() {
        for x in 0..rotated_image.width() {
            let source = transform.transform_point(Point2::new(x as f32 - target_center_x,
                                                               y as f32 - target_center_y));
            let source_fx = source.x;
            let source_fy = source.y;
            let source_x = source_fx.round() as i32;
            let source_y = source_fy.round() as i32;

            if source_x >= 0 && source_x < image.width() as i32 && source_y >= 0 && source_y < image.height() as i32 {
                min_x = min_x.min(x as i32);
                min_y = min_y.min(y as i32);
                max_x = max_x.max(x as i32);
                max_y = max_y.max(y as i32);

                let target = match filter_type {
                    FilterType::Nearest => {
                        *image.get_pixel(source_x as u32, source_y as u32)
                    }
                    FilterType::Triangle => {
                        let source_x1 = source_fx.floor() as i32;
                        let mut source_x2 = source_fx.ceil() as i32;
                        if source_x1 == source_x2 {
                            source_x2 += 1;
                        }

                        let source_x1 = source_x1.clamp(0, image.width() as i32 - 1);
                        let source_x2 = source_x2.clamp(0, image.width() as i32 - 1);

                        let source_y1 = source_fy.floor() as i32;
                        let mut source_y2 = source_fy.ceil() as i32;
                        if source_y1 == source_y2 {
                            source_y2 += 1;
                        }

                        let source_y1 = source_y1.clamp(0, image.height() as i32 - 1);
                        let source_y2 = source_y2.clamp(0, image.height() as i32 - 1);

                        let target11 = pixel_to_vec(image.get_pixel(source_x1 as u32, source_y1 as u32));
                        let target21 = pixel_to_vec(image.get_pixel(source_x2 as u32, source_y1 as u32));
                        let target12 = pixel_to_vec(image.get_pixel(source_x1 as u32, source_y2 as u32));
                        let target22 = pixel_to_vec(image.get_pixel(source_x2 as u32, source_y2 as u32));

                        let source_fx1 = source_x1 as f32;
                        let source_fx2 = source_x2 as f32;
                        let source_fy1 = source_y1 as f32;
                        let source_fy2 = source_y2 as f32;

                        let range_x = source_fx2 - source_fx1;
                        let range_y = source_fy2 - source_fy1;

                        if source_x2 - source_x1 > 0 && source_y2 - source_y1 > 0 {
                            let factor1 = (source_fx2 - source_fx) / range_x;
                            let factor2 = (source_fx - source_fx1) / range_x;

                            let interpolate1 = factor1 * target11 + factor2 * target21;
                            let interpolate2 = factor1 * target12 + factor2 * target22;

                            let target = ((source_fy2 - source_fy) / range_y) * interpolate1
                                          + ((source_fy - source_fy1) / range_y) * interpolate2;
                            vec_to_pixel(&target)
                        } else {
                            *image.get_pixel(source_x as u32, source_y as u32)
                        }
                    }
                    _ => { panic!("Unsupported filter type.") }
                };

                rotated_image.put_pixel(x, y, target);
            }
        }
    }

    rotated_image

    // Shrink to fit
    // let mut sub_image: image::RgbaImage = image::RgbaImage::new(
    //     (max_x - min_x) as u32 + 1,
    //     (max_y - min_y) as u32 + 1
    // );
    //
    // for y in min_y..(max_y + 1) {
    //     for x in min_x..(max_x + 1) {
    //         let sub_x = x - min_x;
    //         let sub_y = y - min_y;
    //
    //         if sub_x >= 0 && sub_x < sub_image.width() as i32 && sub_y >= 0 && sub_y < sub_image.height() as i32 {
    //             sub_image.put_pixel(sub_x as u32, sub_y as u32, *rotated_image.get_pixel(x as u32, y as u32));
    //         }
    //     }
    // }
    //
    // sub_image
}

pub fn symmetric_round(x: f32) -> f32 {
    x.abs().round() * x.signum()
}

pub fn symmetric_floor(x: f32) -> f32 {
    x.abs().floor() * x.signum()
}

pub fn hsv_to_rgb(hue: f64, saturation: f64, value: f64) -> Option<Color> {
    if hue > 360.0 || hue < 0.0 || saturation > 100.0 || saturation < 0.0 || value > 100.0 || value < 0.0 {
        return None;
    }

    let s_scaled = saturation / 100.0;
    let v_scaled = value / 100.0;
    let c = s_scaled * v_scaled;
    let x = c * (1.0 - (fmod(hue / 60.0, 2.0) - 1.0).abs());
    let m = v_scaled - c;

    let r;
    let g;
    let b;
    if hue >= 0.0 && hue < 60.0 {
        r = c;
        g = x;
        b = 0.0;
    } else if hue >= 60.0 && hue < 120.0 {
        r = x;
        g = c;
        b = 0.0;
    } else if hue >= 120.0 && hue < 180.0 {
        r = 0.0;
        g = c;
        b = x;
    } else if hue >= 180.0 && hue < 240.0 {
        r = 0.0;
        g = x;
        b = c;
    } else if hue >= 240.0 && hue < 300.0 {
        r = x;
        g = 0.0;
        b = c;
    } else {
        r = c;
        g = 0.0;
        b = x;
    }

    Some(
        image::Rgba([
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
            255
        ])
    )
}

pub fn rgb_to_hsv(color: Color) -> (f64, f64, f64) {
    let r = color[0] as f64 / 255.0;
    let g = color[1] as f64 / 255.0;
    let b = color[2] as f64 / 255.0;

    let max_rgb = r.max(g).max(b);
    let min_rgb = r.min(g).min(b);
    let rgb_delta = max_rgb - min_rgb;

    let mut hue = 0.0;
    let saturation;
    let value;

    if rgb_delta > 0.0 {
        if max_rgb == r {
            hue = 60.0 * fmod((g - b) / rgb_delta, 6.0);
        } else if max_rgb == g {
            hue = 60.0 * (((b - r) / rgb_delta) + 2.0);
        } else if max_rgb == b {
            hue = 60.0 * (((r - g) / rgb_delta) + 4.0);
        }

        if max_rgb > 0.0 {
            saturation = rgb_delta / max_rgb;
        } else {
            saturation = 0.0;
        }

        value = max_rgb;
    } else {
        hue = 0.0;
        saturation = 0.0;
        value = max_rgb;
    }

    if hue < 0.0 {
        hue = 360.0 + hue;
    }

    (hue, saturation * 100.0, value * 100.0)
}

fn alpha_blend(a: Color, b: Color) -> Color {
    let mut b = b;
    b.blend(&a);
    b
}

fn fmod(numer: f64, denom: f64) -> f64 {
    let rquot: f64 = (numer / denom).floor();
    numer - rquot * denom
}

fn pixel_to_vec(pixel: &image::Rgba<u8>) -> Vector4<f32> {
    Vector4::new(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32, pixel[3] as f32)
}

fn vec_to_pixel(vec: &Vector4<f32>) -> image::Rgba<u8> {
    image::Rgba([
        vec[0].clamp(0.0, 255.0) as u8,
        vec[1].clamp(0.0, 255.0) as u8,
        vec[2].clamp(0.0, 255.0) as u8,
        vec[3].clamp(0.0, 255.0) as u8
    ])
}