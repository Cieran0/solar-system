use glam::{Mat4, Vec3, Vec4};
use gl::types::GLuint;

use crate::shaders;

// Trait (interface) for shapes allowing for ::draw() to be called on any shape 
pub trait Shape {
    fn draw(&self, model: Mat4);
}

// Stores information for cube
pub struct Cube {
    vao: GLuint,
    vbo: [GLuint; 3], 
    vertex_count: i32,
}

// Constructor for Cube
impl Cube {
    pub fn new() -> Self {
        unsafe {
            // Create vertices from constants
            let positions: [Vec3; 36] = [
                
                Vec3::new(-0.25, 0.25, -0.25),
                Vec3::new(-0.25, -0.25, -0.25),
                Vec3::new(0.25, -0.25, -0.25),
                Vec3::new(0.25, -0.25, -0.25),
                Vec3::new(0.25, 0.25, -0.25),
                Vec3::new(-0.25, 0.25, -0.25),
                
                Vec3::new(0.25, -0.25, -0.25),
                Vec3::new(0.25, -0.25, 0.25),
                Vec3::new(0.25, 0.25, -0.25),
                Vec3::new(0.25, -0.25, 0.25),
                Vec3::new(0.25, 0.25, 0.25),
                Vec3::new(0.25, 0.25, -0.25),
                
                Vec3::new(0.25, -0.25, 0.25),
                Vec3::new(-0.25, -0.25, 0.25),
                Vec3::new(0.25, 0.25, 0.25),
                Vec3::new(-0.25, -0.25, 0.25),
                Vec3::new(-0.25, 0.25, 0.25),
                Vec3::new(0.25, 0.25, 0.25),
                
                Vec3::new(-0.25, -0.25, 0.25),
                Vec3::new(-0.25, -0.25, -0.25),
                Vec3::new(-0.25, 0.25, 0.25),
                Vec3::new(-0.25, -0.25, -0.25),
                Vec3::new(-0.25, 0.25, -0.25),
                Vec3::new(-0.25, 0.25, 0.25),
                
                Vec3::new(-0.25, -0.25, 0.25),
                Vec3::new(0.25, -0.25, 0.25),
                Vec3::new(0.25, -0.25, -0.25),
                Vec3::new(0.25, -0.25, -0.25),
                Vec3::new(-0.25, -0.25, -0.25),
                Vec3::new(-0.25, -0.25, 0.25),
                
                Vec3::new(-0.25, 0.25, -0.25),
                Vec3::new(0.25, 0.25, -0.25),
                Vec3::new(0.25, 0.25, 0.25),
                Vec3::new(0.25, 0.25, 0.25),
                Vec3::new(-0.25, 0.25, 0.25),
                Vec3::new(-0.25, 0.25, -0.25),
            ];

            // Colours setup now specifically for the spaceship            
            let mut colours = [Vec4::new(0.8, 0.85, 1.0, 1.0); 36]; 

            
            for i in 0..6 {
                colours[i] = Vec4::new(0.2, 0.4, 1.0, 1.0);
            }
            
            for i in 30..36 {
                colours[i] = Vec4::new(0.6, 0.6, 0.7, 1.0);
            }
            
            for i in 24..30 {
                colours[i] = Vec4::new(0.3, 0.3, 0.4, 1.0);
            }

            // Set normals to the same for each face (6 vertices at a time)            
            let mut normals = Vec::new();
            normals.extend(vec![Vec3::new(0.0, 0.0, -1.0); 6]);  
            normals.extend(vec![Vec3::new(1.0, 0.0, 0.0); 6]);   
            normals.extend(vec![Vec3::new(0.0, 0.0, 1.0); 6]);   
            normals.extend(vec![Vec3::new(-1.0, 0.0, 0.0); 6]);  
            normals.extend(vec![Vec3::new(0.0, -1.0, 0.0); 6]);  
            normals.extend(vec![Vec3::new(0.0, 1.0, 0.0); 6]);   

            // Generate vertex array and buffers
            let mut vao = 0;
            let mut vbo = [0u32; 3];
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(3, vbo.as_mut_ptr());

            gl::BindVertexArray(vao);

            // Bind position buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (positions.len() * std::mem::size_of::<Vec3>()) as isize,
                positions.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            // Bind colours buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[1]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (colours.len() * std::mem::size_of::<Vec4>()) as isize,
                colours.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(1);

            // Bind normals buffer            
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[2]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (normals.len() * std::mem::size_of::<Vec3>()) as isize,
                normals.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(2);

            gl::BindVertexArray(0);

            // Set variables in Cube object
            Self {
                vao,
                vbo,
                vertex_count: 36,
            }
        }
    }
}

