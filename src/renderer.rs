use crate::gl_timer::*;
use crate::gl_utils::*;
use crate::gpu_model::{GpuMaterial, GpuPrimitive};
use crate::renderer_utils::*;
use crate::scene::camera::*;
use crate::scene::material::{Material, Texture};
use crate::scene::model::Mesh;
use gl;
use gl_helpers::*;

use nalgebra_glm as glm;

use std::collections::HashMap;
use std::convert::TryFrom;
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

#[derive(Copy, Clone, PartialEq)]
pub enum VoxelizationMode {
	FragmentOnly,
	Hybrid,
}

pub struct Renderer {
	viewport_size: (usize, usize),
	pub rendering_mode: RenderingMode,
	pub voxelization_mode: VoxelizationMode,
	primitives: Vec<GpuPrimitive>,
	materials: HashMap<String, Rc<GpuMaterial>>,
	textures: HashMap<String, Rc<GLTexture>>,
	pbr_program: GLProgram,
	pub lights: Vec<Light>,
	depth_map: GLTexture,
	depth_map_framebuffer: GLFramebuffer,
	depth_program: GLProgram,
	volume_view_program: GLProgram,
	volume_scene: Volume,
	voxelize_program: GLProgram,
	classify_program: GLProgram,
	bounds_program: GLProgram,
	clear_program: GLProgram,
	inject_program: GLProgram,
	tri_count_buffer: GLBuffer,
	timer: GlTimer,
}

impl Renderer {
	pub fn new(
		window_gl: &glutin::WindowedContext<glutin::PossiblyCurrent>,
		logical_size: glutin::dpi::LogicalSize,
		resolution: usize,
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
		depth_map_framebuffer.unbind();

		// Volume setup
		let volume_view_program = load_voxel_view_program();
		let volume_scene = Volume::new(resolution, &volume_view_program);

		Renderer {
			viewport_size: (logical_size.width as usize, logical_size.height as usize),
			rendering_mode: RenderingMode::Radiance,
			voxelization_mode: VoxelizationMode::FragmentOnly,
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
			classify_program: load_classify_program(),
			bounds_program: load_bounds_program(),
			clear_program: load_clear_program(),
			inject_program: load_radiance_injection_program(),
			tri_count_buffer: GLBuffer::new(BufferTarget::AtomicCounter, 0, Usage::DynamicDraw, &[999]),
			timer: GlTimer::new(10, 1200),
		}
	}

	#[allow(dead_code)]
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

