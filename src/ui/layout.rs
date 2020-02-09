use crate::rendering::prelude::Position;

struct AdaptiveRows {
    origin: Position,
    cell_size: (f32, f32),
    col_width: f32,
    num_elements: usize,

    num_processed: usize,
    current_position: Position
}

impl Iterator for AdaptiveRows {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_processed >= self.num_elements {
            return None;
        }

        let position = self.current_position;

        self.num_processed += 1;
        self.current_position.x += self.cell_size.0;
        if (self.current_position.x - self.origin.x) >= self.col_width {
            self.current_position.x = self.origin.x;
            self.current_position.y += self.cell_size.1;
        }

        Some(position)
    }
}

pub fn adaptive_rows(origin: Position, cell_size: (f32, f32), col_width: f32, num_elements: usize) -> impl Iterator<Item=Position> {
    AdaptiveRows {
        origin,
        cell_size,
        col_width,
        num_elements,
        num_processed: 0,
        current_position: origin
    }
}

struct AdaptiveCols {
    origin: Position,
    cell_size: (f32, f32),
    row_height: f32,
    num_elements: usize,

    num_processed: usize,
    current_position: Position
}

impl Iterator for AdaptiveCols {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_processed >= self.num_elements {
            return None;
        }

        let position = self.current_position;

        self.num_processed += 1;
        self.current_position.y += self.cell_size.1;
        if (self.current_position.y - self.origin.y) >= self.row_height {
            self.current_position.x += self.cell_size.0;
            self.current_position.y = self.origin.y;
        }

        Some(position)
    }
}

pub fn adaptive_cols(origin: Position, cell_size: (f32, f32), row_height: f32, num_elements: usize) -> impl Iterator<Item=Position> {
    AdaptiveCols {
        origin,
        cell_size,
        row_height,
        num_elements,
        num_processed: 0,
        current_position: origin
    }
}