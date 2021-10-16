pub type Color = cgmath::Point3<u8>;
pub type Position = cgmath::Point2<f32>;
pub type Size = cgmath::Point2<f32>;

#[derive(Clone, Debug)]
pub struct Rectangle {
    pub position: cgmath::Point2<f32>,
    pub size: cgmath::Point2<f32>
}

impl Rectangle {
    pub fn new(position_x: f32, position_y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle {
            position: cgmath::Point2::new(position_x, position_y),
            size: cgmath::Point2::new(width, height),
        }
    }

    pub fn from_min_and_max(min_position: &Position, max_position: &Position) -> Rectangle {
        Rectangle::new(
            min_position.x,
            min_position.y,
            max_position.x - min_position.x,
            max_position.y - min_position.y
        )
    }

    pub fn top(&self) -> f32 {
        return self.position.y;
    }

    pub fn bottom(&self) -> f32 {
        return self.position.y + self.size.y;
    }

    pub fn left(&self) -> f32 {
        return self.position.x;
    }

    pub fn right(&self) -> f32 {
        return self.position.x + self.size.x;
    }

    pub fn contains(&self, position: &Position) -> bool {
        return position.x >= self.left() && position.x <= self.right() && position.y >= self.top() && position.y <= self.bottom();
    }
}