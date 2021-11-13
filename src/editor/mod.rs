pub mod image;
pub mod editor;
pub mod image_operation_helpers;
pub mod image_operation;
pub mod tools;

pub use crate::editor::image::Image;
pub use crate::editor::image::Color;
pub use crate::editor::editor::Editor;
pub use crate::editor::editor::EditorImage;

#[derive(Clone, Debug)]
pub struct Region {
    pub position: cgmath::Point2<i32>,
    pub size: cgmath::Point2<i32>
}

impl Region {
    pub fn new(position_x: i32, position_y: i32, width: i32, height: i32) -> Region {
        Region {
            position: cgmath::Point2::new(position_x, position_y),
            size: cgmath::Point2::new(width, height),
        }
    }

    pub fn from_position_and_size(position: cgmath::Point2<i32>, size: cgmath::Point2<i32>) -> Region {
        Region {
            position,
            size
        }
    }

    pub fn top(&self) -> i32 {
        return self.position.y;
    }

    pub fn bottom(&self) -> i32 {
        return self.position.y + self.size.y;
    }

    pub fn left(&self) -> i32 {
        return self.position.x;
    }

    pub fn right(&self) -> i32 {
        return self.position.x + self.size.x;
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        return x >= self.left() && x < self.right() && y >= self.top() && y < self.bottom();
    }
}