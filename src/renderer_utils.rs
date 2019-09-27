use crate::scene::material::{Material, Texture};
use gl_helpers::*;
use nalgebra_glm as glm;
use std::fs;

////////////////////
// SHADER HELPERS //
static VERTEX_EXPECT: &'static str = "Couldn't read the vertex shader :(";
static GEOMETRY_EXPECT: &'static str = "Couldn't read the geometry shader :(";
static FRAGMENT_EXPECT: &'static str = "Couldn't read the fragment shader :(";

pub fn load_pbr_program() -> GLProgram {
	let vs_src = fs::read_to_string("src/shaders/pbr.vert").expect(VERTEX_EXPECT);
	let fs_src = fs::read_to_string("src/shaders/pbr.frag").expect(FRAGMENT_EXPECT);

	GLProgram::new(&vs_src[..], &fs_src[..])
}

pub fn load_depth_program() -> GLProgram {
	let vs_src = fs::read_to_string("src/shaders/depth_pass.vert").expect(VERTEX_EXPECT);
	let fs_src = fs::read_to_string("src/shaders/depth_pass.frag").expect(FRAGMENT_EXPECT);

	GLProgram::new(&vs_src[..], &fs_src[..])
}

pub fn load_voxel_view_program() -> GLProgram {
	let vs_src = fs::read_to_string("src/shaders/voxel_view.vert").expect(VERTEX_EXPECT);
	let fs_src = fs::read_to_string("src/shaders/voxel_view.frag").expect(FRAGMENT_EXPECT);

	GLProgram::new(&vs_src[..], &fs_src[..])
}

//////////////////////
// MATERIAL HELPERS //
pub fn load_texture(texture: &Texture) -> GLTexture {
	use image::GenericImage;

	let (width, height) = texture.image().dimensions();
	let raw_pixels = &texture.image().raw_pixels()[..];

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

///////////////////
// LIGHT HELPERS //
pub struct Light {
	pub direction: glm::Vec3,
	pub position: glm::Vec3,
	pub color: glm::Vec3,
	pub intensity: f32,
}

pub fn load_lights() -> Vec<Light> {
	let mut lights = Vec::new();
	lights.push(Light {
		direction: glm::vec3(0.05, -0.9, -0.2),
		position: glm::vec3(0.0, 2.0, 0.0),
		color: glm::vec3(1.0, 1.0, 1.0),
		intensity: 4.0,
	});
	lights.push(Light {
		direction: glm::vec3(0.0, 0.0, 0.0),
		position: glm::vec3(9.0, 2.0, 0.0),
		color: glm::vec3(0.058, 0.513, 0.415),
		intensity: 1.0,
	});
	lights.push(Light {
		direction: glm::vec3(0.0, 0.0, 0.0),
		position: glm::vec3(-9.0, 2.0, 0.0),
		color: glm::vec3(0.823, 0.117, 0.568),
		intensity: 1.0,
	});

	lights
}
