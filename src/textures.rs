use crate::gpu_model::GpuPrimitive;
use gl;
use gl_helpers::*;
use glm::UVec3;
use nalgebra_glm as glm;
use std::mem;

pub struct Texture3D {
	id: u32,
	resolution: usize,
	primitive: GpuPrimitive,
	translation: glm::Vec3,
	scaling: glm::Vec3,
}

impl Texture3D {
	pub fn new(resolution: usize, program: &GLProgram) -> Texture3D {
		use gl::*;

		let primitive = GpuPrimitive::from_volume([512, 512, 512].into(), &program);

		let mut handle = 0;
		unsafe {
			GenTextures(1, &mut handle);
			BindTexture(TEXTURE_3D, handle);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_S, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_T, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_R, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_MIN_FILTER, LINEAR as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_MAG_FILTER, LINEAR as i32);

			let pixels: Vec<u8> = (0..resolution * resolution * resolution)
				.map(|i| if (i / resolution) % 2 == 0 { 255 } else { 0 })
				.collect();

			TexImage3D(
				TEXTURE_3D,
				0,
				gl::RED as i32,
				resolution as i32,
				resolution as i32,
				resolution as i32,
				0,
				gl::RED,
				gl::UNSIGNED_BYTE,
				mem::transmute(pixels[..].as_ptr()),
			);
		}

		Texture3D {
			id: handle,
			resolution,
			primitive,
			translation: glm::Vec3::new(0.0, 0.0, 0.0),
			scaling: glm::Vec3::new(1.0, 1.0, 1.0),
		}
	}

	pub fn draw(&self) {
		self.bind();
		self.primitive.bind();
		gl_draw_arrays(DrawMode::Points, 0, self.count_cells() as usize);
	}

	pub fn bind(&self) {
		unsafe {
			gl::BindTexture(gl::TEXTURE_3D, self.id);
		}
	}

	pub fn set_sampler(&self, index: u32, location: u32) {
		unsafe {
			gl::ActiveTexture(gl::TEXTURE0 + index);
			gl::Uniform1i(location as i32, index as i32);
			gl::BindTexture(gl::TEXTURE_3D, self.id);
		}
	}

	pub fn count_cells(&self) -> usize {
		self.resolution * self.resolution * self.resolution
	}

	pub fn resolution(&self) -> usize {
		self.resolution
	}

	pub fn resolution_mut(&mut self) -> &mut usize {
		&mut self.resolution
	}

	pub const fn translation(&self) -> &glm::Vec3 {
		&self.translation
	}

	pub const fn scaling(&self) -> &glm::Vec3 {
		&self.scaling
	}

	pub fn translation_mut(&mut self) -> &mut glm::Vec3 {
		&mut self.translation
	}

	pub fn scaling_mut(&mut self) -> &mut glm::Vec3 {
		&mut self.scaling
	}
}
