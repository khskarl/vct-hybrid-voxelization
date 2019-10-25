use crate::gpu_model::{GpuMaterial, GpuPrimitive};
use crate::scene::material::{Material, Texture};
use gl_helpers::*;
use glsl_include::Context;
use nalgebra_glm as glm;
use std::fs;

////////////////////
// SHADER HELPERS //
static VERTEX_EXPECT: &str = "Couldn't read the vertex shader :(";
static GEOMETRY_EXPECT: &str = "Couldn't read the geometry shader :(";
static FRAGMENT_EXPECT: &str = "Couldn't read the fragment shader :(";
static COMPUTE_EXPECT: &str = "Couldn't read the compute shader :(";
static SHARED_EXPECT: &str = "Couldn't read the shared glsl :(";
static EXPAND_EXPECT: &str = "Something went wrong in the expansion :(";

pub fn load_shared_glsl_context<'a>() -> (Context<'a>) {
	let shared_src = fs::read_to_string("src/shaders/shared.glsl").expect(SHARED_EXPECT);

	let mut context = Context::new();
	context.include("shared.glsl", &shared_src[..]);

	context
}

pub fn load_pbr_program() -> GLProgram {
	let vs_src = fs::read_to_string("src/shaders/pbr.vert").expect(VERTEX_EXPECT);
	let fs_src = fs::read_to_string("src/shaders/pbr.frag").expect(FRAGMENT_EXPECT);

	let context = load_shared_glsl_context();
	let vs_src = context.expand(vs_src).expect(EXPAND_EXPECT);
	let fs_src = context.expand(fs_src).expect(EXPAND_EXPECT);

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

	let context = load_shared_glsl_context();
	let vs_src = context.expand(vs_src).expect(EXPAND_EXPECT);
	let gs_src = context.expand(gs_src).expect(EXPAND_EXPECT);
	let fs_src = context.expand(fs_src).expect(EXPAND_EXPECT);

	GLProgram::new_gs(&vs_src[..], &gs_src[..], &fs_src[..])
}

pub fn load_classify_program() -> GLProgram {
	let vs_src = fs::read_to_string("src/shaders/classify.vert").expect(VERTEX_EXPECT);
	let gs_src = fs::read_to_string("src/shaders/classify.geom").expect(GEOMETRY_EXPECT);
	let fs_src = fs::read_to_string("src/shaders/classify.frag").expect(FRAGMENT_EXPECT);

	let context = load_shared_glsl_context();
	let vs_src = context.expand(vs_src).expect(EXPAND_EXPECT);
	let gs_src = context.expand(gs_src).expect(EXPAND_EXPECT);
	let fs_src = context.expand(fs_src).expect(EXPAND_EXPECT);

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

pub fn load_radiance_injection_program() -> GLProgram {
	let cs_src = fs::read_to_string("src/shaders/radiance_injection.comp").expect(COMPUTE_EXPECT);

	GLProgram::new_comp(&cs_src[..])
}

pub fn load_mipmap_program() -> GLProgram {
	let cs_src = fs::read_to_string("src/shaders/mipmap.comp").expect(COMPUTE_EXPECT);

	GLProgram::new_comp(&cs_src[..])
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
		1024,
		1024,
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
	pub position: glm::Vec3,
	pub color: glm::Vec3,
	pub intensity: f32,
}

pub fn lights_to_soa(lights: &Vec<Light>) -> (Vec<f32>, Vec<f32>) {
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

	(positions, colors)
}

pub fn load_lights() -> Vec<Light> {
	let mut lights = Vec::new();
	lights.push(Light {
		position: glm::vec3(-2.5, 7.0, -2.5),
		color: glm::vec3(0.815, 0.0, 0.333),
		intensity: 1.0,
	});
	lights.push(Light {
		position: glm::vec3(2.5, 7.0, -2.5),
		color: glm::vec3(0.0, 0.815, 0.333),
		intensity: 1.0,
	});
	lights.push(Light {
		position: glm::vec3(0.0, 7.0, 2.5),
		color: glm::vec3(0.0, 0.666, 1.0),
		intensity: 1.0,
	});

	lights
}

pub fn light_matrix(light: &Light) -> [f32; 16] {
	let light_view = glm::look_at_rh(
		&light.position,
		&(light.position + glm::vec3(0.0, -1.0, 1.0)),
		&glm::vec3(0.0, 1.0, 0.0),
	);
	let light_proj = glm::ortho_rh_zo(-20.0, 10.0, -20.0, 20.0, 1.0, 20.0);

	let light_matrix: [f32; 16] = {
		let transmute_me: [[f32; 4]; 4] = (light_proj * light_view).into();
		unsafe { std::mem::transmute(transmute_me) }
	};

	light_matrix
}

use crate::textures::Volume;

pub fn voxelization_pv(volume: &Volume) -> [f32; 16] {
	let half_width = volume.scaling().x as f32 / 2.0;
	let half_height = volume.scaling().y as f32 / 2.0;
	let depth = volume.scaling().z;
	let proj = glm::ortho_rh(
		-half_width,
		half_width + 0.1,
		-half_height,
		half_height + 0.1,
		depth + 0.01,
		0.0,
	);
	let position = volume.translation() + glm::vec3(0.0, 0.0, volume.scaling()[2] * 0.5);
	let view = glm::look_at_rh(
		&position,
		&(position + glm::vec3(0.0, 0.0, -1.0)),
		&[0.0, 1.0, 0.0].into(),
	);
	let pv: [f32; 16] = {
		let proj_view = proj * view;
		let transmute_me: [[f32; 4]; 4] = proj_view.into();
		unsafe { std::mem::transmute(transmute_me) }
	};

	pv
}