		self.volume_scene.draw();
	}

	fn inject_light(&self) {
		self.inject_program.bind();

		let (positions, colors) = lights_to_soa(&self.lights);

		self
			.inject_program
			.get_uniform("u_light_position")
			.set_3fv(&positions[..]);
		self
			.inject_program
			.get_uniform("u_light_color")
			.set_3fv(&colors[..]);
		self
			.inject_program
			.get_uniform("u_num_lights")
			.set_1i(self.lights.len() as i32);

		self.volume_scene.bind_texture_albedo(0);
		self.volume_scene.bind_texture_normal(1);
		self.volume_scene.bind_texture_emission(2);
		self.volume_scene.bind_image_radiance(3);

		let resolution = self.volume_scene.resolution() as i32;
		self
			.inject_program
			.get_uniform("u_resolution")
			.set_3i(1, &[resolution, resolution, resolution]);

		let position = *self.volume_scene.translation();
		let scale = *self.volume_scene.scaling();

		self
			.inject_program
			.get_uniform("u_volume_center")
			.set_3f(1, &position.into());
		self
			.inject_program
			.get_uniform("u_volume_scale")
			.set_3f(1, &scale.into());

		unsafe {
			gl::DispatchCompute(
				resolution as u32 / 8,
				resolution as u32 / 8,
				resolution as u32 / 8,
			);

			gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
		}
	}

	fn render_bounds(&self, camera: &Camera) {
		self.bounds_program.bind();

		let translation =
			glm::translation(&(self.volume_scene.translation() - self.volume_scene.scaling() * 0.5));
		let scaling = glm::scaling(self.volume_scene.scaling());
		let mvp = camera.proj_view() * (translation * scaling);

		self
			.volume_view_program
			.get_uniform("mvp")
			.set_mat4f(<&[f32; 16]>::try_from(mvp.as_slice()).unwrap());

		gl_draw_arrays(DrawMode::Lines, 0, 24);

		let translation = glm::translation(
			&(self.volume_scene.view_translation() - self.volume_scene.view_scaling() * 0.5),
		);
		let scaling = glm::scaling(self.volume_scene.view_scaling());
		let mvp = camera.proj_view() * (translation * scaling);

		self
			.volume_view_program
			.get_uniform("mvp")
			.set_mat4f(<&[f32; 16]>::try_from(mvp.as_slice()).unwrap());

		gl_draw_arrays(DrawMode::Lines, 0, 24);
	}

	fn voxelize(&mut self) {
		match self.voxelization_mode {
			VoxelizationMode::FragmentOnly => self.voxelize_fragment(),
			VoxelizationMode::Hybrid => self.voxelize_hybrid(),
		}
	}

	fn voxelize_hybrid(&mut self) {
		self.clear_volume();

		self.timer.begin("voxelize_hybrid_triangle");

		let width = self.volume_scene.resolution() as i32;
		let height = self.volume_scene.resolution() as i32;
		let depth = self.volume_scene.resolution() as i32;

		gl_set_depth_write(false);
		gl_set_cull_face(CullFace::None);
		gl_set_viewport(0, 0, width as usize, height as usize);
		gl_clear(true, true, false);
		unsafe {
			gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
			gl::Enable(gl::RASTERIZER_DISCARD);
		};

		self.classify_program.bind();
		self
			.classify_program
			.get_uniform("u_resolution")
			.set_3i(1, &[width, height, depth]);

		let pv: [f32; 16] = voxelization_pv(&self.volume_scene);
		self.classify_program.get_uniform("pv").set_mat4f(&pv);

		self.volume_scene.bind_image_albedo(0);
		self.volume_scene.bind_image_normal(1);
		self.volume_scene.bind_image_emission(2);
		for primitive in &self.primitives {
			self.tri_count_buffer.bind_base(0);

			let tri_count = self.tri_count_buffer.read_data_u32();
			// println!("Pre-before: {}", tri_count);
			self.tri_count_buffer.write_data_u32(12);
			let tri_count = self.tri_count_buffer.read_data_u32();
			// println!("Before: {}", tri_count);

			primitive.bind();

			self
				.classify_program
				.get_uniform("model")
				.set_mat4f(&primitive.model_matrix_raw());

			let mat = &primitive.material();
			self
				.classify_program
				.get_uniform("albedo_map")
				.set_sampler_2d(&mat.albedo(), 0);

			gl_draw_elements(
				DrawMode::Triangles,
				primitive.count_vertices(),
				IndexKind::UnsignedInt,
				0,
			);

			unsafe {
				gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
			}
			let tri_count = self.tri_count_buffer.read_data_u32();
			// println!("After: {}", tri_count);
		}

		unsafe {
			gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
			gl::Disable(gl::RASTERIZER_DISCARD);
		}
		self.timer.end("voxelize_hybrid_triangle");

		self.tri_count_buffer.unbind();
	}

	fn voxelize_fragment(&self) {
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

		let pv: [f32; 16] = voxelization_pv(&self.volume_scene);

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
				.voxelize_program
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

	pub fn render(&mut self, camera: &Camera) {
		self.timer.begin_frame();
		// self.render_to_shadow_map();

		self.voxelize();
		self.inject_light();

		self.volume_scene.generate_mipmap();

		gl_set_viewport(0, 0, self.viewport_size.0, self.viewport_size.1);
		// gl_set_clear_color(&[0.8, 0.75, 0.79, 1.0]);
		gl_set_clear_color(&[0.02, 0.015, 0.01, 1.0]);

		gl_set_depth_write(true);
		gl_clear(true, true, true);

		if self.rendering_mode != RenderingMode::Scene {
			self.render_voxels(camera);
		}

		gl_set_cull_face(CullFace::Back);
		self.render_scene(camera);
		// self.render_bounds(camera);
		self.timer.end_frame();
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

		let translation = glm::translation(
			&(self.volume_scene.view_translation() - self.volume_scene.view_scaling() * 0.5),
		);
		let scaling = glm::scaling(self.volume_scene.view_scaling());
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

	pub fn render_scene(&mut self, camera: &Camera) {
		self.timer.begin("render_scene");

		let proj_view: [f32; 16] = camera.proj_view_raw();

		let program = &self.pbr_program;
		program.bind();
		program.get_uniform("pv").set_mat4f(&proj_view);

		program
			.get_uniform("u_width")
			.set_1i(self.volume_scene.resolution() as i32);

		self.volume_scene.bind_texture_radiance(4);

		let position = *self.volume_scene.translation();
		let scale = *self.volume_scene.scaling();

		program
			.get_uniform("u_volume_center")
			.set_3f(1, &position.into());
		program
			.get_uniform("u_volume_scale")
			.set_3f(1, &scale.into());

		let (positions, colors) = lights_to_soa(&self.lights);

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
			.set_3f(1, &camera.position.into());

		for primitive in &self.primitives {
			primitive.bind();

			self
				.pbr_program
				.get_uniform("model")
				.set_mat4f(&primitive.model_matrix_raw());

			let mat = &primitive.material();
			mat.albedo().bind_unit(0);
			mat.metaghness().bind_unit(1);
			mat.normal().bind_unit(2);
			mat.occlusion().bind_unit(3);

			gl_draw_elements(
				DrawMode::Triangles,
				primitive.count_vertices(),
				IndexKind::UnsignedInt,
				0,
			);
		}

		self.timer.end("render_scene");
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

	pub fn save_diagnostics(&self, scene_name: &str) {
		let resolution = self.volume_scene.resolution();
		let file_name = format!("{}_{}.csv", resolution, scene_name);

		self.timer.save_file(&file_name).unwrap();
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
