mod window;
mod camera;
mod shaders;
mod transform_stack;
mod input_handler;
mod shape;
mod obj;
mod shadow;

use std::{error::Error, fs::{self, read_to_string}, rc::Rc, time::Instant};
use glam::{Mat3, Mat4, Quat, Vec3, Vec4};
use crate::{
    camera::Camera,
    input_handler::InputHandler,
    shaders::{create_shader_program, set_current_program},
    shape::{Sphere, Shape, Cube},
    transform_stack::TransformStack,
    window::Window,
    obj::ObjModel,
    shadow::ShadowRenderer,
};

struct Renderable {
    geometry: Rc<dyn Shape>,
    model_matrix: Mat4,
    emit_mode: u32,
    use_texture: bool,
    texture_id: u32,
}

fn load_raw_rgba_texture(path: &str, width: u32, height: u32) -> Result<u32, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    assert_eq!(data.len(), (width * height * 4) as usize);
    let mut texture_id = 0;
    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _,
        );
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
    Ok(texture_id)
}

fn print_controls() {
    println!("=== Controls ===");
    println!("Camera movement: W/A/S/D + Space (up) / Left Shift (down)");
    println!("Camera views (Hold): 1 = Sun, 2 = Earth, 3 = Moon");
    println!("Simulation speed: '=' = faster, '-' = slower");
    println!("Spacecraft rotation: Arrow keys");
    println!("Spacecraft distance from Moon: PageUp / PageDown");
    println!("================");
}

