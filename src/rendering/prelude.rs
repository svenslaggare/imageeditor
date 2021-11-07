use image::Pixel;

pub type Color = cgmath::Point3<u8>;
pub type Color4 = cgmath::Vector4<u8>;
pub type Position = cgmath::Point2<f32>;
pub type Size = cgmath::Point2<f32>;

#[derive(Clone, Debug)]
pub struct Rectangle {
    pub position: Position,
    pub size: Size
}

impl Rectangle {
    pub fn new(position_x: f32, position_y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle {
            position: cgmath::Point2::new(position_x, position_y),
            size: cgmath::Point2::new(width, height),
        }
    }

    pub fn from_position_and_size(position: Position, size: Size) -> Rectangle {
        Rectangle {
            position,
            size
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
        return position.x >= self.left() && position.x < self.right() && position.y >= self.top() && position.y < self.bottom();
    }
}

pub fn blend(a: &Color4, b: &Color4) -> Color4 {
    let mut a = image::Rgba([a.x, a.y, a.z, a.w]);
    let b = image::Rgba([b.x, b.y, b.z, b.w]);
    a.blend(&b);
    Color4::new(a[0], a[1], a[2], a[3])
}