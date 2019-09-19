use gl::types::*;
use gl_helpers::*;

use crate::gl_utils;
use crate::scene::camera::*;
use crate::scene::model::Model;

#[cfg_attr(rustfmt, rustfmt_skip)]
static VERTEX_DATA: [GLfloat; 20] = [
// pos        uv
	 1.0, -1.0, 0.0,
	-1.0, -1.0, 0.0,
	 1.0,  1.0, 0.0,
	-1.0,  1.0, 0.0,
	1.0, 0.0,
	0.0, 0.0,
	1.0, 1.0,
	0.0, 1.0,
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

		let inner_size = window_gl.window().inner_size();
		gl_set_viewport(0, 0, inner_size.width as usize, inner_size.height as usize);

		let program = GLProgram::new(gl_utils::VS_SRC, gl_utils::FS_SRC);

		let buffer = GLBuffer::new(BufferTarget::Array, 0, Usage::StaticDraw, &VERTEX_DATA);

		let mut vertex_array = GLVertexArray::new();
		vertex_array.bind();
		vertex_array.add_attribute(&buffer, program.get_attribute("position"), 0);
		vertex_array.add_attribute(&buffer, program.get_attribute("uv"), 3 * 4);

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

	pub fn submit_model(&mut self, model: &Model) {
		let mut buffer = Vec::<f32>::new();

		for position in &model.positions {
			buffer.push(position[0]);
			buffer.push(position[1]);
			buffer.push(position[2]);
		}

		for tex_coord in &model.tex_coords {
			buffer.push(tex_coord[0]);
			buffer.push(tex_coord[1]);
		}

		let vertex_buffer = GLBuffer::new(BufferTarget::Array, 0, Usage::StaticDraw, &buffer);

		let mut vertex_array = GLVertexArray::new();
		vertex_array.bind();
		vertex_array.add_attribute(
			&vertex_buffer,
			self.pbr_program.get_attribute("position"),
			0,
		);
		vertex_array.add_attribute(
			&vertex_buffer,
			self.pbr_program.get_attribute("uv"),
			model.positions.len() * 3,
		);

		let index_buffer = GLBuffer::new(
			BufferTarget::ElementArray,
			0,
			Usage::StaticDraw,
			&model.indices,
		);

		vertex_array.enable_attributes();
	}
}
