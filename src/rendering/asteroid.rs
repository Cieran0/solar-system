// Name: Cieran O'Neill
// Date: 05/12/2025
// CS51012 Assignment Part 2
use std::error::Error;

use gl::types::{GLsizei, GLuint};
use glam::{Mat4, Vec3};
use rand::Rng;

use crate::{assets::obj::ObjModel, uniforms::Uniforms};

#[repr(C)]
pub struct AsteroidStatic {
    pub orbit_radius: f32,
    pub inclination: f32,
    pub scale: f32,
    pub y_axis_offset: f32,
}

#[repr(C)]
pub struct AsteroidDynamic {
    pub orbit_angle: f32,
    pub rotation: f32,
    pub orbit_speed: f32,
    pub rotation_speed: f32,
}

pub struct AsteroidField {
    static_ssbo: GLuint,
    dynamic_ssbo: GLuint,
    matrix_buffer: GLuint,
    model: ObjModel,
    texture: GLuint,
    shader_program: GLuint,
    uniforms: Uniforms,
    compute_program: GLuint,
    instance_count: usize,
}

impl AsteroidField {
    pub fn new(
        count: usize,
        model_path: &str,
        texture: GLuint,
        shader_program: GLuint,
        compute_program: GLuint,
    ) -> Result<Self, Box<dyn Error>> {
        let model = ObjModel::new(model_path)?;
        let uniforms = Uniforms::new(shader_program);

        let (mut static_ssbo, mut dynamic_ssbo, mut matrix_buffer) = (0, 0, 0);
        unsafe {
            gl::GenBuffers(1, &mut static_ssbo);
            gl::GenBuffers(1, &mut dynamic_ssbo);
            gl::GenBuffers(1, &mut matrix_buffer);
        }

        let mut rng = rand::rng();

        // Static asteroid data
        let ast_static: Vec<AsteroidStatic> = (0..count)
            .map(|_| AsteroidStatic {
                orbit_radius: rng.random_range(11.0..14.0),
                inclination: (rng.random_range(-10.0..10.0) as f32).to_radians(),
                scale: rng.random_range(0.05..0.1),
                y_axis_offset: rng.random_range(0.0..std::f32::consts::TAU),
            })
            .collect();

        // Dynamic asteroid data
        let ast_dyn: Vec<AsteroidDynamic> = (0..count)
            .map(|_| AsteroidDynamic {
                orbit_angle: rng.random_range(0.0..std::f32::consts::TAU),
                rotation: 0.0,
                orbit_speed: rng.random_range(0.05..0.25),
                rotation_speed: rng.random_range(0.5..100.0),
            })
            .collect();

        // Upload static data (GPU will not change this)
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, static_ssbo);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                (ast_static.len() * std::mem::size_of::<AsteroidStatic>()) as isize,
                ast_static.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, static_ssbo);
        }

        // Upload dynamic data (GPU will update every frame)
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, dynamic_ssbo);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                (ast_dyn.len() * std::mem::size_of::<AsteroidDynamic>()) as isize,
                ast_dyn.as_ptr() as *const _,
                gl::DYNAMIC_COPY,
            );
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, dynamic_ssbo);
        }

        // Allocate matrix buffer
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, matrix_buffer);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                (count * std::mem::size_of::<[[f32; 4]; 4]>()) as isize,
                std::ptr::null(),
                gl::DYNAMIC_COPY,
            );
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, matrix_buffer);

            gl::BindBuffer(gl::ARRAY_BUFFER, matrix_buffer);
            model.setup_instancing(matrix_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        Ok(Self {
            static_ssbo,
            dynamic_ssbo,
            matrix_buffer,
            model,
            texture,
            shader_program,
            uniforms,
            compute_program,
            instance_count: count,
        })
    }

    pub fn update(&self, dt: f32) {
        unsafe {
            gl::UseProgram(self.compute_program);
            let dt_loc = gl::GetUniformLocation(self.compute_program, b"dt\0".as_ptr() as *const _);
            gl::Uniform1f(dt_loc, dt);

            let groups = ((self.instance_count as u32) + 255) / 256;
            gl::DispatchCompute(groups, 1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
    }

    pub fn draw(
        &self,
        view: &Mat4,
        projection: &Mat4,
        light_pos: Vec3,
        shadow_map: GLuint,
        shadow_far: f32,
    ) {
        unsafe { gl::UseProgram(self.shader_program) };
        self.uniforms.set_view_matrix(view);
        self.uniforms.set_projection_matrix(projection);
        self.uniforms.set_use_texture(true);
        self.uniforms.set_shadow_map(1);
        self.uniforms.set_shadow_far(shadow_far);
        self.uniforms.set_light_pos_world(light_pos);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            self.uniforms.set_base_texture(0);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, shadow_map);
        }

        self.model.draw_instanced(self.instance_count as GLsizei);
    }
}

impl Drop for AsteroidField {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.static_ssbo);
            gl::DeleteBuffers(1, &self.dynamic_ssbo);
            gl::DeleteBuffers(1, &self.matrix_buffer);
        }
    }
}