// Stores information for sphere
pub struct Sphere {
    vao: GLuint,
    vbo: [GLuint; 3],
    vertex_count: i32,
}

impl Sphere {
    pub fn new(num_lats: usize, num_longs: usize, colour: Vec4) -> Self {
        // Generate data for the cube with helper function
        let (positions, normals, colours) = Self::generate_sphere(num_lats, num_longs, colour);
        let vertex_count = positions.len() as i32;

        unsafe {
            // Generate vertex array and buffers
            let mut vao = 0;
            let mut vbo = [0u32; 3];
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(3, vbo.as_mut_ptr());

            gl::BindVertexArray(vao);

            // Bind position buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (positions.len() * std::mem::size_of::<Vec3>()) as isize,
                positions.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            // Bind colours buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[1]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (colours.len() * std::mem::size_of::<Vec4>()) as isize,
                colours.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(1);

            // Bind normals buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[2]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (normals.len() * std::mem::size_of::<Vec3>()) as isize,
                normals.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(2);

            gl::BindVertexArray(0);

            Self { vao, vbo, vertex_count }
        }
    }

    //Generates a Sphere with the specified number of latitude and longitude segments
    fn generate_sphere(num_lats: usize, num_longs: usize, colour: Vec4) -> (Vec<Vec3>, Vec<Vec3>, Vec<Vec4>) {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut colours = Vec::new();

        // Generate vertices in a latitude longitude grid
        for lat in 0..=num_lats {
            // Polar angle
            let theta = (lat as f32) * std::f32::consts::PI / num_lats as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for lon in 0..=num_longs {
                // Horizontal angle
                let phi = (lon as f32) * 2.0 * std::f32::consts::PI / num_longs as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = sin_theta * cos_phi;
                let y = sin_theta * sin_phi;
                let z = cos_theta;

                // Push vertex data
                positions.push(Vec3::new(x, y, z));
                normals.push(Vec3::new(x, y, z)); 
                colours.push(colour);
            }
        }

        // Build indexed triangle list by connecting grid quads into two triangles each
        let mut final_positions = Vec::new();
        let mut final_normals = Vec::new();
        let mut final_colours = Vec::new();

        for lat in 0..num_lats {
            for lon in 0..num_longs {
                // Indices of the four corners of the current quad
                let current = lat * (num_longs + 1) + lon;
                let next = current + num_longs + 1;

                // Two triangles: (current, next, current+1) and (current+1, next, next+1)
                let indices = [current, next, current + 1, current + 1, next, next + 1];

                for &i in &indices {
                    final_positions.push(positions[i]);
                    final_normals.push(normals[i]);
                    final_colours.push(colours[i]);
                }
            }
        }

        (final_positions, final_normals, final_colours)
    }
}

// Implementaion of the Shape trait (::draw) for Cube
impl Shape for Cube {
    fn draw(&self, model: Mat4) {
        unsafe {
            // Get model uniform location from the current shader
            let model_loc = gl::GetUniformLocation(shaders::get_current_program(), b"model\0".as_ptr() as *const _);
            if model_loc != -1 {
                gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.to_cols_array().as_ptr());
            }
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
        }
    }
}

// Implementaion of the Shape trait (::draw) for Sphere
impl Shape for Sphere {
    fn draw(&self, model: Mat4) {
        unsafe {
            // Get model uniform location from the current shader
            let model_loc = gl::GetUniformLocation(shaders::get_current_program(), b"model\0".as_ptr() as *const _);
            if model_loc != -1 {
                gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.to_cols_array().as_ptr());
            }
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
        }
    }
}

// Cleanup opengl buffers when Cube is destroyed (RAII style)
impl Drop for Cube {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(3, self.vbo.as_ptr());
        }
    }
}

// Cleanup opengl buffers when Sphere is destroyed (RAII style)
impl Drop for Sphere {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(3, self.vbo.as_ptr());
        }
    }
}
