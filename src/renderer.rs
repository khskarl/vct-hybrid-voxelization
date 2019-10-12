use crate::gl_utils::*;
use crate::gpu_model::{GpuMaterial, GpuPrimitive};
use crate::renderer_utils::*;
use crate::scene::camera::*;
use crate::scene::material::{Material, Texture};
use crate::scene::model::Mesh;
use gl;
use gl_helpers::*;

use glm::UVec2;
use nalgebra_glm as glm;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::CString;
use std::rc::Rc;

use crate::textures::Volume;

#[derive(Copy, Clone, PartialEq)]
pub enum RenderingMode {
	Scene,
	Albedo,
	Normal,
	Emission,
	Radiance,
}

pub struct Renderer {
	viewport_size: (usize, usize),
	rendering_mode: RenderingMode,
	primitives: Vec<GpuPrimitive>,
	materials: HashMap<String, Rc<GpuMaterial>>,
	textures: HashMap<String, Rc<GLTexture>>,
	pbr_program: GLProgram,
	lights: Vec<Light>,
	depth_map: GLTexture,
	depth_map_framebuffer: GLFramebuffer,
	depth_program: GLProgram,
	volume_view_program: GLProgram,
	volume_scene: Volume,
	voxelize_program: GLProgram,
	bounds_program: GLProgram,
	clear_program: GLProgram,
	inject_program: GLProgram,
}

impl Renderer {
	pub fn new(
		window_gl: &glutin::WindowedContext<glutin::PossiblyCurrent>,
		logical_size: glutin::dpi::LogicalSize,
	) -> Renderer {
		gl::load_with(|symbol| window_gl.get_proc_address(symbol) as *const _);
		gl_set_defaults();
		print_opengl_diagnostics();

		unsafe {
			gl::Enable(gl::PROGRAM_POINT_SIZE);
			gl::Enable(gl::TEXTURE_3D);
			gl::FrontFace(gl::CCW);
		}

		let depth_map = load_depth_texture();
		let depth_map_framebuffer = GLFramebuffer::new(&depth_map, &[Attachment::Depth], 0);

		// Volume setup
		let volume_view_program = load_voxel_view_program();
		let volume_scene = Volume::new(64, &volume_view_program);

		Renderer {
			viewport_size: (logical_size.width as usize, logical_size.height as usize),
			rendering_mode: RenderingMode::Radiance,
			primitives: Vec::new(),
			materials: HashMap::new(),
			textures: HashMap::new(),
			pbr_program: load_pbr_program(),
			lights: load_lights(),
			depth_map,
			depth_map_framebuffer,
			depth_program: load_depth_program(),
			volume_view_program,
			volume_scene,
			voxelize_program: load_voxelize_program(),
			bounds_program: load_bounds_program(),
			clear_program: load_clear_program(),
			inject_program: load_radiance_injection_program(),
		}
	}

	fn render_to_shadow_map(&self) {
		gl_set_cull_face(CullFace::Front);
		gl_set_depth_write(true);
		gl_set_viewport(0, 0, self.depth_map.width(), self.depth_map.height());
		self.depth_map_framebuffer.bind();
		gl_clear(false, true, false);

		self.depth_program.bind();
		self
			.depth_program
			.get_uniform("light_matrix")
			.set_mat4f(&light_matrix(&self.lights[0]));

		for primitive in &self.primitives {
			primitive.bind();

			self
				.depth_program
				.get_uniform("model")
				.set_mat4f(&primitive.model_matrix_raw());

			gl_draw_elements(
				DrawMode::Triangles,
				primitive.count_vertices(),
				IndexKind::UnsignedInt,
				0,
			);
		}

		self.depth_map_framebuffer.unbind();
	}

	fn clear_volume(&self) {
		self.clear_program.bind();

		self.volume_scene.bind_texture_albedo(0);
		self.volume_scene.bind_texture_normal(1);
		self.volume_scene.bind_texture_emission(2);
		self.volume_scene.bind_texture_radiance(3);

		self
			.clear_program
			.get_uniform("u_width")
			.set_1i(self.volume_scene.resolution() as i32);
		self
			.clear_program
			.get_uniform("u_height")
			.set_1i(self.volume_scene.resolution() as i32);
		self
			.clear_program
			.get_uniform("u_depth")
			.set_1i(self.volume_scene.resolution() as i32);
		self.volume_scene.draw();
	}

