use crate::scene::material::Material;
use crate::scene::model::{Mesh, Primitive};
use gl_helpers::*;

pub struct GpuMaterial {
	albedo: GLTexture,
	metaghness: GLTexture,
	normal: GLTexture,
	occlusion: GLTexture,
}

impl GpuMaterial {
	pub fn from_material(material: &Material) -> GpuMaterial {
		GpuMaterial {
			albedo: load_gl_texture(&material.albedo()),
			metaghness: load_gl_texture(&material.metaghness()),
			normal: load_gl_texture(&material.normal()),
			occlusion: load_gl_texture(&material.occlusion()),
		}
	}

	pub fn albedo(&self) -> &GLTexture {
		&self.albedo
	}
	pub fn metaghness(&self) -> &GLTexture {
		&self.metaghness
	}
	pub fn normal(&self) -> &GLTexture {
		&self.normal
	}
	pub fn occlusion(&self) -> &GLTexture {
		&self.occlusion
	}
}

pub struct GpuMesh {
	primitives: Vec<GpuPrimitive>,
}

impl GpuMesh {
	pub fn new(model: &Mesh, program: &GLProgram) -> GpuMesh {
		let mut primitives = Vec::<GpuPrimitive>::new();

		for mesh in model.primitives() {
			primitives.push(GpuPrimitive::new(&mesh, &program));
		}

		GpuMesh { primitives }
	}

	pub fn primitives(&self) -> &Vec<GpuPrimitive> {
		&self.primitives
	}
}

pub struct GpuPrimitive {
	vertex_array: GLVertexArray,
	vertex_buffer: GLBuffer,
	index_buffer: GLBuffer,
	material: GpuMaterial,
	count_vertices: usize,
}

impl GpuPrimitive {
	pub fn new(primitive: &Primitive, program: &GLProgram) -> GpuPrimitive {
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

		let material = GpuMaterial::from_material(&primitive.material);

		GpuPrimitive {
			vertex_array,
			vertex_buffer,
			index_buffer,
			material,
			count_vertices: primitive.indices.len(),
		}
	}

	pub fn bind(&self) {
		self.vertex_array.bind();
	}

	pub const fn count_vertices(&self) -> usize {
		self.count_vertices
	}

	pub fn material(&self) -> &GpuMaterial {
		&self.material()
	}
}

fn load_gl_texture(image: &image::DynamicImage) -> GLTexture {
	use image::GenericImageView;

	let (width, height) = image.dimensions();
	let raw_pixels = &image.raw_pixels()[..];

	let gl_texture = GLTexture::new_2d(
		width as usize,
		height as usize,
		InternalFormat::RGB32F,
		DataFormat::RGB,
		DataKind::UnsignedByte,
		FilterMode::Linear,
		Wrap::Repeat,
		true,
		raw_pixels,
	);

	gl_texture
}
