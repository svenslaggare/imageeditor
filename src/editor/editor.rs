use crate::editor::image_operation::{ImageOperation, ImageOperationMarker, ImageSource};
use crate::editor::Image;

pub struct Editor {
    image: Image,
    undo_stack: Vec<(ImageOperation, ImageOperation)>,
    redo_stack: Vec<ImageOperation>,
}

impl Editor {
    pub fn new(image: Image) -> Editor {
        Editor {
            image,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn new_image_same(&self) -> Image {
        Image::new(image::RgbaImage::new(self.image.width(), self.image.height()))
    }

    fn merge_draw_operations(&mut self) {
        for i in (0..self.undo_stack.len()).rev() {
            let (op, _) = &self.undo_stack[i];
            if op.is_marker(ImageOperationMarker::BeginDraw) {
                let (ops, mut undo_ops): (Vec<ImageOperation>, Vec<ImageOperation>) = self.undo_stack.drain(i..).unzip();
                undo_ops.reverse();
                self.undo_stack.push((ImageOperation::Sequential(ops).remove_markers(), ImageOperation::Sequential(undo_ops)));
                break;
            }
        }
    }

    fn internal_apply_op(&mut self, op: ImageOperation) {
        if !op.is_marker(ImageOperationMarker::EndDraw) {
            let mut update_op = self.image.update_operation();
            if let Some(undo_op) = op.apply(&mut update_op, true) {
                self.undo_stack.push((op, undo_op));
            }
        } else {
            self.merge_draw_operations();
        }
    }

    pub fn apply_op(&mut self, op: ImageOperation) {
        self.internal_apply_op(op);
        self.redo_stack.clear();
    }

    pub fn undo_op(&mut self) {
        if let Some((orig_op, undo)) = self.undo_stack.pop() {
            let mut update_op = self.image.update_operation();
            undo.apply(&mut update_op, false);
            self.redo_stack.push(orig_op);
        }
    }

    pub fn redo_op(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            self.internal_apply_op(op);
        }
    }
}