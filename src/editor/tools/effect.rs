use std::time::SystemTime;

use glfw::{Window, WindowEvent, Key, Action};
use cgmath::{Matrix3};

use crate::rendering::prelude::Position;
use crate::editor::tools::Tool;
use crate::command_buffer::Command;
use crate::editor::Image;
use crate::editor::image_operation::{ImageOperation};
use crate::rendering::shader::Shader;
use crate::editor;
use crate::rendering::texture_render::TextureRender;
use crate::rendering::framebuffer::FrameBuffer;

fn get_changed_time(filename: &str) -> std::io::Result<SystemTime> {
    std::fs::metadata(filename)?.modified()
}

const VERTEX_SHADER_FILENAME: &str = "content/shaders/texture.vs";

pub struct EffectDrawTool {
    shader_filename: String,
    shader_changed_time: SystemTime,
    shader: Option<Shader>,
    texture_render: TextureRender,
    preview_frame_buffer: Option<FrameBuffer>,
    op_frame_buffer: Option<FrameBuffer>,
    changed: bool
}

impl EffectDrawTool {
    pub fn new(shader_filename: &str) -> EffectDrawTool {
        EffectDrawTool {
            shader_filename: shader_filename.to_owned(),
            shader_changed_time: get_changed_time(shader_filename).unwrap(),
            shader: None,
            texture_render: TextureRender::new(),
            preview_frame_buffer: None,
            op_frame_buffer: None,
            changed: true
        }
    }

    fn try_create_shader(&mut self) {
        match Shader::new(VERTEX_SHADER_FILENAME, &self.shader_filename, None) {
            Ok(shader) => {
                println!("Loaded shader: {}.", self.shader_filename);
                self.shader = Some(shader);
                self.changed = true;
            }
            Err(error) => {
                println!("Failed loading shader: {}", error);
            }
        }
    }

    fn generate_image(&self, frame_buffer: &FrameBuffer, source_image: &editor::Image, destination_image_buffer: &mut [u8]) -> bool {
        if let Some(shader) = self.shader.as_ref() {
            let width = frame_buffer.width();
            let height = frame_buffer.height();

            let transform = cgmath::ortho(
                0.0,
                width as f32,
                0.0,
                height as f32,
                0.0,
                1.0
            );

            let binding = frame_buffer.bind();

            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            self.texture_render.render(
                shader,
                &transform,
                source_image.get_texture(),
                Position::new(0.0, 0.0)
            );

            binding.download(destination_image_buffer);
            return true;
        }

        return false;
    }
}

impl Tool for EffectDrawTool {
    fn on_active(&mut self) {
        self.changed = true;
    }

    fn update(&mut self) {
        if let Ok(changed_time) = get_changed_time(&self.shader_filename) {
            if changed_time > self.shader_changed_time {
                self.try_create_shader();
                self.shader_changed_time = changed_time;
            }
        }
    }

    fn handle_command(&mut self, command: &Command) {
        match command {
            Command::SetImageSize(width, height) => {
                self.preview_frame_buffer = Some(FrameBuffer::new(*width, *height, 4));
                self.op_frame_buffer = Some(FrameBuffer::new(*width, *height, 4));
                self.try_create_shader();
                self.changed = true;
            }
            _ => {}
        }
    }

    fn process_event(&mut self, _window: &mut Window, event: &WindowEvent, _transform: &Matrix3<f32>, image: &Image) -> Option<ImageOperation> {
        match event {
            glfw::WindowEvent::Key(Key::Enter, _, Action::Press, _) => {
                if let Some(frame_buffer) = self.op_frame_buffer.as_ref() {
                    let mut op_image = image::RgbaImage::new(frame_buffer.width(), frame_buffer.height());
                    if self.generate_image(frame_buffer, image, op_image.as_mut()) {
                        self.changed = true;
                        return Some(ImageOperation::SetImage { start_x: 0, start_y: 0, image: op_image, blend: false });
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn preview(&mut self, image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        if self.changed {
            if let Some(preview_frame_buffer) = self.preview_frame_buffer.as_ref() {
                let mut preview_image_op = preview_image.update_operation();
                if self.generate_image(preview_frame_buffer, image, preview_image_op.raw_pixels_mut()) {
                    self.changed = false;
                }
            }

            return true;
        }

        return false;
    }
}