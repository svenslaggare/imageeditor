use std::os::raw::c_void;
use std::ptr;
use std::mem;

use gl::types::*;

use cgmath::Matrix4;

use crate::rendering::font::Font;
use crate::rendering::prelude::{Color, Position};
use crate::rendering::shader::Shader;

const NUM_TRIANGLES: i32 = 6;
const NUM_VERTICES_PER_TRIANGLE: i32 = 7;
const NUM_VERTICES_PER_CHARACTER: i32 = NUM_VERTICES_PER_TRIANGLE * NUM_TRIANGLES;
const MAX_BATCH_CHARACTERS: i32 = 200;

pub enum TextAlignment {
    Top,
    Bottom
}

pub struct TextRender {
    vertex_buffer: u32,
    vertex_array: u32
}

type DrawChar = (char, Color);

impl TextRender {
    pub fn new() -> TextRender {
        let (vertex_buffer, vertex_array) = unsafe {
            let (mut vertex_buffer, mut vertex_array) = (0, 0);
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::GenBuffers(1, &mut vertex_buffer);

            gl::BindVertexArray(vertex_array);

            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                ((NUM_VERTICES_PER_CHARACTER * MAX_BATCH_CHARACTERS) as usize * mem::size_of::<GLfloat>()) as GLsizeiptr,
                0 as *const c_void,
                gl::DYNAMIC_DRAW
            );

            let stride = NUM_VERTICES_PER_TRIANGLE * mem::size_of::<GLfloat>() as GLsizei;

            // Position attribute
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);

            // Texture coord attribute
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const c_void);
            gl::EnableVertexAttribArray(2);

            // Color attribute
            gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, stride, (4 * mem::size_of::<GLfloat>()) as *const c_void);
            gl::EnableVertexAttribArray(1);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            (vertex_buffer, vertex_array)
        };

        TextRender {
            vertex_buffer,
            vertex_array
        }
    }

    pub fn render_line<'b, TextIterator>(&self,
                                         shader: &Shader,
                                         transform: &Matrix4<f32>,
                                         font: &mut Font,
                                         text: TextIterator,
                                         position: Position,
                                         alignment: TextAlignment) -> f32
        where TextIterator: Iterator<Item=DrawChar> {

        struct BatchRender {
            characters_vertices: [f32; (NUM_VERTICES_PER_CHARACTER * MAX_BATCH_CHARACTERS) as usize],
            num_characters_drawn: i32
        }

        impl BatchRender {
            fn draw_characters(&mut self) {
                if self.num_characters_drawn > 0 {
                    unsafe {
                        gl::BufferSubData(
                            gl::ARRAY_BUFFER,
                            0,
                            ((self.num_characters_drawn * NUM_VERTICES_PER_CHARACTER) as usize * mem::size_of::<GLfloat>()) as GLsizeiptr,
                            &self.characters_vertices[0] as *const f32 as *const c_void);
                        gl::DrawArrays(gl::TRIANGLES, 0, self.num_characters_drawn as i32 * NUM_TRIANGLES);
                    }

                    self.num_characters_drawn = 0;
                }
            }

            fn draw_character(&mut self, vertices: &[f32]) {
                for (i, value) in vertices.iter().enumerate() {
                    self.characters_vertices[(self.num_characters_drawn * NUM_VERTICES_PER_CHARACTER) as usize + i] = *value;
                }

                self.num_characters_drawn += 1;
                if self.num_characters_drawn >= MAX_BATCH_CHARACTERS {
                    self.draw_characters();
                }
            }
        }

        let mut batch_render = BatchRender {
            characters_vertices: [0.0; (NUM_VERTICES_PER_CHARACTER * MAX_BATCH_CHARACTERS) as usize],
            num_characters_drawn: 0
        };

        let mut line_width = 0.0;

        unsafe {
            shader.activate();
            shader.set_matrix4(c_str!("transform"), &transform);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, font.texture_id());
            gl::BindVertexArray(self.vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
        }

        let mut draw_x = position.x;
        let draw_y = position.y;

        for (character, character_color) in text {
            let character_color = cgmath::Point3::<f32>::new(
                character_color.x as f32 / 255.0,
                character_color.y as f32 / 255.0,
                character_color.z as f32 / 255.0
            );

            let font_character = font.get(character).unwrap();

            let x = draw_x + font_character.bearing.x as f32;
            let y = match alignment {
                TextAlignment::Top => draw_y - font_character.bearing.y as f32 + font_character.line_height,
                TextAlignment::Bottom => draw_y - font_character.bearing.y as f32
            };

            let char_width = font_character.size.x as f32;
            let char_height = font_character.size.y as f32;

            let vertices: [f32; NUM_VERTICES_PER_CHARACTER as usize] = [
                x,               y,                font_character.texture_left,   font_character.texture_top,      character_color.x, character_color.y, character_color.z,
                x + char_width,  y,                font_character.texture_right,  font_character.texture_top,      character_color.x, character_color.y, character_color.z,
                x + char_width,  y + char_height,  font_character.texture_right,  font_character.texture_bottom,   character_color.x, character_color.y, character_color.z,

                x + char_width,  y + char_height,  font_character.texture_right,  font_character.texture_bottom,   character_color.x, character_color.y, character_color.z,
                x,               y + char_height,  font_character.texture_left,   font_character.texture_bottom,   character_color.x, character_color.y, character_color.z,
                x,               y,                font_character.texture_left,   font_character.texture_top,      character_color.x, character_color.y, character_color.z,
            ];

            batch_render.draw_character(&vertices);
            draw_x += font_character.advance_x;
            line_width += font_character.advance_x;
        }

        batch_render.draw_characters();

        line_width
    }
}

impl Drop for TextRender {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteBuffers(1, &self.vertex_buffer);
        }
    }
}