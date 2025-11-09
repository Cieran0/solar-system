mod window;
mod camera;
mod shaders;
mod transform_stack;
mod input_handler;
mod shape;

use std::{error::Error, fs::read_to_string, time::Instant};
use glam::{Mat3, Mat4, Quat, Vec3, Vec4};
use crate::{
    camera::Camera,
    input_handler::InputHandler,
    shaders::{create_shader_program, set_current_program},
    shape::{Sphere, Shape, Cube},
    transform_stack::TransformStack,
    window::Window,
};

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

    let width = 1920;
    let height = 1080;
    let mut window = Window::new(width, height, "Solar System")?;
    let vertex_src = read_to_string("poslight.vert")?;
    let fragment_src = read_to_string("poslight.frag")?;
    let shader_program = create_shader_program(&vertex_src, &fragment_src)?;
    set_current_program(shader_program);

    let mut camera = Camera::new(Vec3::new(0.0, 2.0, 5.0));
    camera.look_at(Vec3::ZERO);

    let projection = Mat4::perspective_rh(
        std::f32::consts::PI / 4.0,
        width as f32 / height as f32,
        0.1,
        100.0,
    );

    let model_loc = unsafe { gl::GetUniformLocation(shader_program, b"model\0".as_ptr() as *const _) };
    let view_loc = unsafe { gl::GetUniformLocation(shader_program, b"view\0".as_ptr() as *const _) };
    let proj_loc = unsafe { gl::GetUniformLocation(shader_program, b"projection\0".as_ptr() as *const _) };
    let normal_matrix_loc = unsafe { gl::GetUniformLocation(shader_program, b"normal_matrix\0".as_ptr() as *const _) };
    let light_pos_loc = unsafe { gl::GetUniformLocation(shader_program, b"light_pos\0".as_ptr() as *const _) };
    let emit_mode_loc = unsafe { gl::GetUniformLocation(shader_program, b"emit_mode\0".as_ptr() as *const _) };

    
    let sun = Sphere::new(128, 128, Vec4::new(1.0, 0.8, 0.0, 1.0));
    let earth = Sphere::new(128, 128, Vec4::new(0.0, 0.3, 1.0, 1.0));
    let moon = Sphere::new(128, 128, Vec4::new(0.5, 0.5, 0.5, 1.0));
    let spacecraft = Cube::new();

    const SUN_RADIUS: f32 = 1.0;
    const EARTH_RADIUS: f32 = 0.2;
    const MOON_RADIUS: f32 = 0.06;
    const EARTH_ORBIT_RADIUS: f32 = 3.0;
    const MOON_ORBIT_RADIUS: f32 = 0.4;
    const MOVE_SPEED: f32 = 2.0;
    const SPACECRAFT_SIZE: f32 = 0.03;
    const SPACECRAFT_ROT_SPEED: f32 = 1.5;

    let mut sun_spin = 0.0;
    let mut earth_orbit = 0.0;
    let mut earth_spin = 0.0;
    let mut moon_orbit = 0.0;
    let mut moon_spin = 0.0;
    let mut spacecraft_rot = Quat::IDENTITY;
    let mut spacecraft_dist: f32 = 0.3;
    let mut sim_time_scale: f32 = 1.0;

    let mut input_handler = InputHandler::new();
    let mut last_time = Instant::now();

    unsafe { gl::Viewport(0, 0, width as i32, height as i32); }

    while !window.should_close() {
        let now = Instant::now();
        let delta = (now - last_time).as_secs_f32();
        last_time = now;

        window.poll_events();
        input_handler.handle_events(&mut window, &mut camera);

        
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
        if input_handler.keys_pressed.contains(&glfw::Key::Left)  { rot = Quat::from_rotation_y(SPACECRAFT_ROT_SPEED*dt) * rot; }
        if input_handler.keys_pressed.contains(&glfw::Key::Right) { rot = Quat::from_rotation_y(-SPACECRAFT_ROT_SPEED*dt) * rot; }
        if input_handler.keys_pressed.contains(&glfw::Key::Up)    { rot = rot * Quat::from_rotation_x(SPACECRAFT_ROT_SPEED*dt); }
        if input_handler.keys_pressed.contains(&glfw::Key::Down)  { rot = rot * Quat::from_rotation_x(-SPACECRAFT_ROT_SPEED*dt); }
        spacecraft_rot = (rot * spacecraft_rot).normalize();
        if input_handler.keys_pressed.contains(&glfw::Key::PageDown)   { spacecraft_dist = (spacecraft_dist - 0.1*dt).max(0.05); }
        if input_handler.keys_pressed.contains(&glfw::Key::PageUp) { spacecraft_dist += 0.1*dt; }
        
        if input_handler.keys_pressed.contains(&glfw::Key::Num1) { camera.position = Vec3::new(0.0, 2.0, 5.0); camera.look_at(Vec3::ZERO); }
        if input_handler.keys_pressed.contains(&glfw::Key::Num2) {
            let earth_pos = Vec3::new(EARTH_ORBIT_RADIUS*earth_orbit.cos(),0.0,EARTH_ORBIT_RADIUS*earth_orbit.sin());
            camera.position = earth_pos + Vec3::new(0.0,0.5,1.0); camera.look_at(earth_pos);
        }
        if input_handler.keys_pressed.contains(&glfw::Key::Num3) {
            let earth_pos = Vec3::new(EARTH_ORBIT_RADIUS*earth_orbit.cos(),0.0,EARTH_ORBIT_RADIUS*earth_orbit.sin());
            let moon_pos = earth_pos + Vec3::new(MOON_ORBIT_RADIUS*moon_orbit.cos(),0.0,MOON_ORBIT_RADIUS*moon_orbit.sin());
            camera.position = moon_pos + Vec3::new(0.0,0.3,0.6); camera.look_at(moon_pos);
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
        if vel.length_squared() > 0.0 { camera.position += vel.normalize() * MOVE_SPEED * delta; }

        
        let view = camera.view_matrix();
        unsafe {
            gl::ClearColor(0.0,0.0,0.0,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UseProgram(shader_program);
            set_current_program(shader_program);
            gl::UniformMatrix4fv(view_loc,1,gl::FALSE,view.to_cols_array().as_ptr());
            gl::UniformMatrix4fv(proj_loc,1,gl::FALSE,projection.to_cols_array().as_ptr());
            let light_world = Vec4::new(0.0,0.0,0.0,1.0);
            let light_view = view * light_world;
            gl::Uniform4f(light_pos_loc,light_view.x,light_view.y,light_view.z,1.0);
        }
        
        let mut ts = TransformStack::new();

        unsafe { gl::Uniform1ui(emit_mode_loc,1); }
        let sun_model = ts.current() * Mat4::from_rotation_y(sun_spin) * Mat4::from_scale(Vec3::splat(SUN_RADIUS));
        let sun_mv = view * sun_model;
        let sun_normal = Mat3::from_mat4(sun_mv).inverse().transpose();
        unsafe {
            gl::UniformMatrix4fv(model_loc,1,gl::FALSE,sun_model.to_cols_array().as_ptr());
            gl::UniformMatrix3fv(normal_matrix_loc,1,gl::FALSE,sun_normal.to_cols_array().as_ptr());
        }
        sun.draw(sun_model);

        unsafe { gl::Uniform1ui(emit_mode_loc,0); }
        let earth_pos = Vec3::new(EARTH_ORBIT_RADIUS*earth_orbit.cos(),0.0,EARTH_ORBIT_RADIUS*earth_orbit.sin());
        ts.push(Mat4::from_translation(earth_pos));
        let earth_model = ts.current() * Mat4::from_rotation_y(earth_spin) * Mat4::from_scale(Vec3::splat(EARTH_RADIUS));
        let earth_mv = view * earth_model;
        let earth_normal = Mat3::from_mat4(earth_mv).inverse().transpose();
        unsafe {
            gl::UniformMatrix4fv(model_loc,1,gl::FALSE,earth_model.to_cols_array().as_ptr());
            gl::UniformMatrix3fv(normal_matrix_loc,1,gl::FALSE,earth_normal.to_cols_array().as_ptr());
        }
        earth.draw(earth_model);

        
        let moon_pos = Vec3::new(MOON_ORBIT_RADIUS*moon_orbit.cos(),0.0,MOON_ORBIT_RADIUS*moon_orbit.sin());
        ts.push(Mat4::from_translation(moon_pos));
        let moon_model = ts.current() * Mat4::from_rotation_y(moon_spin) * Mat4::from_scale(Vec3::splat(MOON_RADIUS));
        let moon_mv = view * moon_model;
        let moon_normal = Mat3::from_mat4(moon_mv).inverse().transpose();
        unsafe {
            gl::UniformMatrix4fv(model_loc,1,gl::FALSE,moon_model.to_cols_array().as_ptr());
            gl::UniformMatrix3fv(normal_matrix_loc,1,gl::FALSE,moon_normal.to_cols_array().as_ptr());
        }
        moon.draw(moon_model);
        
        ts.push(Mat4::from_quat(spacecraft_rot));
        ts.push(Mat4::from_translation(Vec3::new(0.0,0.0,spacecraft_dist)));
        let spacecraft_model = ts.current() * Mat4::from_scale(Vec3::new(SPACECRAFT_SIZE*0.6,SPACECRAFT_SIZE*0.5,SPACECRAFT_SIZE*1.8));
        let spacecraft_mv = view * spacecraft_model;
        let spacecraft_normal = Mat3::from_mat4(spacecraft_mv).inverse().transpose();
        unsafe {
            gl::UniformMatrix4fv(model_loc,1,gl::FALSE,spacecraft_model.to_cols_array().as_ptr());
            gl::UniformMatrix3fv(normal_matrix_loc,1,gl::FALSE,spacecraft_normal.to_cols_array().as_ptr());
        }
        spacecraft.draw(spacecraft_model);
        ts.pop(); 
        ts.pop(); 

        ts.pop(); 
        ts.pop(); 

        window.swap_buffers();
    }

    unsafe { gl::DeleteProgram(shader_program); }
    Ok(())
}
