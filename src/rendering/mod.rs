use std::ops::Deref;

use crate::rendering::shader::Shader;

pub mod prelude;
#[macro_use]
pub mod helpers;
pub mod font;
pub mod shader;
pub mod texture;
pub mod framebuffer;
pub mod texture_render;
pub mod text_render;
pub mod rectangle_render;
pub mod solid_rectangle_render;

pub struct ShaderAndRender<T> {
    shader: Shader,
    render: T,
}

impl<T> ShaderAndRender<T> {
    pub fn new(shader: Shader, render: T) -> ShaderAndRender<T> {
        ShaderAndRender {
            shader,
            render
        }
    }
}

impl<T> ShaderAndRender<T> {
    pub fn shader(&self) -> &Shader {
        &self.shader
    }
}

impl<T> Deref for ShaderAndRender<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.render
    }
}