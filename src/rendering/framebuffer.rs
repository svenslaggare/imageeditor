use crate::rendering::helpers::channels_type;
use std::os::raw::c_void;

pub struct FrameBuffer {
    width: u32,
    height: u32,
    channels: u32,
    frame_buffer_id: u32,
    color_buffer_id: u32
}

pub struct FrameBufferBinding<'a> {
    frame_buffer: &'a FrameBuffer
}

impl<'a> FrameBufferBinding<'a> {
    pub fn new(frame_buffer: &'a FrameBuffer) -> FrameBufferBinding {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer.frame_buffer_id);
        }

        FrameBufferBinding {
            frame_buffer
        }
    }

    pub fn download(&self, buffer: &mut [u8]) {
        assert!(buffer.len() >= (self.frame_buffer.width * self.frame_buffer.height * self.frame_buffer.channels) as usize);

        unsafe {
            gl::ReadPixels(
                0,
                0,
                self.frame_buffer.width as i32,
                self.frame_buffer.height as i32,
                channels_type(self.frame_buffer.channels),
                gl::UNSIGNED_BYTE,
                &mut buffer[0] as *mut u8 as *mut c_void
            );
        }
    }
}

impl<'a> Drop for FrameBufferBinding<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32, channels: u32) -> FrameBuffer {
        let mut frame_buffer_id = 0;
        let mut color_buffer_id = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut frame_buffer_id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer_id);

            gl::GenTextures(1, &mut color_buffer_id);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, color_buffer_id);

            let channel_type = channels_type(channels);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                channel_type as i32,
                width as i32,
                height as i32,
                0,
                channel_type,
                gl::UNSIGNED_BYTE,
                std::ptr::null()
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
	        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                color_buffer_id,
                0
            );

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        FrameBuffer {
            width,
            height,
            channels,
            frame_buffer_id,
            color_buffer_id
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn color_buffer_id(&self) -> u32 {
        self.color_buffer_id
    }

    pub fn bind(&self) -> FrameBufferBinding {
        FrameBufferBinding::new(self)
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.frame_buffer_id);
            gl::DeleteTextures(1, &self.color_buffer_id);
        }
    }
}