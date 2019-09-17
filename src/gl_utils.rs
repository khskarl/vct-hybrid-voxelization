use gl::types::*;

use std::ffi::CStr;
use std::ffi::CString;
use std::ptr;
use std::str;

// Vertex data
pub static VERTEX_DATA: [GLfloat; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

// Shader sources
pub static VS_SRC: &'static str = "
#version 150
in vec2 position;

void main() {
		gl_Position = vec4(position, 0.0, 1.0);
}";

pub static FS_SRC: &'static str = "
#version 150
out vec4 out_color;

void main() {
		out_color = vec4(1.0, 1.0, 1.0, 1.0);
}";

pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
	let shader;
	unsafe {
		shader = gl::CreateShader(ty);
		// Attempt to compile the shader
		let c_str = CString::new(src.as_bytes()).unwrap();
		gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
		gl::CompileShader(shader);

		// Get the compile status
		let mut status = gl::FALSE as GLint;
		gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

		// Fail on error
		if status != (gl::TRUE as GLint) {
			let mut len = 0;
			gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
			let mut buf = Vec::with_capacity(len as usize);
			buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
			gl::GetShaderInfoLog(
				shader,
				len,
				ptr::null_mut(),
				buf.as_mut_ptr() as *mut GLchar,
			);
			panic!(
				"{}",
				str::from_utf8(&buf)
					.ok()
					.expect("ShaderInfoLog not valid utf8")
			);
		}
	}
	shader
}

pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
	unsafe {
		let program = gl::CreateProgram();
		gl::AttachShader(program, vs);
		gl::AttachShader(program, fs);
		gl::LinkProgram(program);
		// Get the link status
		let mut status = gl::FALSE as GLint;
		gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

		// Fail on error
		if status != (gl::TRUE as GLint) {
			let mut len: GLint = 0;
			gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
			let mut buf = Vec::with_capacity(len as usize);
			buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
			gl::GetProgramInfoLog(
				program,
				len,
				ptr::null_mut(),
				buf.as_mut_ptr() as *mut GLchar,
			);
			panic!(
				"{}",
				str::from_utf8(&buf)
					.ok()
					.expect("ProgramInfoLog not valid utf8")
			);
		}
		program
	}
}

pub fn print_opengl_diagnostics() {
	print_needed_extensions();
}

pub fn print_needed_extensions() {
	let needed_extensions = vec![
		"GL_EXT_texture3D",
		"GL_NV_conservative_raster",
		"GL_INTEL_conservative_rasterization",
	];

	println!("EXTENSIONS");
	println!("----------");
	for extension in needed_extensions {
		println!("{} : {}", extension, is_extension_supported(extension));
	}
}

#[allow(dead_code)]
pub fn print_extensions() {
	let mut num_extensions = 0;
	unsafe {
		gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions);
	}

	println!("Num of extensions: {}", num_extensions);

	for i in 0..num_extensions {
		unsafe {
			let extension_name = gl::GetStringi(gl::EXTENSIONS, i as u32) as *const i8;
			let extension_name = CStr::from_ptr(extension_name);
			println!("Extension {}: {:?}", i, extension_name);
		}
	}
}

pub fn is_extension_supported(extension: &str) -> bool {
	let mut num_extensions = 0;
	unsafe {
		gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions);
	}

	for i in 0..num_extensions {
		unsafe {
			let extension_name = gl::GetStringi(gl::EXTENSIONS, i as u32) as *const i8;
			let extension_name = CStr::from_ptr(extension_name).to_str().unwrap();

			if extension_name == extension {
				return true;
			}
		}
	}

	return false;
}
