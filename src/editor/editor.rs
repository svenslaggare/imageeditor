use crate::editor::image_operation::{ImageOperation, ImageOperationMarker, ImageSource, ImageOperationSource};
use crate::editor::Image;

pub struct LayeredImage {
    layers: Vec<Image>
}

impl LayeredImage {
    pub fn new(image: Image) -> LayeredImage {
        let mut blank_image = Image::new(image::RgbaImage::new(image.width(), image.height()));
        {
            let mut update_op = blank_image.update_operation();

            for y in 50..60 {
                for x in 50..60 {
                    update_op.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
                }
            }
        }

        LayeredImage {
            layers: vec![image, blank_image]
        }
    }

    pub fn width(&self) -> u32 {
        self.layers[0].width()
    }

    pub fn height(&self) -> u32 {
        self.layers[0].height()
    }

    pub fn layers(&self) -> &Vec<Image> {
        &self.layers
    }

    pub fn get_layer(&self, layer: usize) -> Option<&Image> {
        self.layers.get(layer)
    }

    pub fn get_layer_mut(&mut self, layer: usize) -> Option<&mut Image> {
        self.layers.get_mut(layer)
    }

    pub fn add_layer(&mut self) {
        self.layers.push(Image::new(image::RgbaImage::new(self.width(), self.height())));
    }
}

pub type LayeredImageOperation = (usize, ImageOperation);

pub struct Editor {
    image: LayeredImage,
    active_layer: usize,
    undo_stack: Vec<(LayeredImageOperation, LayeredImageOperation)>,
    redo_stack: Vec<LayeredImageOperation>,
}

impl Editor {
    pub fn new(image: Image) -> Editor {
        Editor {
            image: LayeredImage::new(image),
            active_layer: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn image(&self) -> &LayeredImage {
        &self.image
    }

    pub fn image_mut(&mut self) -> &mut LayeredImage {
        &mut self.image
    }

    pub fn new_image_same(&self) -> Image {
        Image::new(image::RgbaImage::new(self.image.width(), self.image.height()))
    }

    pub fn set_image(&mut self, image: Image) {
        self.image = LayeredImage::new(image);
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn apply_op(&mut self, op: ImageOperation) {
        self.internal_apply_op((self.active_layer, op));
        self.redo_stack.clear();
    }

    pub fn undo_op(&mut self) {
        if let Some((orig_op, undo)) = self.undo_stack.pop() {
            let mut update_op = self.image.get_layer_mut(undo.0).unwrap().update_operation();
            undo.1.apply(&mut update_op, false);
            self.redo_stack.push(orig_op);
        }
    }

    pub fn redo_op(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            self.internal_apply_op(op);
        }
    }

    pub fn active_layer(&self) -> &Image {
        self.image.get_layer(self.active_layer).unwrap()
    }

    pub fn next_layer(&mut self) {
        self.active_layer = (self.active_layer + 1) % self.image.layers.len();
    }

    pub fn prev_layer(&mut self) {
        if self.active_layer == 0 {
            self.active_layer = self.image.layers.len() - 1;
            return;
        }

        self.active_layer = (self.active_layer - 1) % self.image.layers.len();
    }

    pub fn is_active_layer(&self, layer: usize) -> bool {
        self.active_layer == layer
    }

    fn merge_draw_operations(&mut self) {
        for i in (0..self.undo_stack.len()).rev() {
            let (op, _) = &self.undo_stack[i];
            if op.1.is_marker(ImageOperationMarker::BeginDraw) {
                let op_layer = op.0;
                let (ops, mut undo_ops): (Vec<ImageOperation>, Vec<ImageOperation>) = self.undo_stack.drain(i..).map(|(x, y)| (x.1, y.1)).unzip();
                undo_ops.reverse();
                self.undo_stack.push(((op_layer, ImageOperation::Sequential(ops).remove_markers()), (op_layer, ImageOperation::Sequential(undo_ops))));
                break;
            }
        }
    }

    fn internal_apply_op(&mut self, op: LayeredImageOperation) {
        if !op.1.is_marker(ImageOperationMarker::EndDraw) {
            let op_layer = op.0;
            let mut update_op = self.image.get_layer_mut(op_layer).unwrap().update_operation();
            if let Some(undo_op) = op.1.apply(&mut update_op, true) {
                self.undo_stack.push((op, (op_layer, undo_op)));
            }
        } else {
            self.merge_draw_operations();
        }
    }
}