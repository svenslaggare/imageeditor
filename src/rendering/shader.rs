use std::ffi::{CString, CStr};
use std::fs::File;
use std::io::Read;
use std::ptr;
use std::str;

use gl;
use gl::types::*;

use cgmath::{Matrix, Matrix4};

pub struct Shader {
    id: u32,
}

fn load_file_as_cstr(filename: &str) -> Result<CString, String> {
    let mut file = File::open(filename).map_err(|_| format!("Failed to open {}.", filename))?;

    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|_| "Failed to read shader.".to_string())?;
    return CString::new(content.as_bytes()).map_err(|_| "Failed converting to CString.".to_string());
}

fn strip_null(chars: &mut Vec<u8>) {
    chars.retain(|x| *x != 0);
}

unsafe fn get_compile_error(name: &str, shader: u32, is_program: bool) -> Option<String> {
    let mut success = gl::FALSE as GLint;
    let mut info_log = vec![0; 1024];
    info_log.set_len(1024 - 1); // Subtract 1 to skip the trailing null character

    if is_program {
        gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(shader, 1024, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
        } else {
            return None;
        }
    } else {
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(shader, 1024, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
        } else {
            return None;
        }
    }

    strip_null(&mut info_log);
    return Some(name.to_owned() + "\n" + &String::from_utf8(info_log).unwrap());
}

impl Shader {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str, geometry_shader_path: Option<&str>) -> Result<Shader, String> {
        let mut shader = Shader { id: 0 };

        let vertex_shader_code = load_file_as_cstr(vertex_shader_path).map_err(|_| "Failed loading vertex shader.".to_owned())?;
        let fragment_shader_code = load_file_as_cstr(fragment_shader_path).map_err(|_| "Failed loading fragment shader.".to_owned())?;
        let geometry_shader_code = geometry_shader_path.map(|path| load_file_as_cstr(path));

        unsafe {
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex_shader, 1, &vertex_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(vertex_shader);
            if let Some(err) = get_compile_error(vertex_shader_path, vertex_shader, false) {
                gl::DeleteShader(vertex_shader);
                return Err(err);
            }

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment_shader, 1, &fragment_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(fragment_shader);
            if let Some(err) = get_compile_error(fragment_shader_path, fragment_shader, false) {
                gl::DeleteShader(vertex_shader);
                gl::DeleteShader(fragment_shader);
                return Err(err);
            }

            let geometry_shader = if let Some(geometry_shader_code) = geometry_shader_code {
                let geometry_shader_code = geometry_shader_code.map_err(|_| "Failed loading geometry shader.".to_owned())?;

                let geometry_shader = gl::CreateShader(gl::GEOMETRY_SHADER);
                gl::ShaderSource(geometry_shader, 1, &geometry_shader_code.as_ptr(), ptr::null());
                gl::CompileShader(geometry_shader);
                if let Some(err) = get_compile_error(geometry_shader_path.unwrap(), geometry_shader, false) {
                    gl::DeleteShader(vertex_shader);
                    gl::DeleteShader(fragment_shader);
                    gl::DeleteShader(geometry_shader);
                    return Err(err);
                }

                Some(geometry_shader)
            } else {
                None
            };

            let program_id = gl::CreateProgram();
            gl::AttachShader(program_id, vertex_shader);
            gl::AttachShader(program_id, fragment_shader);
            if let Some(geometry_shader) = geometry_shader {
                gl::AttachShader(program_id, geometry_shader);
            }

            gl::LinkProgram(program_id);
            if let Some(err) = get_compile_error(&vertex_shader_path.replace(".vs", ".pŕogram"), program_id, true) {
                gl::DeleteShader(vertex_shader);
                gl::DeleteShader(fragment_shader);

                if let Some(geometry_shader) = geometry_shader {
                    gl::AttachShader(program_id, geometry_shader);
                }
                return Err(err);
            }

            // Delete the shaders as they linked into our program
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            if let Some(geometry_shader) = geometry_shader {
                gl::AttachShader(program_id, geometry_shader);
            }

            shader.id = program_id;
        }

        Ok(shader)
    }

    pub unsafe fn activate(&self) {
        gl::UseProgram(self.id)
    }

    pub unsafe fn set_bool(&self, name: &CStr, value: bool) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as i32);
    }

    pub unsafe fn set_int32(&self, name: &CStr, value: i32) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value);
    }

    pub unsafe fn set_float32(&self, name: &CStr, value: f32) {
        gl::Uniform1f(gl::GetUniformLocation(self.id, name.as_ptr()), value);
    }

    pub unsafe fn set_vector2(&self, name: &CStr, x: f32, y: f32) {
        gl::Uniform2f(gl::GetUniformLocation(self.id, name.as_ptr()), x, y);
    }

    pub unsafe fn set_vector3(&self, name: &CStr, x: f32, y: f32, z: f32) {
        gl::Uniform3f(gl::GetUniformLocation(self.id, name.as_ptr()), x, y, z);
    }

    pub unsafe fn set_matrix4(&self, name: &CStr, mat: &Matrix4<f32>) {
        gl::UniformMatrix4fv(gl::GetUniformLocation(self.id, name.as_ptr()), 1, gl::FALSE, mat.as_ptr());
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}