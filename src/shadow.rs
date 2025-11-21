use std::{fs, ptr};
use gl::types::{GLenum, GLuint};
use glam::{Mat4, Vec3};

pub struct ShadowRenderer {
    pub depth_cubemap: GLuint,
    pub depth_fbo: GLuint,
    pub shader: GLuint, // depth-pass program (vertex+geom+frag)
    light_pos: Vec3,
    pub shadow_far: f32,
    size: i32,
}

impl ShadowRenderer {
    pub fn new(light_pos: Vec3, size: i32) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            // --- Create cubemap depth texture ---
            let mut depth_cubemap: GLuint = 0;
            gl::GenTextures(1, &mut depth_cubemap);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, depth_cubemap);

            for i in 0..6 {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum,
                    0,
                    gl::DEPTH_COMPONENT32F as i32,
                    size,
                    size,
                    0,
                    gl::DEPTH_COMPONENT,
                    gl::FLOAT,
                    ptr::null(),
                );
            }

            // filtering/wrap
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);

            // Use hardware compare mode (we will store normalized distance in gl_FragDepth)
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_COMPARE_MODE,
                gl::COMPARE_REF_TO_TEXTURE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_COMPARE_FUNC,
                gl::LEQUAL as i32,
            );

            // Unbind texture for now
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);

            // --- Create FBO and attach cubemap as depth attachment ---
            let mut depth_fbo: GLuint = 0;
            gl::GenFramebuffers(1, &mut depth_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_fbo);
            // Attach the cubemap (driver will accept layered rendering once we set gl_Layer in the geometry shader)
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, depth_cubemap, 0);
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            // Check completeness
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                // Unbind
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                return Err(format!("Depth FBO incomplete: 0x{:X}", status).into());
            }

            // Unbind FBO
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // --- Load & compile depth-pass shaders ---
            let vert_src = fs::read_to_string("shaders/shadow_cubemap.vert")?;
            let geom_src = fs::read_to_string("shaders/shadow_cubemap.geom")?;
            let frag_src = fs::read_to_string("shaders/shadow_cubemap.frag")?;

            let vert_shader =
                crate::shaders::compile_shader(&vert_src, gl::VERTEX_SHADER)?;
            let geom_shader =
                crate::shaders::compile_shader(&geom_src, gl::GEOMETRY_SHADER)?;
            let frag_shader =
                crate::shaders::compile_shader(&frag_src, gl::FRAGMENT_SHADER)?;

            let shader = gl::CreateProgram();
            gl::AttachShader(shader, vert_shader);
            gl::AttachShader(shader, geom_shader);
            gl::AttachShader(shader, frag_shader);
            gl::LinkProgram(shader);

            // Check link status
            let mut success: i32 = 0;
            gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
            if success != (gl::TRUE as i32) {
                let mut len: i32 = 0;
                gl::GetProgramiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf: Vec<u8> = vec![0u8; (len.max(1) as usize)];
                gl::GetProgramInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut i8,
                );
                let msg = String::from_utf8_lossy(&buf).into_owned();
                return Err(format!("Shadow shader linking failed: {}", msg).into());
            }

            // Clean up shaders
            gl::DeleteShader(vert_shader);
            gl::DeleteShader(geom_shader);
            gl::DeleteShader(frag_shader);

            Ok(Self {
                depth_cubemap,
                depth_fbo,
                shader,
                light_pos,
                shadow_far: 100.0,
                size,
            })
        }
    }

    pub fn get_light_space_matrices(&self) -> [Mat4; 6] {
        let far = self.shadow_far;
        // 90 degree fov, aspect 1, near 0.1, far = shadow_far
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, 0.1, far);

        // Standard cube face targets & ups
        let pos = self.light_pos;
        let views = [
            // +X
            Mat4::look_at_rh(pos, pos + Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
            // -X
            Mat4::look_at_rh(pos, pos + Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
            // +Y
            Mat4::look_at_rh(pos, pos + Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
            // -Y
            Mat4::look_at_rh(pos, pos + Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
            // +Z
            Mat4::look_at_rh(pos, pos + Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, -1.0, 0.0)),
            // -Z
            Mat4::look_at_rh(pos, pos + Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, -1.0, 0.0)),
        ];

        [
            proj * views[0],
            proj * views[1],
            proj * views[2],
            proj * views[3],
            proj * views[4],
            proj * views[5],
        ]
    }

    pub fn render_depth_pass(&self, renderables: &[super::Renderable]) {
        unsafe {
            // Use depth-pass program
            gl::UseProgram(self.shader);

            // Set uniforms: light_pos and shadow_far
            let light_pos_loc =
                gl::GetUniformLocation(self.shader, b"light_pos\0".as_ptr() as *const _);
            if light_pos_loc >= 0 {
                gl::Uniform3f(light_pos_loc, self.light_pos.x, self.light_pos.y, self.light_pos.z);
            }
            let far_loc =
                gl::GetUniformLocation(self.shader, b"shadow_far\0".as_ptr() as *const _);
            if far_loc >= 0 {
                gl::Uniform1f(far_loc, self.shadow_far);
            }

            // Upload 6 light matrices
            let matrices = self.get_light_space_matrices();
            for i in 0..6usize {
                let name_str = format!("light_matrix[{}]\0", i);
                let loc = gl::GetUniformLocation(self.shader, name_str.as_ptr() as *const _);
                if loc >= 0 {
                    // Mat4::to_cols_array gives column-major floats (gl expects column-major)
                    let arr = matrices[i].to_cols_array();
                    gl::UniformMatrix4fv(loc, 1, gl::FALSE, arr.as_ptr());
                }
            }

            // Bind FBO and viewport
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);
            gl::Viewport(0, 0, self.size, self.size);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            // Render each object; geometry shader writes into the cubemap layers
            for r in renderables {
                if r.emit_mode == 1 {
                    continue; // skip emissive objects for the shadow map if desired
                }
                let model_loc =
                    gl::GetUniformLocation(self.shader, b"model\0".as_ptr() as *const _);
                if model_loc >= 0 {
                    gl::UniformMatrix4fv(
                        model_loc,
                        1,
                        gl::FALSE,
                        r.model_matrix.to_cols_array().as_ptr(),
                    );
                }
                r.geometry.draw();
            }

            // unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn destroy(&self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.depth_fbo);
            gl::DeleteTextures(1, &self.depth_cubemap);
            gl::DeleteProgram(self.shader);
        }
    }
}
