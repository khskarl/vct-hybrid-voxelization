use gl::types::*;
use gl_helpers::*;

use crate::gl_utils;
use crate::scene::camera::*;

#[cfg_attr(rustfmt, rustfmt_skip)]
static VERTEX_DATA: [GLfloat; 16] = [
// pos        uv
	 1.0, -1.0, 1.0, 0.0,
	-1.0, -1.0, 0.0, 0.0,
	 1.0,  1.0, 1.0, 1.0,
	-1.0,  1.0, 0.0, 1.0,
];

#[derive(Debug)]
pub struct Renderer {
	vertex_buffer: GLBuffer,
	vertex_array: GLVertexArray,
	pbr_program: GLProgram,
}

impl Renderer {
	pub fn new(window_gl: &glutin::WindowedContext<glutin::PossiblyCurrent>) -> Renderer {
		gl::load_with(|symbol| window_gl.get_proc_address(symbol) as *const _);

		gl_utils::print_opengl_diagnostics();

		gl_set_defaults();
		gl_set_viewport(0, 0, 800, 600);

		let program = GLProgram::new(gl_utils::VS_SRC, gl_utils::FS_SRC);

		let buffer = GLBuffer::new(BufferTarget::Array, 4, Usage::StaticDraw, &VERTEX_DATA);

		let mut vertex_array = GLVertexArray::new();
		vertex_array.bind();
		vertex_array.add_attribute(&buffer, program.get_attribute("position"), 0);
		vertex_array.add_attribute(&buffer, program.get_attribute("uv"), 2);

		vertex_array.enable_attributes();

		program.get_uniform("time").set_1f(1.0_f32);

		Renderer {
			vertex_buffer: buffer,
			vertex_array,
			pbr_program: program,
		}
	}

	pub fn render(&self, camera: &Camera) {
		gl_set_clear_color(&[0.1, 0.1, 0.1, 1.0]);
		gl_clear(true, true, true);

		let proj: [f32; 16] = {
			let transmute_me: [[f32; 4]; 4] = camera.projection().into();
			unsafe { std::mem::transmute(transmute_me) }
		};

		let view: [f32; 16] = {
			let transmute_me: [[f32; 4]; 4] = camera.view().into();
			unsafe { std::mem::transmute(transmute_me) }
		};

		self.pbr_program.get_uniform("proj").set_mat4f(&proj);
		self.pbr_program.get_uniform("view").set_mat4f(&view);

		gl_draw_arrays(DrawMode::TriangleStrip, 0, 4);

		// gl::DrawElements()

		// void glDrawElements(	GLenum mode,
		// 	GLsizei count,
		// 	GLenum type,
		// 	const GLvoid * indices);
	}
}
