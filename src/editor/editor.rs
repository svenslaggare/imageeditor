use std::path::{Path, PathBuf};
use std::fmt::{Display};

use itertools::Itertools;

use image::{GenericImage, FilterType};

use crate::editor::image_operation::{ImageOperation, ImageOperationMarker, ImageSource};
use crate::editor::{Image, Region};

#[derive(Debug, Clone)]
pub enum ImageFormat {
    Png,
    Jpeg(u8),
    Bmp,
    Tiff
}

impl ImageFormat {
    pub fn from_extension(extension: &str) -> Option<ImageFormat> {
        match extension {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg(80)),
            "bmp" => Some(ImageFormat::Bmp),
            "tif" | "tiff" => Some(ImageFormat::Tiff),
            _ => None
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum LayerState {
    Visible,
    Hidden,
    Deleted
}

#[derive(Clone, Debug)]
pub struct EditorImage {
    path: Option<PathBuf>,
    image_format: Option<ImageFormat>,
    width: u32,
    height: u32,
    layers: Vec<(LayerState, Image)>
}

impl EditorImage {
    pub fn new(path: Option<PathBuf>, image: Image) -> EditorImage {
        let image_format = path
            .as_ref().map(|path| path.extension()).flatten()
            .map(|extension| extension.to_str()).flatten()
            .map(|extension| ImageFormat::from_extension(extension)).flatten();

        EditorImage {
            path,
            image_format,
            width: image.width(),
            height: image.height(),
            layers: vec![(LayerState::Visible, image)]
        }
    }

    pub fn from_rgba(path: Option<PathBuf>, image: image::RgbaImage) -> EditorImage {
        EditorImage::new(path, Image::new(image))
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_ref().map(|path| path.as_path())
    }

    pub fn image_format(&self) -> Option<&ImageFormat> {
        self.image_format.as_ref()
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

    pub fn add_layer_with_image(&mut self, image: image::RgbaImage) {
        assert_eq!(self.width, image.width());
        assert_eq!(self.height, image.height());
        self.layers.push((LayerState::Visible, Image::new(image)));
    }

    pub fn save(&self, path: &Path, format: &ImageFormat) -> std::io::Result<()> {
        let mut image: image::RgbaImage = image::RgbaImage::new(self.width(), self.height());
        for (state, layer) in &self.layers {
            if state == &LayerState::Visible {
                let layer = layer.get_image();

                for y in 0..image.height() {
                    for x in 0..image.width() {
                        image.blend_pixel(x, y, *layer.get_pixel(x, y));
                    }
                }
            }
        }

        let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);

        match format {
            ImageFormat::Png => {
                let encoder = image::png::PNGEncoder::new(writer);
                encoder.encode(
                    &image,
                    image.width(),
                    image.height(),
                    image::ColorType::RGBA(8)
                )?;
            }
            ImageFormat::Jpeg(quality) => {
                let mut encoder = image::jpeg::JPEGEncoder::new_with_quality(&mut writer, *quality);
                encoder.encode(
                    &image,
                    image.width(),
                    image.height(),
                    image::ColorType::RGBA(8)
                )?;
            }
            ImageFormat::Bmp => {
                let mut encoder = image::bmp::BMPEncoder::new(&mut writer);
                encoder.encode(
                    &image,
                    image.width(),
                    image.height(),
                    image::ColorType::RGBA(8)
                )?;
            }
            ImageFormat::Tiff => {
                let encoder = image::tiff::TiffEncoder::new(&mut writer);
                encoder.encode(
                    &image,
                    image.width(),
                    image.height(),
                    image::ColorType::RGBA(8)
                ).map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "decode error"))?;
            }
        }

        Ok(())
    }

    pub fn save_as(&mut self, path: &Path, format: &ImageFormat) -> std::io::Result<()> {
        self.save(path, format)?;
        self.path = Some(path.to_path_buf());
        self.image_format = Some(format.clone());
        Ok(())
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;

        for (_, layer) in &mut self.layers {
            let resized_image = image::imageops::resize(
                layer.get_image(),
                new_width,
                new_height,
                FilterType::Triangle
            );

            *layer = Image::new(resized_image);
        }
    }

