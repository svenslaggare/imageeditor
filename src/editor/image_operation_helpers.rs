use crate::editor::image_operation::{ImageSource, ImageOperationSource, SparseImage};
use crate::editor::Color;

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

    let mut x = 0;
    let mut y = 0;

    if dy1 <= dx1 {
        let mut end_x = 0;

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
        let mut end_y = 0;

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
