use crate::gpu_model::{GpuMaterial, GpuPrimitive};
use crate::scene::material::{Material, Texture};
use gl_helpers::*;
use nalgebra_glm as glm;
use std::fs;

////////////////////
// SHADER HELPERS //
static VERTEX_EXPECT: &str = "Couldn't read the vertex shader :(";
static GEOMETRY_EXPECT: &str = "Couldn't read the geometry shader :(";
static FRAGMENT_EXPECT: &str = "Couldn't read the fragment shader :(";

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
	let gs_src = fs::read_to_string("src/shaders/voxel_view.geom").expect(GEOMETRY_EXPECT);
	let fs_src = fs::read_to_string("src/shaders/voxel_view.frag").expect(FRAGMENT_EXPECT);

	GLProgram::new_gs(&vs_src[..], &gs_src[..], &fs_src[..])
}

pub fn load_voxelize_program() -> GLProgram {
	let vs_src = fs::read_to_string("src/shaders/voxelize.vert").expect(VERTEX_EXPECT);
	let gs_src = fs::read_to_string("src/shaders/voxelize.geom").expect(GEOMETRY_EXPECT);
	let fs_src = fs::read_to_string("src/shaders/voxelize.frag").expect(FRAGMENT_EXPECT);

	GLProgram::new_gs(&vs_src[..], &gs_src[..], &fs_src[..])
}

pub fn load_bounds_program() -> GLProgram {
	let vs_src = fs::read_to_string("src/shaders/volume_bounds.vert").expect(VERTEX_EXPECT);
	let fs_src = fs::read_to_string("src/shaders/volume_bounds.frag").expect(FRAGMENT_EXPECT);

	GLProgram::new(&vs_src[..], &fs_src[..])
}

pub fn load_clear_program() -> GLProgram {
	let vs_src = fs::read_to_string("src/shaders/clear_volume.vert").expect(VERTEX_EXPECT);
	let fs_src = fs::read_to_string("src/shaders/empty.frag").expect(FRAGMENT_EXPECT);

	GLProgram::new(&vs_src[..], &fs_src[..])
}

//////////////////////
// MATERIAL HELPERS //
pub fn load_texture(texture: &Texture) -> GLTexture {
	use image::GenericImage;

	let (width, height) = texture.image().dimensions();
	let raw_pixels = &texture.image().raw_pixels()[..];

	GLTexture::new_2d(
		width as usize,
		height as usize,
		InternalFormat::RGB32F,
		DataFormat::RGB,
		DataKind::UnsignedByte,
		FilterMode::Linear,
		Wrap::Repeat,
		true,
		raw_pixels,
	)
}

pub fn load_depth_texture() -> GLTexture {
	GLTexture::new_null_2d(
		2048,
		2048,
		InternalFormat::DepthComponent24,
		DataFormat::DepthComponent,
		DataKind::Float,
		FilterMode::Linear,
		Wrap::Clamp,
		false,
	)
}

///////////////////
// LIGHT HELPERS //
pub struct Light {
	pub direction: glm::Vec3,
	pub position: glm::Vec3,
	pub color: glm::Vec3,
	pub intensity: f32,
}

pub fn lights_to_soa(lights: &Vec<Light>) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
	let directions: Vec<f32> = lights
		.iter()
		.map(|l| l.direction.into_iter())
		.flatten()
		.cloned()
		.collect();

	let positions: Vec<f32> = lights
		.iter()
		.map(|l| l.position.into_iter())
		.flatten()
		.cloned()
		.collect();

	let colors_unflattened: Vec<glm::Vec3> = lights.iter().map(|l| (l.color * l.intensity)).collect();

	let colors: Vec<f32> = colors_unflattened
		.iter()
		.map(|c| c)
		.flatten()
		.cloned()
		.collect();

	(directions, positions, colors)
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

pub fn light_matrix(light: &Light) -> [f32; 16] {
	let light_view = glm::look_at_rh(
		&light.position,
		&(light.position + light.direction),
		&glm::vec3(0.0, 1.0, 0.0),
	);
	let light_proj = glm::ortho_rh_zo(-20.0, 10.0, -20.0, 20.0, 1.0, 20.0);

	let light_matrix: [f32; 16] = {
		let transmute_me: [[f32; 4]; 4] = (light_proj * light_view).into();
		unsafe { std::mem::transmute(transmute_me) }
	};

	light_matrix
}