    pub fn resize_canvas(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;

        for (_, layer) in &mut self.layers {
            let mut resized_image: image::RgbaImage = image::RgbaImage::new(new_width, new_height);
            for y in 0..layer.height().min(new_height) {
                for x in 0..layer.width().min(new_width) {
                    resized_image.put_pixel(x, y, layer.get_pixel(x, y));
                }
            }

            *layer = Image::new(resized_image);
        }
    }
}

#[derive(Clone, Debug)]
pub enum EditorOperation {
    Sequential(Vec<EditorOperation>),
    SetLayerState(usize, LayerState),
    SetActiveLayer(usize),
    SetImage(EditorImage),
    ImageOp(usize, ImageOperation)
}

impl EditorOperation {
    pub fn is_image_op(&self) -> bool {
        match self {
            EditorOperation::ImageOp(_, _) => true,
            _ => false
        }
    }

    pub fn extract_image_op(self) -> Option<(usize, ImageOperation)> {
        match self {
            EditorOperation::ImageOp(index, op) => Some((index, op)),
            _ => None
        }
    }
}

impl Display for EditorOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorOperation::Sequential(ops) => write!(f, "{}", ops.iter().map(|op| format!("{}", op)).join(", ")),
            EditorOperation::SetLayerState(_, _) => write!(f, "Set layer state"),
            EditorOperation::SetActiveLayer(_) => write!(f, "Set active layer"),
            EditorOperation::SetImage(image) => {
                match &image.path {
                    Some(_) => write!(f, "Open image"),
                    None => write!(f, "New image")
                }
            }
            EditorOperation::ImageOp(_, op) => write!(f, "{}", op),
        }
    }
}

pub struct Editor {
    image: EditorImage,
    active_layer_index: usize,
    undo_stack: Vec<(EditorOperation, EditorOperation)>,
    redo_stack: Vec<EditorOperation>,
    valid_region: Option<Region>
}

impl Editor {
    pub fn new(image: EditorImage) -> Editor {
        Editor {
            image,
            active_layer_index: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            valid_region: None
        }
    }

    pub fn image(&self) -> &EditorImage {
        &self.image
    }

    pub fn image_mut(&mut self) -> &mut EditorImage {
        &mut self.image
    }

    pub fn set_valid_region(&mut self, region: Option<Region>) {
        self.valid_region = region;
    }

    pub fn new_image_same(&self) -> Image {
        Image::new(image::RgbaImage::new(self.image.width(), self.image.height()))
    }

    pub fn apply_image_op(&mut self, op: ImageOperation) {
        self.apply_editor_op(EditorOperation::ImageOp(self.active_layer_index, op));
    }

    pub fn apply_editor_op(&mut self, op: EditorOperation) {
        self.internal_apply_op(op);
        self.redo_stack.clear();
    }

