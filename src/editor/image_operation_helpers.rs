use crate::editor::image_operation::{ImageSource, ImageOperationSource, SparseImage};
use crate::editor::Color;
use std::collections::{HashSet, VecDeque};
use crate::helpers::TimeMeasurement;

pub fn draw_block<T: ImageOperationSource>(update_op: &mut T,
                                           center_x: i32,
                                           center_y: i32,
                                           side_half_width: i32,
                                           color: Color,
                                           undo: bool,
                                           undo_image: &mut SparseImage) {
    for y in (center_y - side_half_width)..(center_y + side_half_width + 1) {
        for x in (center_x - side_half_width)..(center_x + side_half_width + 1) {
            if x >= 0 && x < update_op.width() as i32 && y >= 0 && y < update_op.height() as i32 {
                if undo && !undo_image.contains_key(&(x as u32, y as u32)) {
                    undo_image.insert((x as u32, y as u32), update_op.get_pixel(x as u32, y as u32));
                }

                update_op.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

pub fn draw_line<F: FnMut(i32, i32)>(x1: i32, y1: i32, x2: i32, y2: i32, mut set_pixel: F) {
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

        set_pixel(x, y);

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
            set_pixel(x, y);
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

        set_pixel(x, y);

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
            set_pixel(x, y);
        }
    }
}

pub fn draw_circle<F: FnMut(i32, i32)>(center_x: i32, center_y: i32, radius: i32, filled: bool, mut set_pixel: F) {
    let mut draw_circle_internal = |xc: i32, yc: i32, x: i32, y: i32| {
        if filled {
            draw_line(xc - x, yc + y, xc + x, yc + y, |x, y| set_pixel(x, y));
            draw_line(xc - x, yc - y, xc + x, yc - y, |x, y| set_pixel(x, y));
            draw_line(xc - y, yc + x, xc + y, yc + x, |x, y| set_pixel(x, y));
            draw_line(xc - y, yc - x, xc + y, yc - x, |x, y| set_pixel(x, y));
        } else {
            set_pixel(xc - x, yc + y);
            set_pixel(xc + x, yc + y);

            set_pixel(xc - x, yc - y);
            set_pixel(xc + x, yc - y);

            set_pixel(xc - y, yc + x);
            set_pixel(xc + y, yc + x);

            set_pixel(xc - y, yc - x);
            set_pixel(xc + y, yc - x);
        }
    };

    let mut x = 0;
    let mut y = radius;
    let mut d = 3 - 2 * radius;
    draw_circle_internal(center_x, center_y, x, y);
    while y >= x {
        x += 1;

        if d > 0 {
            y -= 1;
            d += 4 * (x - y) + 10;
        } else {
            d += 4 * x + 6;
        }

        draw_circle_internal(center_x, center_y, x, y);
    }
}

pub fn fill_rectangle<F: FnMut(i32, i32)>(min_x: i32, min_y: i32, max_x: i32, max_y: i32, mut set_pixel: F) {
    for y in min_y..max_y {
        for x in min_x..max_x {
            set_pixel(x, y);
        }
    }
}

pub fn bucket_fill<T: ImageOperationSource>(update_op: &mut T,
                                            start_x: i32, start_y: i32,
                                            fill_color: Color,
                                            undo: bool,
                                            undo_image: &mut SparseImage) {
    let _tm = TimeMeasurement::new("bucket fill");
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

            update_op.put_pixel(x as u32, y as u32, fill_color);
            visited[(y * width + x) as usize] = true;

            for ny in (y - 1)..(y + 2) {
                for nx in (x - 1)..(x + 2) {
                    if nx >= 0 && nx < width && ny >= 0 && ny < height {
                        if !visited[(ny * width + nx) as usize] {
                            let color = update_op.get_pixel(nx as u32, ny as u32);
                            if color == ref_color {
                                stack.push((nx, ny, color));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn sub_image<T: ImageSource>(image: &T, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> image::RgbaImage {
    let min_x = std::cmp::max(min_x, 0);
    let min_y = std::cmp::max(min_y, 0);
    let max_x = std::cmp::min(max_x, image.width() as i32);
    let max_y = std::cmp::min(max_y, image.height() as i32);

    let mut sub_image = image::RgbaImage::new((max_x - min_x) as u32, (max_y - min_y) as u32);
    for y in min_y..max_y {
        for x in min_x..max_x {
            sub_image.put_pixel(
                (x - min_x) as u32,
                (y - min_y) as u32,
                image.get_pixel(x as u32, y as u32));
        }
    }

    sub_image
}

pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Option<Color> {
    if h > 360.0 || h < 0.0 || s > 100.0 || s < 0.0 || v > 100.0 || v < 0.0 {
        return None;
    }

    let s_scaled = s / 100.0;
    let v_scaled = v / 100.0;
    let c = s_scaled * v_scaled;
    let x = c * (1.0 - (fmod(h / 60.0, 2.0) - 1.0).abs());
    let m = v_scaled - c;

    let r;
    let g;
    let b;
    if h >= 0.0 && h < 60.0 {
        r = c;
        g = x;
        b = 0.0;
    } else if h >= 60.0 && h < 120.0 {
        r = x;
        g = c;
        b = 0.0;
    } else if h >= 120.0 && h < 180.0 {
        r = 0.0;
        g = c;
        b = x;
    } else if h >= 180.0 && h < 240.0 {
        r = 0.0;
        g = x;
        b = c;
    } else if h >= 240.0 && h < 300.0 {
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

fn fmod(numer: f64, denom: f64) -> f64 {
    let rquot: f64 = (numer / denom).floor();
    numer - rquot * denom
}