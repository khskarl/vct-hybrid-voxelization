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

		let primitive = GpuPrimitive::from_volume(
			[resolution as u32, resolution as u32, resolution as u32].into(),
			&program,
		);

		let mut handle = 0;
		unsafe {
			GenTextures(1, &mut handle);
			BindTexture(TEXTURE_3D, handle);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_S, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_T, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_R, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_MIN_FILTER, LINEAR as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_MAG_FILTER, LINEAR as i32);

			let mut pixels = Vec::<[u8; 4]>::new();
			for i in 0..resolution * resolution * resolution {
				let r = i % 155 + 100;
				let g = (i + 10) % 155 + 100;
				let b = (i + 20) % 155 + 100;
				let a = if (i % 5) == 0 { 255 } else { 0 };
				pixels.push([r as u8, g as u8, b as u8, a as u8]);
			}

			let raw_pixels: Vec<&u8> = pixels.iter().flatten().collect();

			TexImage3D(
				TEXTURE_3D,
				0,
				gl::RGBA as i32,
				resolution as i32,
				resolution as i32,
				resolution as i32,
				0,
				gl::RGBA,
				gl::UNSIGNED_BYTE,
				mem::transmute(raw_pixels[..].as_ptr()),
			);
		}

		Texture3D {
			id: handle,
			resolution,
			primitive,
			translation: glm::Vec3::new(-12.15, -0.4, -6.5),
			scaling: glm::Vec3::new(11.5, 6.0, 6.0),
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
