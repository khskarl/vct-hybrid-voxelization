use crate::gpu_model::GpuPrimitive;
use gl;
use gl_helpers::*;
use glm::UVec3;
use nalgebra_glm as glm;
use std::mem;

pub struct Volume {
	diffuse_id: u32,
	resolution: usize,
	primitive: GpuPrimitive,
	translation: glm::Vec3,
	scaling: glm::Vec3,
}

impl Volume {
	pub fn new(resolution: usize, program: &GLProgram) -> Volume {
		use gl::*;

		let primitive = GpuPrimitive::from_volume(
			[resolution as u32, resolution as u32, resolution as u32].into(),
			&program,
		);

		let mut handle = 0;
		unsafe {
			ActiveTexture(0);
			GenTextures(1, &mut handle);
			BindTexture(TEXTURE_3D, handle);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_S, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_T, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_WRAP_R, CLAMP_TO_BORDER as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_MIN_FILTER, LINEAR as i32);
			TexParameteri(TEXTURE_3D, TEXTURE_MAG_FILTER, LINEAR as i32);

			let mut pixels = Vec::<[u8; 4]>::new();
			for i in 0..resolution * resolution * resolution {
				let (r, g, b, a) = (0, 0, 0, 0);
				pixels.push([r as u8, g as u8, b as u8, a as u8]);
			}

			let mut raw_pixels = Vec::<u8>::new();
			for p in pixels {
				raw_pixels.push(p[0]);
				raw_pixels.push(p[1]);
				raw_pixels.push(p[2]);
				raw_pixels.push(p[3]);
			}
			let resolution = resolution as i32;
			// TexImage3D(
			// 	TEXTURE_3D,
			// 	0,
			// 	gl::RGBA8 as i32,
			// 	resolution,
			// 	resolution,
			// 	resolution,
			// 	0,
			// 	gl::RGBA,
			// 	gl::UNSIGNED_BYTE,
			// 	mem::transmute(raw_pixels[..].as_ptr()),
			// );

			TexStorage3D(
				TEXTURE_3D,
				1,
				gl::RGBA8 as u32,
				resolution,
				resolution,
				resolution,
			);
			TexSubImage3D(
				TEXTURE_3D,
				0,
				0,
				0,
				0,
				resolution,
				resolution,
				resolution,
				gl::RGBA,
				gl::UNSIGNED_BYTE,
				mem::transmute(raw_pixels[..].as_ptr()),
			);

			// gl::TexStorage3D(
			// 	GL_TEXTURE_3D,
			// 	1,
			// 	GL_RGBA8,
			// 	voxelResolution[0],
			// 	voxelResolution[1],
			// 	voxelResolution[2],
			// );
			// glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, image.width, image.height, GL_RGB, GL_UNSIGNED_BYTE, image.data.data());
		}

		Volume {
			diffuse_id: handle,
			resolution,
			primitive,
			// translation: glm::Vec3::new(-12.15, -0.4, -6.5),
			// scaling: glm::Vec3::new(11.5, 6.0, 6.0),
			translation: glm::Vec3::new(0.0, 0.0, 0.0),
			scaling: glm::Vec3::new(5.0, 5.0, 5.0),
		}
	}

	pub fn draw(&self) {
		self.primitive.bind();
		gl_draw_arrays(DrawMode::Points, 0, self.count_cells() as usize);
	}

	pub fn diffuse_id(&self) -> u32 {
		self.diffuse_id
	}

	pub fn set_sampler(&self, index: u32, location: u32) {
		unsafe {
			gl::ActiveTexture(gl::TEXTURE0 + index);
			gl::Uniform1i(location as i32, index as i32);
			gl::BindTexture(gl::TEXTURE_3D, self.diffuse_id());
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