	fn inject_light(&self) {
		self.inject_program.bind();

		// self
		// 	.inject_program
		// 	.get_uniform("u_light_matrix")
		// 	.set_mat4f(&light_matrix(&self.lights[0]));

		self.volume_scene.bind_texture_albedo(0);
		self.volume_scene.bind_texture_normal(1);
		self.volume_scene.bind_texture_emission(2);
		self.volume_scene.bind_image_radiance(3);
		// self
		// 	.inject_program
		// 	.get_uniform("u_shadow_map")
		// 	.set_sampler_2d(&self.depth_map, 4);

		let resolution = self.volume_scene.resolution();
		self
			.inject_program
			.get_uniform("u_width")
			.set_1i(resolution as i32);
		self
			.inject_program
			.get_uniform("u_height")
			.set_1i(resolution as i32);
		self
			.inject_program
			.get_uniform("u_depth")
			.set_1i(resolution as i32);
		unsafe {
			gl::DispatchCompute(resolution as u32, resolution as u32, resolution as u32);
		}
	}

	fn render_bounds(&self, camera: &Camera) {
		self.bounds_program.bind();

		let translation = glm::translation(self.volume_scene.translation());
		let scaling = glm::scaling(self.volume_scene.scaling());
		let mvp = camera.proj_view() * (translation * scaling);

		self
			.volume_view_program
			.get_uniform("mvp")
			.set_mat4f(<&[f32; 16]>::try_from(mvp.as_slice()).unwrap());

		gl_draw_arrays(DrawMode::Lines, 0, 24);
	}

	fn voxelize(&self) {
		self.clear_volume();

		let width = self.volume_scene.resolution() as i32;
		let height = self.volume_scene.resolution() as i32;
		let depth = self.volume_scene.resolution() as i32;

		gl_set_depth_write(false);
		gl_set_cull_face(CullFace::None);
		gl_set_viewport(0, 0, width as usize, height as usize);
		gl_clear(true, true, false);
		unsafe {
			gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
			gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
		};

		self.voxelize_program.bind();
		self.voxelize_program.get_uniform("u_width").set_1i(width);
		// self.voxelize_program.get_uniform("u_height").set_1i(height);
		self.voxelize_program.get_uniform("u_depth").set_1i(depth);

		let half_width = self.volume_scene.scaling().x as f32 / 2.0;
		let half_height = self.volume_scene.scaling().y as f32 / 2.0;
		let half_depth = self.volume_scene.scaling().z;
		let proj = glm::ortho_rh(
			-half_width,
			half_width + 0.1,
			-half_height,
			half_height + 0.1,
			half_depth as f32,
			0.0,
		);
		// let translation = self.volume_scene.translation() + self.volume_scene.scaling() * 0.5;
		// let view = glm::look_at_rh(
		// 	&translation,
		// 	&(translation + glm::vec3(0.0, 0.0, 1.0)),
		// 	&[0.0, 1.0, 0.0].into(),
		// );
		let view = glm::look_at_rh(
			&glm::vec3(0.0, 5.0, 5.0),
			&(glm::vec3(0.0, 5.0, 5.0) + glm::vec3(0.0, 0.0, -1.0)),
			&[0.0, 1.0, 0.0].into(),
		);
		let pv: [f32; 16] = {
			let proj_view = proj * view;
			let transmute_me: [[f32; 4]; 4] = proj_view.into();
			unsafe { std::mem::transmute(transmute_me) }
		};
		self.voxelize_program.get_uniform("pv").set_mat4f(&pv);

		self.volume_scene.bind_image_albedo(0);
		self.volume_scene.bind_image_normal(1);
		self.volume_scene.bind_image_emission(2);

		for primitive in &self.primitives {
			primitive.bind();

			self
				.voxelize_program
				.get_uniform("model")
				.set_mat4f(&primitive.model_matrix_raw());

			let mat = &primitive.material();
			self
				.pbr_program
				.get_uniform("albedo_map")
				.set_sampler_2d(&mat.albedo(), 0);

			gl_draw_elements(
				DrawMode::Triangles,
				primitive.count_vertices(),
				IndexKind::UnsignedInt,
				0,
			);
		}

		unsafe {
			gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
			gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
			// gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
		}
	}

	pub fn render(&self, camera: &Camera) {
		self.render_to_shadow_map();

		self.voxelize();
		self.inject_light();

		gl_set_viewport(0, 0, self.viewport_size.0, self.viewport_size.1);
		gl_set_clear_color(&[0.8, 0.75, 0.79, 1.0]);
		gl_set_depth_write(true);
		gl_clear(true, true, true);

		if self.rendering_mode != RenderingMode::Scene {
			self.render_voxels(camera);
		}

		gl_set_cull_face(CullFace::Back);
		self.render_scene(camera);
		self.render_bounds(camera);
	}

