use std::{mem, ptr};
use std::os::raw::c_void;

use gl::types::*;
use cgmath::Matrix4;

use crate::rendering::shader::Shader;
use crate::rendering::prelude::{Rectangle, Color, Color4};

const FLOATS_PER_VERTEX: i32 = 2 + 4;
const NUM_VERTICES: i32 = 5;
const BUFFER_SIZE: usize = (NUM_VERTICES * FLOATS_PER_VERTEX) as usize;

pub struct RectangleRender {
    vertex_buffer: u32,
    vertex_array: u32
}

impl RectangleRender {
    pub fn new() -> RectangleRender {
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
                gl::DYNAMIC_DRAW
            );

            let stride = FLOATS_PER_VERTEX * mem::size_of::<GLfloat>() as GLsizei;

            // Position attribute
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);

            // Color attribute
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const c_void);
            gl::EnableVertexAttribArray(1);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            RectangleRender {
                vertex_buffer,
                vertex_array
            }
        }
    }

    pub fn render(&self,
                  shader: &Shader,
                  transform: &Matrix4<f32>,
                  rectangle: &Rectangle,
                  color: Color4) {
        unsafe {
            shader.activate();
            shader.set_matrix4(c_str!("transform"), &transform);

            gl::BindVertexArray(self.vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);

            let color = [
                color.x as f32 / 255.0,
                color.y as f32 / 255.0,
                color.z as f32 / 255.0,
                color.w as f32 / 255.0
            ];

            let vertices: [f32; BUFFER_SIZE] = [
                rectangle.left(),    rectangle.top(),      color[0], color[1], color[2], color[3],
                rectangle.right(),   rectangle.top(),      color[0], color[1], color[2], color[3],
                rectangle.right(),   rectangle.bottom(),   color[0], color[1], color[2], color[3],
                rectangle.left(),    rectangle.bottom(),   color[0], color[1], color[2], color[3],
                rectangle.left(),    rectangle.top(),      color[0], color[1], color[2], color[3]
            ];

            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (BUFFER_SIZE * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &vertices[0] as *const f32 as *const c_void
            );
            gl::DrawArrays(gl::LINE_STRIP, 0, NUM_VERTICES);
        }
    }
}

impl Drop for RectangleRender {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteBuffers(1, &self.vertex_buffer);
        }
    }
}