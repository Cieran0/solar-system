use glam::{Mat4, Vec3, Vec4};
use crate::{
    uniforms::Uniforms,
    rendering::{
        asteroid::AsteroidField,
        shadow::ShadowRenderer,
        skybox::Skybox,
    },
    simulation::solar_system::Renderable,
};

pub struct Renderer {
    pub shader_program: u32,
    pub instanced_shader_program: u32,
    pub uniforms: Uniforms,
    pub shadow_renderer: ShadowRenderer,
    pub skybox: Skybox,
    pub asteroid_field: AsteroidField,
    projection: Mat4,
    win_width: i32,
    win_height: i32,
}

impl Renderer {
    pub fn new(
        shader_program: u32,
        instanced_shader_program: u32,
        shadow_renderer: ShadowRenderer,
        skybox: Skybox,
        asteroid_field: AsteroidField,
        width: i32,
        height: i32,
    ) -> Self {
        let uniforms = Uniforms::new(shader_program);
        let projection = Mat4::perspective_rh(
            std::f32::consts::PI / 4.0,
            width as f32 / height as f32,
            0.1,
            100.0,
        );
        
        Self {
            shader_program,
            instanced_shader_program,
            uniforms,
            shadow_renderer,
            skybox,
            asteroid_field,
            projection,
            win_width: width,
            win_height: height,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.win_width = width;
        self.win_height = height;
        self.projection = Mat4::perspective_rh(
            std::f32::consts::PI / 4.0,
            width as f32 / height as f32,
            0.1,
            100.0,
        );
    }

    pub fn update_asteroids(&mut self, dt: f32) {
        self.asteroid_field.update(dt);
    }

    pub fn draw(&mut self, view: &Mat4, renderables: &[Renderable]) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        
        // Shadow pass (without asteroids)
        self.shadow_renderer.render_depth_pass(renderables);
        
        // Main rendering pass
        unsafe {
            gl::UseProgram(self.shader_program);
            gl::Viewport(0, 0, self.win_width, self.win_height);
        }
        
        self.uniforms.set_view_matrix(view);
        self.uniforms.set_projection_matrix(&self.projection);
        self.uniforms.set_light_pos(Vec4::new(0.0, 0.0, 0.0, 1.0));
        self.uniforms.set_light_pos_world(Vec3::ZERO);
        self.uniforms.set_shadow_map(1);
        self.uniforms.set_shadow_far(self.shadow_renderer.shadow_far);
        
        unsafe {
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.shadow_renderer.depth_cubemap);
        }
        
        // Draw skybox first
        self.skybox.draw(view, &self.projection);
        
        // Draw celestial bodies
        for r in renderables {
            self.uniforms.set_model_matrix(&r.model_matrix);
            self.uniforms.set_normal_matrix(&r.model_matrix);
            self.uniforms.set_emit_mode(r.emit_mode);
            self.uniforms.set_use_texture(r.use_texture);
            
            if r.use_texture {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, r.texture_id);
                }
                self.uniforms.set_base_texture(0);
            }
            
            r.geometry.draw();
        }
        
        // Draw asteroids
        self.asteroid_field.draw(
            view,
            &self.projection,
            Vec3::ZERO,
            self.shadow_renderer.depth_cubemap,
            self.shadow_renderer.shadow_far
        );
    }

    pub fn cleanup(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteProgram(self.instanced_shader_program);
        }
    }
}