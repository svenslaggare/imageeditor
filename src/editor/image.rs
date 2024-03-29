use image::{Rgba};

use crate::rendering::texture::Texture;
use crate::editor::image_operation::{ImageOperationSource, ImageSource};
use crate::editor::Region;

pub type Color = Rgba<u8>;

#[derive(Debug)]
pub struct Image {
    underlying_image: image::RgbaImage,
    texture: Texture
}

impl Image {
    pub fn new(image: image::RgbaImage) -> Image {
        let texture = Texture::new(image.width(), image.height(), 4);
        texture.upload(image.as_ref());
        Image {
            underlying_image: image,
            texture
        }
    }

    pub fn get_texture(&self) -> &Texture {
        &self.texture
    }

    pub fn get_image(&self) -> &image::RgbaImage {
        &self.underlying_image
    }

    fn upload_to_gpu(&mut self) {
        self.texture.upload(self.underlying_image.as_ref());
    }

    pub fn update_operation(&mut self) -> ImageUpdateOperation {
        ImageUpdateOperation::new(self, None)
    }

    pub fn update_operation_with_region(&mut self, valid_region: Option<Region>) -> ImageUpdateOperation {
        ImageUpdateOperation::new(self, valid_region)
    }

    pub fn clear_cpu(&mut self) {
        for pixel in self.underlying_image.pixels_mut() {
            *pixel = image::Rgba([0, 0, 0, 0]);
        }
    }
}

impl Clone for Image {
    fn clone(&self) -> Self {
        Image::new(self.underlying_image.clone())
    }
}

impl ImageSource for Image {
    fn width(&self) -> u32 {
        self.underlying_image.width()
    }

    fn height(&self) -> u32 {
        self.underlying_image.height()
    }

    fn get_pixel(&self, x: u32, y: u32) -> Color {
        *self.underlying_image.get_pixel(x, y)
    }
}

pub struct ImageUpdateOperation<'a> {
    image: &'a mut Image,
    valid_region: Option<Region>
}

impl<'a> ImageUpdateOperation<'a> {
    pub fn new(image: &'a mut Image, valid_region: Option<Region>) -> ImageUpdateOperation<'a> {
        ImageUpdateOperation {
            image,
            valid_region
        }
    }

    pub fn raw_pixels(&self) -> &[u8] {
        self.image.underlying_image.as_ref()
    }

    pub fn raw_pixels_mut(&mut self) -> &mut [u8] {
        self.image.underlying_image.as_mut()
    }

    pub fn get_image(&self) -> &Image {
        self.image
    }
}

impl<'a> ImageSource for ImageUpdateOperation<'a> {
    fn width(&self) -> u32 {
        self.image.width()
    }

    fn height(&self) -> u32 {
        self.image.height()
    }

    fn get_pixel(&self, x: u32, y: u32) -> Color {
        *self.image.underlying_image.get_pixel(x, y)
    }
}

impl<'a> ImageOperationSource for ImageUpdateOperation<'a> {
    fn put_pixel(&mut self, x: u32, y: u32, pixel: Color) {
        if let Some(valid_region) = self.valid_region.as_ref() {
            if valid_region.contains(x as i32, y as i32) {
                self.image.underlying_image.put_pixel(x, y, pixel);
            }
        } else {
            self.image.underlying_image.put_pixel(x, y, pixel);
        }
    }
}

impl<'a> Drop for ImageUpdateOperation<'a> {
    fn drop(&mut self) {
        self.image.upload_to_gpu();
    }
}