use std::os::raw::c_void;

use crate::rendering::helpers::channels_type;

#[derive(Debug)]
pub struct Texture {
    texture_id: u32,
    width: u32,
    height: u32,
    channels: u32
}

impl Texture {
    pub fn new(width: u32, height: u32, channels: u32) -> Texture {
        let mut texture_id: u32 = 0;
        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1); // Disable byte-alignment restriction

            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);
            let mut color = [0.0, 0.0, 0.0, 0.0];
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR as u32, color.as_mut_ptr());
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR as u32, color.as_mut_ptr());
        }

        Texture {
            texture_id,
            width,
            height,
            channels,
        }
    }

    pub fn from_image(image: &image::RgbaImage) -> Texture {
        let texture = Texture::new(image.width(), image.height(), 4);
        texture.upload(image.as_ref());
        texture
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn base_index(&self, x: u32, y: u32) -> usize {
        (y * self.width * self.channels + x * self.channels) as usize
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
        }
    }

    pub fn upload(&self, buffer: &[u8]) {
        assert!(buffer.len() >= (self.width * self.height * self.channels) as usize);

        let channel_type = channels_type(self.channels);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                channel_type as i32,
                self.width as i32,
                self.height as i32,
                0,
                channel_type,
                gl::UNSIGNED_BYTE,
                &buffer[0] as *const u8 as *const c_void
            );
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture_id);
        }
    }
}