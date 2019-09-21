use gl_helpers::*;
use crate::scene::model::{Mesh, Primitive};

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

#[derive(Debug)]
pub struct GpuPrimitive {
	vertex_array: GLVertexArray,
	vertex_buffer: GLBuffer,
	index_buffer: GLBuffer,
	color_texture: GLTexture,
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

		let vertex_buffer = GLBuffer::new(BufferTarget::Array, 0, Usage::StaticDraw, &buffer);

		let mut vertex_array = GLVertexArray::new();
		vertex_array.bind();
		vertex_array.add_attribute(&vertex_buffer, program.get_attribute("position"), 0);
		vertex_array.add_attribute(
			&vertex_buffer,
			program.get_attribute("uv"),
			primitive.positions.len() * 3,
		);

		let index_buffer = GLBuffer::new(
			BufferTarget::ElementArray,
			0,
			Usage::StaticDraw,
			&primitive.indices,
		);

		vertex_array.enable_attributes();

		let color_texture = load_gl_texture(&primitive.color_texture);

		GpuPrimitive {
			vertex_array,
			vertex_buffer,
			index_buffer,
			color_texture,
			count_vertices: primitive.indices.len(),
		}
	}

	pub fn bind(&self) {
		self.vertex_array.bind();
	}

	pub const fn count_vertices(&self) -> usize {
		self.count_vertices
	}

	pub const fn color_texture(&self) -> &GLTexture {
		&self.color_texture
	}
}

fn load_gl_texture(image: &image::DynamicImage) -> GLTexture {
	use image::GenericImageView;

	// let image = image.flipv();
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