fn main() -> Result<(), Box<dyn Error>> {
    print_controls();

    // initial logical size (these are the values the window was created with)
    let width = 1280i32;
    let height = 720i32;

    // track the *current* framebuffer size (update on resize)
    let mut win_width = width;
    let mut win_height = height;

    let mut window = Window::new(width as u32, height as u32, "Solar System")?;

    // enable depth testing once
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    // load render shader
    let vertex_src = read_to_string("shaders/render.vert")?;
    let fragment_src = read_to_string("shaders/render.frag")?;
    let shader_program = create_shader_program(&vertex_src, &fragment_src)?;
    set_current_program(shader_program);

    // get uniform locations once
    let model_loc = unsafe { gl::GetUniformLocation(shader_program, b"model\0".as_ptr() as *const _) };
    let view_loc = unsafe { gl::GetUniformLocation(shader_program, b"view\0".as_ptr() as *const _) };
    let proj_loc = unsafe { gl::GetUniformLocation(shader_program, b"projection\0".as_ptr() as *const _) };
    let normal_matrix_loc = unsafe { gl::GetUniformLocation(shader_program, b"normal_matrix\0".as_ptr() as *const _) };
    let light_pos_loc = unsafe { gl::GetUniformLocation(shader_program, b"light_pos\0".as_ptr() as *const _) };
    let light_pos_world_loc = unsafe { gl::GetUniformLocation(shader_program, b"light_pos_world\0".as_ptr() as *const _) };
    let emit_mode_loc = unsafe { gl::GetUniformLocation(shader_program, b"emit_mode\0".as_ptr() as *const _) };
    let use_texture_loc = unsafe { gl::GetUniformLocation(shader_program, b"use_texture\0".as_ptr() as *const _) };
    let base_texture_loc = unsafe { gl::GetUniformLocation(shader_program, b"base_texture\0".as_ptr() as *const _) };
    // shadow uniforms
    let shadow_map_loc = unsafe { gl::GetUniformLocation(shader_program, b"shadow_map\0".as_ptr() as *const _) };
    let shadow_far_loc = unsafe { gl::GetUniformLocation(shader_program, b"shadow_far\0".as_ptr() as *const _) };

    // load textures
    let earth_texture = load_raw_rgba_texture("textures/earth.rgba", 4096, 2048)?;
    let sun_texture = load_raw_rgba_texture("textures/sun.rgba", 4096, 2048)?;
    let moon_texture = load_raw_rgba_texture("textures/moon.rgba", 2048, 1024)?;

    // Create shadow renderer (this sets up the depth cubemap and FBO)
    let shadow_renderer = ShadowRenderer::new(Vec3::ZERO, 4096)?;

    // Camera & projection
    let mut camera = Camera::new(Vec3::new(0.0, 2.0, 5.0));
    camera.look_at(Vec3::ZERO);
    let mut projection = Mat4::perspective_rh(
        std::f32::consts::PI / 4.0,
        win_width as f32 / win_height as f32,
        0.1,
        100.0,
    );

    // Shapes
    let sun: Rc<dyn Shape> = Rc::new(Sphere::new(128, 128, Vec4::new(1.0, 0.8, 0.0, 1.0)));
    let earth: Rc<dyn Shape> = Rc::new(ObjModel::new("models/earth.obj")?);
    let moon: Rc<dyn Shape> = Rc::new(Sphere::new(128, 128, Vec4::new(0.5, 0.5, 0.5, 1.0)));
    let spacecraft: Rc<dyn Shape> = Rc::new(Cube::new());

    // constants
    const SUN_RADIUS: f32 = 1.0;
    const EARTH_RADIUS: f32 = 0.2;
    const MOON_RADIUS: f32 = 0.06;
    const EARTH_ORBIT_RADIUS: f32 = 3.0;
    const MOON_ORBIT_RADIUS: f32 = 0.4;
    const MOVE_SPEED: f32 = 2.0;
    const SPACECRAFT_SIZE: f32 = 0.03;
    const SPACECRAFT_ROT_SPEED: f32 = 1.5;

    // simulation state
    let mut sun_spin = 0.0f32;
    let mut earth_orbit = 0.0f32;
    let mut earth_spin = 0.0f32;
    let mut moon_orbit = 0.0f32;
    let mut moon_spin = 0.0f32;
    let mut spacecraft_rot = Quat::IDENTITY;
    let mut spacecraft_dist: f32 = 0.3;
    let mut sim_time_scale: f32 = 1.0;

    let mut input_handler = InputHandler::new();

    // initial viewport
    unsafe { gl::Viewport(0, 0, win_width, win_height); }

    let mut last_time = Instant::now();

    while !window.should_close() {
        let now = Instant::now();
        let delta = (now - last_time).as_secs_f32();
        last_time = now;

        window.poll_events();
        input_handler.handle_events(&mut window, &mut camera);

        // handle framebuffer resize events (update tracked window size and projection)
        if let Some((w, h)) = input_handler.framebuffer_size.take() {
            // store new current size
            win_width = w;
            win_height = h;
            unsafe { gl::Viewport(0, 0, win_width, win_height); }
            projection = Mat4::perspective_rh(
                std::f32::consts::PI / 4.0,
                win_width as f32 / win_height as f32,
                0.1,
                100.0,
            );
        }

        if input_handler.keys_pressed.contains(&glfw::Key::Equal) { sim_time_scale *= 1.2; }
        if input_handler.keys_pressed.contains(&glfw::Key::Minus) { sim_time_scale /= 1.2; }
        sim_time_scale = sim_time_scale.clamp(0.01, 10.0);
        let dt = delta * sim_time_scale;

        sun_spin += 0.1 * dt;
        earth_orbit += 0.5 * dt;
        earth_spin += 2.0 * dt;
        moon_orbit += 1.5 * dt;
        moon_spin += 1.5 * dt;

        let mut rot = Quat::IDENTITY;
        if input_handler.keys_pressed.contains(&glfw::Key::Left)  { rot = Quat::from_rotation_y(SPACECRAFT_ROT_SPEED * dt) * rot; }
        if input_handler.keys_pressed.contains(&glfw::Key::Right) { rot = Quat::from_rotation_y(-SPACECRAFT_ROT_SPEED * dt) * rot; }
        if input_handler.keys_pressed.contains(&glfw::Key::Up)    { rot = rot * Quat::from_rotation_x(SPACECRAFT_ROT_SPEED * dt); }
        if input_handler.keys_pressed.contains(&glfw::Key::Down)  { rot = rot * Quat::from_rotation_x(-SPACECRAFT_ROT_SPEED * dt); }
        spacecraft_rot = (rot * spacecraft_rot).normalize();
        if input_handler.keys_pressed.contains(&glfw::Key::PageDown) { spacecraft_dist = (spacecraft_dist - 0.1 * dt).max(0.05); }
        if input_handler.keys_pressed.contains(&glfw::Key::PageUp)   { spacecraft_dist += 0.1 * dt; }

        if input_handler.keys_pressed.contains(&glfw::Key::Num1) {
            camera.position = Vec3::new(0.0, 2.0, 5.0);
            camera.look_at(Vec3::ZERO);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Num2) {
            let earth_pos = Vec3::new(EARTH_ORBIT_RADIUS * earth_orbit.cos(), 0.0, EARTH_ORBIT_RADIUS * earth_orbit.sin());
            camera.position = earth_pos + Vec3::new(0.0, 0.5, 1.0);
            camera.look_at(earth_pos);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Num3) {
            let earth_pos = Vec3::new(EARTH_ORBIT_RADIUS * earth_orbit.cos(), 0.0, EARTH_ORBIT_RADIUS * earth_orbit.sin());
            let moon_pos = earth_pos + Vec3::new(MOON_ORBIT_RADIUS * moon_orbit.cos(), 0.0, MOON_ORBIT_RADIUS * moon_orbit.sin());
            camera.position = moon_pos + Vec3::new(0.0, 0.3, 0.6);
            camera.look_at(moon_pos);
        }

        let fwd = camera.forward();
        let right = fwd.cross(camera.up).normalize();
        let mut vel = Vec3::ZERO;
        if input_handler.keys_pressed.contains(&glfw::Key::W) { vel += fwd; }
        if input_handler.keys_pressed.contains(&glfw::Key::S) { vel -= fwd; }
        if input_handler.keys_pressed.contains(&glfw::Key::A) { vel -= right; }
        if input_handler.keys_pressed.contains(&glfw::Key::D) { vel += right; }
        if input_handler.keys_pressed.contains(&glfw::Key::Space)     { vel += camera.up; }
        if input_handler.keys_pressed.contains(&glfw::Key::LeftShift) { vel -= camera.up; }
        if vel.length_squared() > 0.0 {
            camera.position += vel.normalize() * MOVE_SPEED * delta;
        }

        let view = camera.view_matrix();

        // clear for this frame
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // === 1. BUILD RENDERABLES (with current transforms) ===
        let mut ts = TransformStack::new();
        let mut renderables = Vec::new();

        // SUN
        let sun_model = ts.current() * Mat4::from_rotation_y(sun_spin) * Mat4::from_scale(Vec3::splat(SUN_RADIUS));
        renderables.push(Renderable {
            geometry: Rc::clone(&sun),
            model_matrix: sun_model,
            emit_mode: 1,
            use_texture: true,
            texture_id: sun_texture,
        });

        // EARTH
        let earth_pos = Vec3::new(EARTH_ORBIT_RADIUS * earth_orbit.cos(), 0.0, EARTH_ORBIT_RADIUS * earth_orbit.sin());
        ts.push(Mat4::from_translation(earth_pos));
        let earth_model = ts.current() * Mat4::from_rotation_y(earth_spin) * Mat4::from_scale(Vec3::splat(EARTH_RADIUS));
        renderables.push(Renderable {
            geometry: Rc::clone(&earth),
            model_matrix: earth_model,
            emit_mode: 0,
            use_texture: true,
            texture_id: earth_texture,
        });

        // MOON
        let moon_pos = Vec3::new(MOON_ORBIT_RADIUS * moon_orbit.cos(), 0.0, MOON_ORBIT_RADIUS * moon_orbit.sin());
        ts.push(Mat4::from_translation(moon_pos));
        let moon_model = ts.current() * Mat4::from_rotation_y(moon_spin) * Mat4::from_scale(Vec3::splat(MOON_RADIUS));
        renderables.push(Renderable {
            geometry: Rc::clone(&moon),
            model_matrix: moon_model,
            emit_mode: 0,
            use_texture: true,
            texture_id: moon_texture,
        });

        // SPACECRAFT
        ts.push(Mat4::from_quat(spacecraft_rot));
        ts.push(Mat4::from_translation(Vec3::new(0.0, 0.0, spacecraft_dist)));
        let spacecraft_model = ts.current() * Mat4::from_scale(Vec3::new(SPACECRAFT_SIZE * 0.6, SPACECRAFT_SIZE * 0.5, SPACECRAFT_SIZE * 1.8));
        renderables.push(Renderable {
            geometry: Rc::clone(&spacecraft),
            model_matrix: spacecraft_model,
            emit_mode: 0,
            use_texture: false,
            texture_id: 0,
        });
        // Clean up transform stack
        for _ in 0..4 { ts.pop(); }

        // === 2. SHADOW DEPTH PASS (now that renderables exist) ===
        shadow_renderer.render_depth_pass(&renderables);

        // After depth pass, re-bind forward shader (in case depth pass changed current program)
        unsafe {
            gl::UseProgram(shader_program);
        }
        set_current_program(shader_program);

        // Restore viewport to tracked size (in case shadow pass changed it)
        unsafe {
            gl::Viewport(0, 0, win_width, win_height);
        }

        // === 3. FORWARD RENDER PASS ===
        unsafe {
            // set view / projection
            if view_loc >= 0 {
                gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.to_cols_array().as_ptr());
            }
            if proj_loc >= 0 {
                gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.to_cols_array().as_ptr());
            }

            // light in view space for the vertex shader (vec4)
            let light_world = Vec4::new(0.0, 0.0, 0.0, 1.0);
            let light_view = view * light_world;
            if light_pos_loc >= 0 {
                gl::Uniform4f(light_pos_loc, light_view.x, light_view.y, light_view.z, 1.0);
            }
            // also provide world-space light pos for shadow sampling in fragment shader
            if light_pos_world_loc >= 0 {
                gl::Uniform3f(light_pos_world_loc, 0.0, 0.0, 0.0);
            }

            // Bind shadow cubemap to texture unit 1 and set uniform (every frame to be safe)
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, shadow_renderer.depth_cubemap);
            if shadow_map_loc >= 0 {
                gl::Uniform1i(shadow_map_loc, 1);
            }
            if shadow_far_loc >= 0 {
                gl::Uniform1f(shadow_far_loc, shadow_renderer.shadow_far);
            }

            // Set viewport again just to be explicit (no-op if already set)
            gl::Viewport(0, 0, win_width, win_height);
        }

        for r in &renderables {
            // normal matrix must be inverse-transpose of model's upper-left 3x3 (not MV)
            let normal_matrix = Mat3::from_mat4(r.model_matrix).inverse().transpose();

            unsafe {
                if model_loc >= 0 {
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, r.model_matrix.to_cols_array().as_ptr());
                }
                if normal_matrix_loc >= 0 {
                    gl::UniformMatrix3fv(normal_matrix_loc, 1, gl::FALSE, normal_matrix.to_cols_array().as_ptr());
                }
                if emit_mode_loc >= 0 {
                    gl::Uniform1ui(emit_mode_loc, r.emit_mode);
                }
                if use_texture_loc >= 0 {
                    gl::Uniform1i(use_texture_loc, if r.use_texture { 1 } else { 0 });
                }

                if r.use_texture {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, r.texture_id);
                    if base_texture_loc >= 0 {
                        gl::Uniform1i(base_texture_loc, 0);
                    }
                }

                r.geometry.draw();
            }
        }

        window.swap_buffers();
    } // main loop

    // cleanup
    unsafe {
        gl::DeleteProgram(shader_program);
        gl::DeleteTextures(1, &earth_texture);
        gl::DeleteTextures(1, &sun_texture);
        gl::DeleteTextures(1, &moon_texture);
    }

    shadow_renderer.destroy();

    Ok(())
}
