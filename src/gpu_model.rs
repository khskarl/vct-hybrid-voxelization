use crate::scene::model::Primitive;
use gl_helpers::*;
use glm::UVec3;
use nalgebra_glm as glm;
use std::rc::Rc;

pub struct GpuPrimitive {
	vertex_array: GLVertexArray,
	_vertex_buffer: GLBuffer,
	_index_buffer: Option<GLBuffer>,
	count_vertices: usize,
	material: Option<Rc<GpuMaterial>>,
	position: glm::Vec3,
	scale: glm::Vec3,
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
			_vertex_buffer: vertex_buffer,
			_index_buffer: None,
			count_vertices: (width * height * depth) as usize,
			material: None,
			position: glm::vec3(0.0, 0.0, 0.0),
			scale: glm::vec3(1.0, 1.0, 1.0),
		}
	}

	pub fn new(
		primitive: &Primitive,
		program: &GLProgram,
		material: Rc<GpuMaterial>,
		position: glm::Vec3,
		scale: glm::Vec3,
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

		for tangent in &primitive.tangents {
			buffer.push(tangent[0]);
			buffer.push(tangent[1]);
			buffer.push(tangent[2]);
		}

		let vertex_buffer = GLBuffer::new(BufferTarget::Array, 0, Usage::StaticDraw, &buffer);

		let mut vertex_array = GLVertexArray::new();
		vertex_array.bind();

		let positions_size = primitive.positions.len() * 3;
		let tex_coords_size = primitive.tex_coords.len() * 2;
		let normals_size = primitive.normals.len() * 3;

		vertex_array.add_attribute(&vertex_buffer, program.get_attribute("aPosition"), 0);
		vertex_array.add_attribute(
			&vertex_buffer,
			program.get_attribute("aTexCoord"),
			positions_size,
		);
		vertex_array.add_attribute(
			&vertex_buffer,
			program.get_attribute("aNormal"),
			positions_size + tex_coords_size,
		);
		vertex_array.add_attribute(
			&vertex_buffer,
			program.get_attribute("aTangent"),
			positions_size + tex_coords_size + normals_size,
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
			_vertex_buffer: vertex_buffer,
			_index_buffer: Some(index_buffer),
			count_vertices: primitive.indices.len(),
			material: Some(material),
			position,
			scale,
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

	pub fn model_matrix(&self) -> glm::Mat4 {
		let translation = glm::translation(&self.position);
		let scaling = glm::scaling(&self.scale);
		(translation * scaling)
	}

	pub fn model_matrix_raw(&self) -> [f32; 16] {
		let transmute_me: [[f32; 4]; 4] = self.model_matrix().into();
		unsafe { std::mem::transmute(transmute_me) }
	}

	pub fn translation_mut(&mut self) -> &mut glm::Vec3 {
		&mut self.position
	}

	pub fn scaling_mut(&mut self) -> &mut glm::Vec3 {
		&mut self.scale
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
