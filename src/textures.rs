use gl;
use gl_helpers::*;

use glm::{UVec3, Vec3};
use nalgebra_glm as glm;

use std::{mem, ptr};

pub struct Texture3D {
	id: u32,
	resolution: UVec3,
}

impl Texture3D {
	pub fn new(resolution: UVec3) -> Texture3D {
		use gl::*;

		let mut handle = 0;

		unsafe {
			GenTextures(1, &mut handle);
			BindTexture(TEXTURE_3D, handle);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_S, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_T, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_R, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_MIN_FILTER, LINEAR as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_MAG_FILTER, LINEAR as i32);

			let pixels: Vec<u8> = (0..resolution.x * resolution.y * resolution.z)
				.map(|i| if (i / resolution.x) % 2 == 0 { 255 } else { 0 })
				.collect();

			TexImage3D(
				TEXTURE_3D,
				0,
				gl::RED as i32,
				resolution.x as i32,
				resolution.y as i32,
				resolution.z as i32,
				0,
				gl::RED,
				gl::UNSIGNED_BYTE,
				mem::transmute(pixels[..].as_ptr()),
			);
		}

		Texture3D {
			id: handle,
			resolution,
		}
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

	pub fn count_cells(&self) -> u32 {
		self.resolution.x * self.resolution.y * self.resolution.z
	}
}
