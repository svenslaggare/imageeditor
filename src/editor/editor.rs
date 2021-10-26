use itertools::Itertools;

use crate::editor::image_operation::{ImageOperation, ImageOperationMarker, ImageSource, ImageOperationSource};
use crate::editor::Image;

#[derive(Clone, PartialEq, Debug)]
pub enum LayerState {
    Visible,
    Hidden,
    Deleted
}

pub struct LayeredImage {
    width: u32,
    height: u32,
    layers: Vec<(LayerState, Image)>
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
            width: image.width(),
            height: image.height(),
            layers: vec![(LayerState::Visible, image), (LayerState::Visible, blank_image)]
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn layers(&self) -> &Vec<(LayerState, Image)> {
        &self.layers
    }

    pub fn layers_mut(&mut self) -> &mut Vec<(LayerState, Image)> {
        &mut self.layers
    }

    pub fn get_layer(&self, layer: usize) -> Option<&Image> {
        self.layers.get(layer).map(|(_, layer)| layer)
    }

    pub fn get_layer_mut(&mut self, layer: usize) -> Option<&mut Image> {
        self.layers.get_mut(layer).map(|(_, layer)| layer)
    }

    pub fn add_layer(&mut self) {
        self.layers.push((LayerState::Visible, Image::new(image::RgbaImage::new(self.width(), self.height()))));
    }
}

#[derive(Debug)]
pub enum LayeredImageOperation {
    SetLayerState(usize, LayerState),
    SetActiveLayer(usize),
    ImageOp(usize, ImageOperation)
}

impl LayeredImageOperation {
    pub fn is_image_op(&self) -> bool {
        match self {
            LayeredImageOperation::ImageOp(_, _) => true,
            _ => false
        }
    }

    pub fn extract_image_op(self) -> Option<(usize, ImageOperation)> {
        match self {
            LayeredImageOperation::ImageOp(index, op) => Some((index, op)),
            _ => None
        }
    }
}

pub struct Editor {
    image: LayeredImage,
    active_layer_index: usize,
    undo_stack: Vec<(LayeredImageOperation, LayeredImageOperation)>,
    redo_stack: Vec<LayeredImageOperation>,
}

impl Editor {
    pub fn new(image: Image) -> Editor {
        Editor {
            image: LayeredImage::new(image),
            active_layer_index: 0,
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
        self.apply_layer_op(LayeredImageOperation::ImageOp(self.active_layer_index, op));
    }

    pub fn apply_layer_op(&mut self, op: LayeredImageOperation) {
        self.internal_apply_op(op);
        self.redo_stack.clear();
    }

    pub fn undo_op(&mut self) {
        if let Some((orig_op, undo)) = self.undo_stack.pop() {
            match orig_op {
                LayeredImageOperation::ImageOp(orig_op_layer, orig_op) => {
                    let undo = undo.extract_image_op().unwrap();
                    let mut update_op = self.image.get_layer_mut(undo.0).unwrap().update_operation();
                    undo.1.apply(&mut update_op, false);
                    self.redo_stack.push(LayeredImageOperation::ImageOp(orig_op_layer, orig_op));
                }
                orig_op => {
                    self.internal_apply_other_op(undo, false);
                    self.redo_stack.push(orig_op);
                }
            }
        }
    }

    pub fn redo_op(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            self.internal_apply_op(op);
        }
    }

    pub fn active_layer(&self) -> &Image {
        self.image.get_layer(self.active_layer_index).unwrap()
    }

    pub fn active_layer_index(&self) -> usize {
        self.active_layer_index
    }

    fn merge_draw_operations(&mut self) {
        for i in (0..self.undo_stack.len()).rev() {
            match &self.undo_stack[i].0 {
                LayeredImageOperation::ImageOp(op_layer, op) => {
                    let op_layer = *op_layer;
                    if op.is_marker(ImageOperationMarker::BeginDraw) {
                        let max_index = self.undo_stack
                            .iter()
                            .enumerate()
                            .find(|(index, (op, _))| *index >= i && !op.is_image_op())
                            .map(|(index, op)| index)
                            .unwrap_or(self.undo_stack.len());

                        let (ops, mut undo_ops): (Vec<ImageOperation>, Vec<ImageOperation>) = self.undo_stack.drain(i..max_index)
                            .map(|(x, y)| (x.extract_image_op(), y.extract_image_op()))
                            .filter(|(x, y)| x.is_some() && y.is_some())
                            .map(|(x, y)| (x.unwrap().1, y.unwrap().1))
                            .unzip();

                        undo_ops.reverse();
                        self.undo_stack.push((
                            LayeredImageOperation::ImageOp(op_layer, ImageOperation::Sequential(ops).remove_markers()),
                            LayeredImageOperation::ImageOp(op_layer, ImageOperation::Sequential(undo_ops))
                        ));
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    fn internal_apply_op(&mut self, op: LayeredImageOperation) {
        match op {
            LayeredImageOperation::ImageOp(op_layer, op) => {
                if !op.is_marker(ImageOperationMarker::EndDraw) {
                    let mut update_op = self.image.get_layer_mut(op_layer).unwrap().update_operation();
                    if let Some(undo_op) = op.apply(&mut update_op, true) {
                        self.undo_stack.push((LayeredImageOperation::ImageOp(op_layer, op),
                                              LayeredImageOperation::ImageOp(op_layer, undo_op)));
                    }
                } else {
                    self.merge_draw_operations();
                }
            }
            op => self.internal_apply_other_op(op, true)
        }
    }

    fn internal_apply_other_op(&mut self, op: LayeredImageOperation, push_undo: bool) {
        match op {
            LayeredImageOperation::SetLayerState(index, state) => {
                let current_state = self.image.layers_mut()[index].0.clone();
                self.image.layers_mut()[index].0 = state.clone();

                if push_undo {
                    self.undo_stack.push((
                        LayeredImageOperation::SetLayerState(index, state),
                        LayeredImageOperation::SetLayerState(index, current_state)
                    ));
                }
            }
            LayeredImageOperation::SetActiveLayer(layer_index) => {
                let current_active_layer_index = self.active_layer_index;
                self.active_layer_index = layer_index;

                if push_undo {
                    self.undo_stack.push((
                        LayeredImageOperation::SetActiveLayer(layer_index),
                        LayeredImageOperation::SetActiveLayer(current_active_layer_index)
                    ));
                }
            }
            LayeredImageOperation::ImageOp(_, _) => panic!("Should not be used in this way.")
        }
    }
}