use std::ffi::CString;
use glam::{Mat3, Mat4, Vec3, Vec4};

pub struct Uniforms {
    pub model: i32,
    pub view: i32,
    pub projection: i32,
    pub normal_matrix: i32,
    pub light_pos: i32,
    pub light_pos_world: i32,
    pub emit_mode: i32,
    pub use_texture: i32,
    pub base_texture: i32,
    pub shadow_map: i32,
    pub shadow_far: i32,
}

impl Uniforms {
    pub fn new(shader_program: u32) -> Self {
        Self {
            model: Self::get_location(shader_program, "model"),
            view: Self::get_location(shader_program, "view"),
            projection: Self::get_location(shader_program, "projection"),
            normal_matrix: Self::get_location(shader_program, "normal_matrix"),
            light_pos: Self::get_location(shader_program, "light_pos"),
            light_pos_world: Self::get_location(shader_program, "light_pos_world"),
            emit_mode: Self::get_location(shader_program, "emit_mode"),
            use_texture: Self::get_location(shader_program, "use_texture"),
            base_texture: Self::get_location(shader_program, "base_texture"),
            shadow_map: Self::get_location(shader_program, "shadow_map"),
            shadow_far: Self::get_location(shader_program, "shadow_far"),
        }
    }

    fn get_location(shader_program: u32, name: &str) -> i32 {
        let c_name = CString::new(name).expect("Invalid uniform name");
        unsafe { gl::GetUniformLocation(shader_program, c_name.as_ptr()) }
    }

    pub fn set_model_matrix(&self, matrix: &Mat4) {
        if self.model >= 0 {
            unsafe {
                gl::UniformMatrix4fv(self.model, 1, gl::FALSE, matrix.to_cols_array().as_ptr());
            }
        }
    }

    pub fn set_view_matrix(&self, matrix: &Mat4) {
        if self.view >= 0 {
            unsafe {
                gl::UniformMatrix4fv(self.view, 1, gl::FALSE, matrix.to_cols_array().as_ptr());
            }
        }
    }

    pub fn set_projection_matrix(&self, matrix: &Mat4) {
        if self.projection >= 0 {
            unsafe {
                gl::UniformMatrix4fv(self.projection, 1, gl::FALSE, matrix.to_cols_array().as_ptr());
            }
        }
    }

    pub fn set_normal_matrix(&self, model_matrix: &Mat4) {
        if self.normal_matrix >= 0 {
            let normal_matrix = Mat3::from_mat4(*model_matrix).inverse().transpose();
            unsafe {
                gl::UniformMatrix3fv(self.normal_matrix, 1, gl::FALSE, normal_matrix.to_cols_array().as_ptr());
            }
        }
    }

    pub fn set_light_pos(&self, pos: Vec4) {
        if self.light_pos >= 0 {
            unsafe {
                gl::Uniform4f(self.light_pos, pos.x, pos.y, pos.z, pos.w);
            }
        }
    }

    pub fn set_light_pos_world(&self, pos: Vec3) {
        if self.light_pos_world >= 0 {
            unsafe {
                gl::Uniform3f(self.light_pos_world, pos.x, pos.y, pos.z);
            }
        }
    }

    pub fn set_emit_mode(&self, mode: u32) {
        if self.emit_mode >= 0 {
            unsafe {
                gl::Uniform1ui(self.emit_mode, mode);
            }
        }
    }

    pub fn set_use_texture(&self, use_tex: bool) {
        if self.use_texture >= 0 {
            unsafe {
                gl::Uniform1i(self.use_texture, if use_tex { 1 } else { 0 });
            }
        }
    }

    pub fn set_base_texture(&self, texture_unit: i32) {
        if self.base_texture >= 0 {
            unsafe {
                gl::Uniform1i(self.base_texture, texture_unit);
            }
        }
    }

    pub fn set_shadow_map(&self, texture_unit: i32) {
        if self.shadow_map >= 0 {
            unsafe {
                gl::Uniform1i(self.shadow_map, texture_unit);
            }
        }
    }

    pub fn set_shadow_far(&self, far: f32) {
        if self.shadow_far >= 0 {
            unsafe {
                gl::Uniform1f(self.shadow_far, far);
            }
        }
    }
}