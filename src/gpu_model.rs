use crate::scene::model::Primitive;
use gl_helpers::*;
use glm::UVec3;
use nalgebra_glm as glm;
use std::rc::Rc;

pub struct GpuPrimitive {
	vertex_array: GLVertexArray,
	vertex_buffer: GLBuffer,
	index_buffer: Option<GLBuffer>,
	count_vertices: usize,
	material: Option<Rc<GpuMaterial>>,
}

impl GpuPrimitive {
	pub fn from_volume(resolution: UVec3, program: &GLProgram) -> GpuPrimitive {
		let mut buffer = Vec::<f32>::new();

		let (width, height, depth) = (resolution.x, resolution.y, resolution.z);
		for i in 0..width * height * depth {
			buffer.push(0.0);
			buffer.push(0.0);
			buffer.push(0.0);
		}

		let vertex_buffer = GLBuffer::new(BufferTarget::Array, 0, Usage::StaticDraw, &buffer);

		let mut vertex_array = GLVertexArray::new();
		vertex_array.bind();
		vertex_array.add_attribute(&vertex_buffer, program.get_attribute("aPosition"), 0);
		vertex_array.enable_attributes();

		GpuPrimitive {
			vertex_array,
			vertex_buffer,
			index_buffer: None,
			count_vertices: (width * height * depth) as usize,
			material: None,
		}
	}

	pub fn new(
		primitive: &Primitive,
		program: &GLProgram,
		material: Rc<GpuMaterial>,
	) -> GpuPrimitive {
		let mut buffer = Vec::<f32>::new();

		for position in &primitive.positions {
			buffer.push(position[0]);
			buffer.push(position[1]);
			buffer.push(position[2]);
		}

		for tex_coord in &primitive.tex_coords {
			buffer.push(tex_coord[0]);
			buffer.push(tex_coord[1]);
		}

		for normal in &primitive.normals {
			buffer.push(normal[0]);
			buffer.push(normal[1]);
			buffer.push(normal[2]);
		}

		let vertex_buffer = GLBuffer::new(BufferTarget::Array, 0, Usage::StaticDraw, &buffer);

		let mut vertex_array = GLVertexArray::new();
		vertex_array.bind();
		vertex_array.add_attribute(&vertex_buffer, program.get_attribute("aPosition"), 0);
		vertex_array.add_attribute(
			&vertex_buffer,
			program.get_attribute("aTexCoord"),
			primitive.positions.len() * 3,
		);
		vertex_array.add_attribute(
			&vertex_buffer,
			program.get_attribute("aNormal"),
			primitive.positions.len() * 3 + primitive.tex_coords.len() * 2,
		);

		let index_buffer = GLBuffer::new(
			BufferTarget::ElementArray,
			0,
			Usage::StaticDraw,
			&primitive.indices,
		);

		vertex_array.enable_attributes();

		GpuPrimitive {
			vertex_array,
			vertex_buffer,
			index_buffer: Some(index_buffer),
			count_vertices: primitive.indices.len(),
			material: Some(material),
		}
	}

	pub fn bind(&self) {
		self.vertex_array.bind();
	}

	pub const fn count_vertices(&self) -> usize {
		self.count_vertices
	}

	pub fn material(&self) -> Rc<GpuMaterial> {
		Rc::clone(&self.material.as_ref().unwrap())
	}
}

pub struct GpuMaterial {
	albedo: Rc<GLTexture>,
	metaghness: Rc<GLTexture>,
	normal: Rc<GLTexture>,
	occlusion: Rc<GLTexture>,
}

impl GpuMaterial {
	pub fn new(
		albedo: Rc<GLTexture>,
		metaghness: Rc<GLTexture>,
		normal: Rc<GLTexture>,
		occlusion: Rc<GLTexture>,
	) -> GpuMaterial {
		GpuMaterial {
			albedo,
			metaghness,
			normal,
			occlusion,
		}
	}

	pub fn albedo(&self) -> &Rc<GLTexture> {
		&self.albedo
	}
	pub fn metaghness(&self) -> &Rc<GLTexture> {
		&self.metaghness
	}
	pub fn normal(&self) -> &Rc<GLTexture> {
		&self.normal
	}
	pub fn occlusion(&self) -> &Rc<GLTexture> {
		&self.occlusion
	}
}
