use gl::types::*;

use std::ffi::CString;
use std::mem;
use std::ptr;

use crate::gl_utils;

#[derive(Debug)]
pub struct Renderer {}

impl Renderer {
	pub fn new(window_gl: &glutin::WindowedContext<glutin::PossiblyCurrent>) -> Renderer {
		Renderer::initialize_opengl(&window_gl);
		Renderer {}
	}

	fn initialize_opengl(window_gl: &glutin::WindowedContext<glutin::PossiblyCurrent>) {
		gl::load_with(|symbol| window_gl.get_proc_address(symbol) as *const _);

		gl_utils::print_opengl_diagnostics();

		let vs = gl_utils::compile_shader(gl_utils::VS_SRC, gl::VERTEX_SHADER);
		let fs = gl_utils::compile_shader(gl_utils::FS_SRC, gl::FRAGMENT_SHADER);
		let program = gl_utils::link_program(vs, fs);

		let (mut vao, mut vbo) = (0, 0);

		unsafe {
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);

			gl::GenBuffers(1, &mut vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(gl_utils::VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
				mem::transmute(&gl_utils::VERTEX_DATA[0]),
				gl::STATIC_DRAW,
			);

			gl::UseProgram(program);
			gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());

			let pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());
			gl::EnableVertexAttribArray(pos_attr as GLuint);
			gl::VertexAttribPointer(
				pos_attr as GLuint,
				2,
				gl::FLOAT,
				gl::FALSE as GLboolean,
				0,
				ptr::null(),
			);
		}
	}

	pub fn render(&self) {
		unsafe {
			gl::ClearColor(0.1, 0.1, 0.1, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);

			gl::DrawArrays(gl::TRIANGLES, 0, 3);

			// gl::DrawElements()

			// void glDrawElements(	GLenum mode,
			// 	GLsizei count,
			// 	GLenum type,
			// 	const GLvoid * indices);
		}
	}
}

impl Drop for Renderer {
	fn drop(&mut self) {
		unsafe {
			// gl::DeleteProgram(program);
			// gl::DeleteShader(fs);
			// gl::DeleteShader(vs);
			// gl::DeleteBuffers(1, &vbo);
			// gl::DeleteVertexArrays(1, &vao);
		}
	}
}
