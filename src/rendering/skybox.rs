// Name: Cieran O'Neill
// Date: 05/12/2025
// CS51012 Assignment Part 2
use std::fs;

use gl::types::GLuint;
use glam::{Mat3, Mat4};

use crate::{os_str_sub, assets::shaders::create_shader_program};

pub struct Skybox {
    vao: GLuint,
    vbo: GLuint,
    cubemap_texture: u32,
    shader_program: u32,
}

impl Skybox {
    pub fn new(cubemap_texture: u32) -> Result<Self, Box<dyn std::error::Error>> {
        // Skybox vertices (a simple cube centered at origin)
        let vertices: [f32; 108] = [
            // positions          
            -1.0,  1.0, -1.0,
            -1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
             1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,
            
            -1.0, -1.0,  1.0,
            -1.0, -1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0,  1.0,
            -1.0, -1.0,  1.0,
            
             1.0, -1.0, -1.0,
             1.0, -1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0, -1.0,
             1.0, -1.0, -1.0,
            
            -1.0, -1.0,  1.0,
            -1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0, -1.0,  1.0,
            -1.0, -1.0,  1.0,
            
            -1.0,  1.0, -1.0,
             1.0,  1.0, -1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,
            -1.0,  1.0, -1.0,
            
            -1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
             1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
             1.0, -1.0,  1.0
        ];
        
        // Load skybox shaders
        let vertex_src = fs::read_to_string(os_str_sub("shaders", "skybox", "skybox.vert"))?;
        let fragment_src = fs::read_to_string(os_str_sub("shaders", "skybox", "skybox.frag"))?;
        let shader_program = create_shader_program(&vertex_src, &fragment_src)?;
        
        unsafe {
            let mut vao = 0;
            let mut vbo = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            );
            
            // Position attribute
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as i32, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            
            gl::BindVertexArray(0);
            
            Ok(Self {
                vao,
                vbo,
                cubemap_texture,
                shader_program,
            })
        }
    }
    
    pub fn draw(&self, view: &Mat4, projection: &Mat4) {
        unsafe {
            // Save current OpenGL state
            let mut current_program: i32 = 0;
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut current_program);
            
            let mut current_depth_func: i32 = 0;
            gl::GetIntegerv(gl::DEPTH_FUNC, &mut current_depth_func);
            
            // Proper setup for skybox rendering
            gl::DepthFunc(gl::LEQUAL); // Critical for skybox depth testing
            gl::DepthMask(gl::FALSE);  // Don't write to depth buffer
            gl::UseProgram(self.shader_program);
            
            // Set uniforms
            let view_loc = gl::GetUniformLocation(self.shader_program, b"view\0".as_ptr() as *const _);
            let proj_loc = gl::GetUniformLocation(self.shader_program, b"projection\0".as_ptr() as *const _);
            let skybox_loc = gl::GetUniformLocation(self.shader_program, b"skybox\0".as_ptr() as *const _);
            
            // Create a view matrix without translation for the skybox
            let view_no_translate = Mat4::from_mat3(Mat3::from_mat4(*view));
            
            if view_loc >= 0 {
                gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view_no_translate.to_cols_array().as_ptr());
            }
            if proj_loc >= 0 {
                gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());
            }
            
            // Bind texture
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.cubemap_texture);
            if skybox_loc >= 0 {
                gl::Uniform1i(skybox_loc, 0);
            }
            
            // Draw skybox
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);
            
            // Restore state
            gl::DepthMask(gl::TRUE); // Re-enable depth writing
            gl::DepthFunc(current_depth_func as u32); // Restore depth function
            gl::UseProgram(current_program as u32); // CRITICAL: Restore previous shader
        }
    }
}

impl Drop for Skybox {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.shader_program);
        }
    }
}
