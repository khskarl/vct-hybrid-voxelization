use gl_helpers::*;
use crate::scene::model::Model;

#[derive(Debug)]
pub struct GpuModel {
	vertex_array: GLVertexArray,
	vertex_buffer: GLBuffer,
	index_buffer: GLBuffer,
	count_vertices: usize,
}

impl GpuModel {
	pub fn new(model: &Model, program: &GLProgram) -> GpuModel {
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
		vertex_array.add_attribute(&vertex_buffer, program.get_attribute("position"), 0);
		vertex_array.add_attribute(
			&vertex_buffer,
			program.get_attribute("uv"),
			model.positions.len() * 3,
		);

		let index_buffer = GLBuffer::new(
			BufferTarget::ElementArray,
			0,
			Usage::StaticDraw,
			&model.indices,
		);

		vertex_array.enable_attributes();

		GpuModel {
			vertex_array,
			vertex_buffer,
			index_buffer,
			count_vertices: model.indices.len(),
		}
	}

	pub fn bind(&self) {
		self.vertex_array.bind();
		self.vertex_buffer.bind();
		self.index_buffer.bind();
	}

	pub const fn count_vertices(&self) -> usize {
		self.count_vertices
	}
}
