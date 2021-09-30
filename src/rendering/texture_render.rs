use std::{mem, ptr};
use std::os::raw::c_void;

use gl::types::*;
use cgmath::Matrix4;

use crate::rendering::texture::Texture;
use crate::rendering::shader::Shader;
use crate::rendering::prelude::Rectangle;

const FLOATS_PER_VERTEX: i32 = 4;
const NUM_VERTICES: i32 = 6;
const BUFFER_SIZE: usize = (NUM_VERTICES * FLOATS_PER_VERTEX) as usize;

pub struct TextureRender {
    vertex_buffer: u32,
    vertex_array: u32
}

impl TextureRender {
    pub fn new() -> TextureRender {
        unsafe {
            let (mut vertex_buffer, mut vertex_array) = (0, 0);
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::GenBuffers(1, &mut vertex_buffer);

            gl::BindVertexArray(vertex_array);

            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (BUFFER_SIZE * mem::size_of::<GLfloat>()) as GLsizeiptr,
                0 as *const c_void,
                gl::DYNAMIC_DRAW);

            let stride = FLOATS_PER_VERTEX * mem::size_of::<GLfloat>() as GLsizei;

            // Position attribute
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);

            // Texture coordinates attribute
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const c_void);
            gl::EnableVertexAttribArray(1);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            TextureRender {
                vertex_buffer,
                vertex_array
            }
        }
    }

    pub fn render_sub(&self,
                      shader: &Shader,
                      transform: &Matrix4<f32>,
                      texture: &Texture,
                      position: cgmath::Point2<f32>,
                      scale: f32,
                      source_rectangle: Option<Rectangle>) {
        unsafe {
            shader.activate();
            shader.set_matrix4(c_str!("transform"), &transform);

            gl::ActiveTexture(gl::TEXTURE0);
            texture.bind();

            gl::BindVertexArray(self.vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);

            let source_rectangle = source_rectangle.unwrap_or(
                Rectangle::new(
                    0.0,
                    0.0,
                    texture.width() as f32,
                    texture.height() as f32
                )
            );

            let width = source_rectangle.size.x * scale;
            let height = source_rectangle.size.y * scale;

            let top_left_x = source_rectangle.left() / texture.width() as f32;
            let top_left_y = source_rectangle.top() / texture.height() as f32;
            let top_right_x = source_rectangle.right() / texture.width() as f32;
            let top_right_y = source_rectangle.top() / texture.height() as f32;

            let bottom_left_x = source_rectangle.left() / texture.width() as f32;
            let bottom_left_y = source_rectangle.bottom() / texture.height() as f32;
            let bottom_right_x = source_rectangle.right() / texture.width() as f32;
            let bottom_right_y = source_rectangle.bottom() / texture.height() as f32;

            let vertices: [f32; BUFFER_SIZE] = [
                position.x, position.y,                   top_left_x, top_left_y,               // Top-left
                position.x + width, position.y,           top_right_x, top_right_y,             // Top-right
                position.x + width, position.y + height,  bottom_right_x, bottom_right_y,       // Bottom-right

                position.x, position.y + height,          bottom_left_x, bottom_left_y,         // Bottom-left
                position.x + width, position.y + height,  bottom_right_x, bottom_right_y,       // Bottom-right
                position.x, position.y,                   top_left_x, top_left_y,               // Top-left
            ];

            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (BUFFER_SIZE * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &vertices[0] as *const f32 as *const c_void
            );
            gl::DrawArrays(gl::TRIANGLES, 0, NUM_VERTICES);
        }
    }

    pub fn render(&self,
                  shader: &Shader,
                  transform: &Matrix4<f32>,
                  texture: &Texture,
                  position: cgmath::Point2<f32>) {
        self.render_sub(shader, transform, texture, position, 1.0, None)
    }
}

impl Drop for TextureRender {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteBuffers(1, &self.vertex_buffer);
        }
    }
}