	pub fn render_voxels(&self, camera: &Camera) {
		gl_set_cull_face(CullFace::None);

		self.volume_view_program.bind();

		match self.rendering_mode {
			RenderingMode::Scene => self.volume_scene.bind_texture_albedo(0),
			RenderingMode::Albedo => self.volume_scene.bind_texture_albedo(0),
			RenderingMode::Normal => self.volume_scene.bind_texture_normal(0),
			RenderingMode::Emission => self.volume_scene.bind_texture_emission(0),
			RenderingMode::Radiance => self.volume_scene.bind_texture_radiance(0),
		};

		let translation = glm::translation(self.volume_scene.translation());
		let scaling = glm::scaling(self.volume_scene.scaling());
		let mvp = camera.proj_view() * (translation * scaling);

		self
			.volume_view_program
			.get_uniform("mvp")
			.set_mat4f(<&[f32; 16]>::try_from(mvp.as_slice()).unwrap());

		self
			.volume_view_program
			.get_uniform("resolution")
			.set_1i(self.volume_scene.resolution() as i32);

		self.volume_scene.draw();
	}

	pub fn render_scene(&self, camera: &Camera) {
		let proj_view: [f32; 16] = camera.proj_view_raw();

		let program = &self.pbr_program;
		program.bind();
		program.get_uniform("pv").set_mat4f(&proj_view);

		program
			.get_uniform("light_matrix")
			.set_mat4f(&light_matrix(&self.lights[0]));
		program
			.get_uniform("shadow_map")
			.set_sampler_2d(&self.depth_map, 4);

		//
		let (directions, positions, colors) = lights_to_soa(&self.lights);
		program
			.get_uniform("light_direction")
			.set_3fv(&directions[..]);
		program
			.get_uniform("light_position")
			.set_3fv(&positions[..]);
		self
			.pbr_program
			.get_uniform("light_color")
			.set_3fv(&colors[..]);
		self
			.pbr_program
			.get_uniform("num_lights")
			.set_1i(self.lights.len() as i32);

		self
			.pbr_program
			.get_uniform("camera_position")
			.set_3f(0, &camera.position.into());

		for primitive in &self.primitives {
			primitive.bind();

			self
				.pbr_program
				.get_uniform("model")
				.set_mat4f(&primitive.model_matrix_raw());

			let mat = &primitive.material();
			self
				.pbr_program
				.get_uniform("albedo_map")
				.set_sampler_2d(&mat.albedo(), 0);
			self
				.pbr_program
				.get_uniform("metaghness_map")
				.set_sampler_2d(&mat.metaghness(), 1);
			self
				.pbr_program
				.get_uniform("normal_map")
				.set_sampler_2d(&mat.normal(), 2);
			self
				.pbr_program
				.get_uniform("occlusion_map")
				.set_sampler_2d(&mat.occlusion(), 3);

			gl_draw_elements(
				DrawMode::Triangles,
				primitive.count_vertices(),
				IndexKind::UnsignedInt,
				0,
			);
		}
	}

	pub fn set_viewport_size(&mut self, size: (usize, usize)) {
		self.viewport_size = size;
	}

	pub fn submit_mesh(&mut self, mesh: &Mesh) {
		for primitive in mesh.primitives() {
			let material = self.fetch_material(&primitive.material);
			let gpu_primitive = GpuPrimitive::new(
				&primitive,
				&self.pbr_program,
				material,
				mesh.position,
				mesh.scale,
			);
			self.primitives.push(gpu_primitive);
		}
	}

	pub fn light(&mut self, index: usize) -> &mut Light {
		&mut self.lights[index]
	}

	pub fn volume_mut(&mut self) -> &mut Volume {
		&mut self.volume_scene
	}

	pub fn primitives_mut(&mut self) -> &mut Vec<GpuPrimitive> {
		&mut self.primitives
	}

	pub fn rendering_mode_mut(&mut self) -> &mut RenderingMode {
		&mut self.rendering_mode
	}

	fn fetch_material(&mut self, material: &Material) -> Rc<GpuMaterial> {
		let key = material.name();

		if let Some(material_rc) = self.materials.get(key) {
			println!("Fetching GPU material '{}'...", key);

			Rc::clone(material_rc)
		} else {
			println!("Loading GPU material '{}'...", key);

			let material_rc = Rc::new(self.load_material(material));
			self
				.materials
				.insert(key.to_owned(), Rc::clone(&material_rc));

			Rc::clone(&material_rc)
		}
	}

	fn load_material(&mut self, material: &Material) -> GpuMaterial {
		let albedo = self.fetch_texture(material.albedo());
		let metaghness = self.fetch_texture(material.metaghness());
		let normal = self.fetch_texture(material.normal());
		let occlusion = self.fetch_texture(material.occlusion());

		GpuMaterial::new(albedo, metaghness, normal, occlusion)
	}

	fn fetch_texture(&mut self, texture: &Texture) -> Rc<GLTexture> {
		let key = texture.name();

		if let Some(texture_rc) = self.textures.get(key) {
			println!("Fetching GPU texture '{}'...", key);

			Rc::clone(texture_rc)
		} else {
			println!("Loading GPU texture '{}'...", key);

			let texture_rc = Rc::new(load_texture(texture));
			self.textures.insert(key.to_owned(), Rc::clone(&texture_rc));

			Rc::clone(&texture_rc)
		}
	}
}