    pub fn undo_op(&mut self) {
        if let Some((orig_op, undo)) = self.undo_stack.pop() {
            match orig_op {
                EditorOperation::ImageOp(orig_op_layer, orig_op) => {
                    let undo = undo.extract_image_op().unwrap();
                    let mut update_op = self.image.get_layer_mut(undo.0).unwrap().update_operation_with_region(self.valid_region.clone());
                    undo.1.apply(&mut update_op, false);
                    self.redo_stack.push(EditorOperation::ImageOp(orig_op_layer, orig_op));
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

    pub fn history(&self) -> impl Iterator<Item=&EditorOperation> {
        self.undo_stack.iter().map(|(op, _)| op)
    }

    pub fn active_layer(&self) -> &Image {
        self.image.get_layer(self.active_layer_index).unwrap()
    }

    pub fn active_layer_index(&self) -> usize {
        self.active_layer_index
    }

    pub fn num_alive_layers(&self) -> usize {
        self.image.layers.iter().map(|(state, _)| state != &LayerState::Deleted).count()
    }

    pub fn add_layer(&mut self) {
        self.image_mut().add_layer();
    }

    pub fn duplicate_active_layer(&mut self) {
        if let Some(layer) = self.image.layers.get(self.active_layer_index) {
            if layer.0 == LayerState::Visible {
                let layer_image = layer.1.get_image().clone();
                self.image_mut().add_layer_with_image(layer_image);
            }
        }
    }

    pub fn delete_active_layer(&mut self) {
        if self.num_alive_layers() > 1 {
            self.apply_editor_op(
                EditorOperation::SetLayerState(
                    self.active_layer_index(),
                    LayerState::Deleted
                )
            );
        }
    }

    fn merge_draw_operations(&mut self) {
        for i in (0..self.undo_stack.len()).rev() {
            match &self.undo_stack[i].0 {
                EditorOperation::ImageOp(op_layer, op) => {
                    let op_layer = *op_layer;
                    if op.is_marker(ImageOperationMarker::BeginDraw) {
                        let first_marker_message = op.get_first_marker_message().cloned();

                        let max_index = self.undo_stack
                            .iter()
                            .enumerate()
                            .find(|(index, (op, _))| *index >= i && !op.is_image_op())
                            .map(|(index, _)| index)
                            .unwrap_or(self.undo_stack.len());

                        let (ops, mut undo_ops): (Vec<ImageOperation>, Vec<ImageOperation>) = self.undo_stack.drain(i..max_index)
                            .map(|(x, y)| (x.extract_image_op(), y.extract_image_op()))
                            .filter(|(x, y)| x.is_some() && y.is_some())
                            .map(|(x, y)| (x.unwrap().1, y.unwrap().1))
                            .unzip();

                        undo_ops.reverse();
                        self.undo_stack.push((
                            EditorOperation::ImageOp(op_layer, ImageOperation::Sequential(first_marker_message.clone(), ops).remove_markers()),
                            EditorOperation::ImageOp(op_layer, ImageOperation::Sequential(first_marker_message, undo_ops))
                        ));
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    fn internal_apply_op(&mut self, op: EditorOperation) {
        match op {
            EditorOperation::ImageOp(op_layer, op) => {
                if !op.is_marker(ImageOperationMarker::EndDraw) {
                    let mut update_op = self.image.get_layer_mut(op_layer).unwrap().update_operation_with_region(self.valid_region.clone());
                    if let Some(undo_op) = op.apply(&mut update_op, true) {
                        self.undo_stack.push((
                            EditorOperation::ImageOp(op_layer, op),
                            EditorOperation::ImageOp(op_layer, undo_op)
                        ));
                    }
                } else {
                    self.merge_draw_operations();
                }
            }
            op => self.internal_apply_other_op(op, true)
        }
    }

    fn internal_apply_other_op(&mut self, op: EditorOperation, push_undo: bool) {
        match op {
            EditorOperation::Sequential(ops) => {
                for op in ops {
                    self.internal_apply_other_op(op, push_undo);
                }
            }
            EditorOperation::SetLayerState(index, state) => {
                let current_state = self.image.layers_mut()[index].0.clone();
                self.image.layers_mut()[index].0 = state.clone();

                let change_active_layer_index = if state == LayerState::Deleted && self.active_layer_index == index {
                    if let Some((new_active_layer_index, _)) = self.image.layers().iter().enumerate().find(|(_, (state, _))| state != &LayerState::Deleted) {
                        let current_active_layer_index = self.active_layer_index;
                        self.active_layer_index = new_active_layer_index;
                        Some((current_active_layer_index, new_active_layer_index))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if push_undo {
                    let mut ops = vec![EditorOperation::SetLayerState(index, state)];
                    let mut undo_ops = vec![EditorOperation::SetLayerState(index, current_state)];

                    if let Some((old_active_layer_index, new_active_layer_index)) = change_active_layer_index {
                        ops.push(EditorOperation::SetActiveLayer(new_active_layer_index));
                        undo_ops.push(EditorOperation::SetActiveLayer(old_active_layer_index));
                    }

                    self.undo_stack.push((
                        EditorOperation::Sequential(ops),
                        EditorOperation::Sequential(undo_ops)
                    ));
                }
            }
            EditorOperation::SetActiveLayer(layer_index) => {
                let current_active_layer_index = self.active_layer_index;
                self.active_layer_index = layer_index;

                if push_undo {
                    self.undo_stack.push((
                        EditorOperation::SetActiveLayer(layer_index),
                        EditorOperation::SetActiveLayer(current_active_layer_index)
                    ));
                }
            }
            EditorOperation::SetImage(image) => {
                let mut current_image = image;
                std::mem::swap(&mut current_image, &mut self.image);
                self.active_layer_index = 0;

                if push_undo {
                    self.undo_stack.push((
                        EditorOperation::SetImage(self.image.clone()),
                        EditorOperation::SetImage(current_image)
                    ));
                }
            }
            EditorOperation::ImageOp(_, _) => panic!("Should not be used in this way.")
        }
    }
}