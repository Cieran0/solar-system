use std::{cell::Cell, ffi::{CStr, CString}};

// Hold current shader so accessible anywhere
// Global variables in rust are weird due to forced thread safety
thread_local! {
    static CURRENT_PROGRAM: Cell<u32> = Cell::new(0);
}

pub fn set_current_program(program: u32) {
    CURRENT_PROGRAM.with(|p| p.set(program));
}

pub fn get_current_program() -> u32 {
    CURRENT_PROGRAM.with(|p| p.get())
}

// Compile shader from a string
pub fn compile_shader(source: &str, shader_type: u32) -> Result<u32, String> {
    let shader = unsafe { gl::CreateShader(shader_type) };
    let c_source = CString::new(source).expect("Shader source must be valid UTF-8");
    unsafe {
        gl::ShaderSource(shader, 1, &c_source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
        let mut success: i32 = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        // Error out if compiling shader fails 
        if success != i32::from(gl::TRUE) {
            let mut len: i32 = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut error = vec![0; len as usize];
            gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), error.as_mut_ptr());
            let error = CStr::from_ptr(error.as_ptr()).to_str().unwrap().to_string();

            // Return why the shader failed to compile
            return Err(format!("Shader compilation failed: {}", error));
        }
    }
    Ok(shader)
}

// Create shader program from the shader source strings
pub fn create_shader_program(vertex_src: &str, fragment_src: &str) -> Result<u32, String> {
    let vertex_shader = compile_shader(vertex_src, gl::VERTEX_SHADER)?;
    let fragment_shader = compile_shader(fragment_src, gl::FRAGMENT_SHADER)?;

    // Create program and attach shaders
    let program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);
        let mut success: i32 = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            let mut len: i32 = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut error = vec![0; len as usize];
            gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), error.as_mut_ptr());
            let error = CStr::from_ptr(error.as_ptr()).to_str().unwrap().to_string();
            // Return error if the shader failed to link to program
            return Err(format!("Shader linking failed: {}", error));
        }
        
        // Cleanup
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }
    Ok(program)
}