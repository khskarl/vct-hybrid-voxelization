use crate::gpu_model::GpuPrimitive;
use gl;
use gl_helpers::*;
use glm::UVec3;
use nalgebra_glm as glm;
use std::mem;
use crate::renderer_utils::*;

pub struct Volume {
	albedo_id: u32,
	normal_id: u32,
	emission_id: u32,
	radiance_id: u32,
	resolution: usize,
	primitive: GpuPrimitive,
	translation: glm::Vec3,
	scaling: glm::Vec3,
	view_translation: glm::Vec3,
	view_scaling: glm::Vec3,
	mipmap_program: GLProgram,
}

impl Volume {
	pub fn new(resolution: usize, program: &GLProgram) -> Volume {
		let primitive = GpuPrimitive::from_volume(
			[resolution as u32, resolution as u32, resolution as u32].into(),
			&program,
		);

		Volume {
			albedo_id: allocate_texture_3D(resolution, 1),
			normal_id: allocate_texture_3D(resolution, 1),
			emission_id: allocate_texture_3D(resolution, 1),
			radiance_id: allocate_texture_3D(resolution, 9),
			resolution,
			primitive,
			translation: glm::Vec3::new(0.0, 5.0, 0.0),
			scaling: glm::Vec3::new(10.0, 10.0, 10.0),
			view_translation: glm::Vec3::new(10.15, 5.0, 0.0),
			view_scaling: glm::Vec3::new(10.0, 10.0, 10.0),
			mipmap_program: load_mipmap_program(),
		}
	}

	pub fn draw(&self) {
		self.primitive.bind();
		gl_draw_arrays(DrawMode::Points, 0, self.count_cells() as usize);
	}

	pub fn albedo_id(&self) -> u32 {
		self.albedo_id
	}

	pub fn normal_id(&self) -> u32 {
		self.normal_id
	}

	pub fn emission_id(&self) -> u32 {
		self.emission_id
	}

	pub fn radiance_id(&self) -> u32 {
		self.radiance_id
	}

	pub fn bind_image_albedo(&self, index: u32) {
		unsafe {
			gl::BindImageTexture(
				index,
				self.albedo_id(),
				0,
				gl::TRUE,
				0,
				gl::READ_WRITE,
				gl::RGBA8,
			);
		}
	}

	pub fn bind_image_normal(&self, index: u32) {
		unsafe {
			gl::BindImageTexture(
				index,
				self.normal_id(),
				0,
				gl::TRUE,
				0,
				gl::READ_WRITE,
				gl::RGBA8,
			);
		}
	}

	pub fn bind_image_emission(&self, index: u32) {
		unsafe {
			gl::BindImageTexture(
				index,
				self.emission_id(),
				0,
				gl::TRUE,
				0,
				gl::READ_WRITE,
				gl::RGBA8,
			);
		}
	}

	pub fn bind_image_radiance(&self, index: u32) {
		unsafe {
			gl::BindImageTexture(
				index,
				self.radiance_id(),
				0,
				gl::TRUE,
				0,
				gl::READ_WRITE,
				gl::RGBA8,
			);
		}
	}

	pub fn bind_texture_albedo(&self, index: u32) {
		unsafe {
			gl::BindTextureUnit(index, self.albedo_id());
		}
	}

	pub fn bind_texture_normal(&self, index: u32) {
		unsafe {
			gl::BindTextureUnit(index, self.normal_id());
		}
	}

	pub fn bind_texture_emission(&self, index: u32) {
		unsafe {
			gl::BindTextureUnit(index, self.emission_id());
		}
	}

	pub fn bind_texture_radiance(&self, index: u32) {
		unsafe {
			gl::BindTextureUnit(index, self.radiance_id());
		}
	}

	pub fn generate_mipmap(&self) {
		unsafe {
			// gl::GenerateTextureMipmap(self.albedo_id());
			// gl::GenerateTextureMipmap(self.normal_id());
			// gl::GenerateTextureMipmap(self.emission_id());
			// gl::GenerateTextureMipmap(self.radiance_id());
			// gl::GenerateTextureMipmap(self.radiance_id());
			// gl::BindTexture(gl::TEXTURE_3D, self.radiance_id());
			// gl::GenerateMipmap(gl::TEXTURE_3D);
		}

		// self.mipmap_program.bind();

		
		// gl::ActiveTexture(gl::TEXTURE0 + index);
		// gl::BindTexture(gl::TEXTURE_3D, self.radiance_id());

		// let resolution = self.resolution();

		// unsafe {
		// 	gl::DispatchCompute(
		// 		resolution as u32 / 8,
		// 		resolution as u32 / 8,
		// 		resolution as u32 / 8,
		// 	);

		// 	gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
		// }
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

	pub const fn view_translation(&self) -> &glm::Vec3 {
		&self.view_translation
	}

	pub const fn view_scaling(&self) -> &glm::Vec3 {
		&self.view_scaling
	}

	pub fn translation_mut(&mut self) -> &mut glm::Vec3 {
		&mut self.translation
	}

	pub fn scaling_mut(&mut self) -> &mut glm::Vec3 {
		&mut self.scaling
	}

	pub fn view_translation_mut(&mut self) -> &mut glm::Vec3 {
		&mut self.view_translation
	}

	pub fn view_scaling_mut(&mut self) -> &mut glm::Vec3 {
		&mut self.view_scaling
	}
}

pub fn allocate_texture_3D(resolution: usize, mipmap: usize) -> u32 {
	use gl::*;

	let mut handle = 0;
	unsafe {
		GenTextures(1, &mut handle);
		BindTexture(TEXTURE_3D, handle);
		TexParameteri(TEXTURE_3D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as i32);
		TexParameteri(TEXTURE_3D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as i32);
		TexParameteri(TEXTURE_3D, TEXTURE_WRAP_R, CLAMP_TO_EDGE as i32);
		TexParameteri(TEXTURE_3D, TEXTURE_MIN_FILTER, LINEAR as i32);
		TexParameteri(TEXTURE_3D, TEXTURE_MAG_FILTER, LINEAR as i32);

		let mut pixels = Vec::<[u8; 4]>::new();
		for i in 0..resolution * resolution * resolution {
			let (r, g, b, a) = (1, 0, 1, 1);
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
		TexStorage3D(
			TEXTURE_3D,
			mipmap as i32,
			gl::RGBA8 as u32,
			resolution,
			resolution,
			resolution,
		);

		for level in 0..mipmap {
			TexSubImage3D(
				TEXTURE_3D,
				level as i32,
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
		}
	}

	handle
}
