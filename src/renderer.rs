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
use std::rc::Rc;

use crate::textures::Texture3D;

pub struct Renderer {
	viewport_size: (usize, usize),
	primitives: Vec<GpuPrimitive>,
	materials: HashMap<String, Rc<GpuMaterial>>,
	textures: HashMap<String, Rc<GLTexture>>,
	pbr_program: GLProgram,
	lights: Vec<Light>,
	depth_map: GLTexture,
	depth_map_framebuffer: GLFramebuffer,
	depth_program: GLProgram,
	volume_view_program: GLProgram,
	volume_view_primitive: GpuPrimitive,
	volume_scene: Texture3D,
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
			gl::FrontFace(gl::CW);
		}

		let depth_map = load_depth_texture();
		let depth_map_framebuffer = GLFramebuffer::new(&depth_map, &[Attachment::Depth], 0);

		// Volume setup
		let volume_scene = Texture3D::new([16, 16, 16].into());
		let volume_view_program = load_voxel_view_program();
		let volume_view_primitive =
			GpuPrimitive::from_volume((16.0, 16.0, 16.0), (16, 16, 16), &volume_view_program);

		Renderer {
			viewport_size: (logical_size.width as usize, logical_size.height as usize),
			primitives: Vec::new(),
			materials: HashMap::new(),
			textures: HashMap::new(),
			pbr_program: load_pbr_program(),
			lights: load_lights(),
			depth_map,
			depth_map_framebuffer,
			depth_program: load_depth_program(),
			volume_view_program,
			volume_view_primitive,
			volume_scene,
		}
	}

	fn render_to_shadow_map(&self) {
		gl_set_cull_face(CullFace::Front);
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

			gl_draw_elements(
				DrawMode::Triangles,
				primitive.count_vertices(),
				IndexKind::UnsignedInt,
				0,
			);
		}

		self.depth_map_framebuffer.unbind();
	}

	pub fn render(&self, camera: &Camera) {
		self.render_to_shadow_map();

		gl_set_cull_face(CullFace::Back);
		gl_set_viewport(0, 0, self.viewport_size.0, self.viewport_size.1);
		gl_set_clear_color(&[0.1, 0.1, 0.1, 1.0]);
		gl_clear(true, true, true);

		self.render_voxels(camera);
		self.render_scene(camera);
	}

	pub fn render_voxels(&self, camera: &Camera) {
		let proj_view: [f32; 16] = camera.proj_view_raw();

		self.volume_scene.bind();
		self.volume_view_program.bind();
		let loc = self.volume_view_program.get_uniform("volume").location();
		self.volume_scene.set_sampler(0, loc as u32);

		self
			.volume_view_program
			.get_uniform("mvp")
			.set_mat4f(&proj_view);

		self.volume_view_primitive.bind();

		gl_draw_arrays(
			DrawMode::Points,
			0,
			self.volume_view_primitive.count_vertices(),
		);
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
			let gpu_primitive = GpuPrimitive::new(&primitive, &self.pbr_program, material);
			self.primitives.push(gpu_primitive);
		}
	}

	pub fn light(&mut self, index: usize) -> &mut Light {
		&mut self.lights[index]
	}

	fn fetch_material(&mut self, material: &Material) -> Rc<GpuMaterial> {
		let key = material.name();

		if let Some(material_rc) = self.materials.get(key) {
			println!("Fetching GPU material '{}'...", key);

			return Rc::clone(material_rc);
		} else {
			println!("Loading GPU material '{}'...", key);

			let material_rc = Rc::new(self.load_material(material));
			self
				.materials
				.insert(key.to_owned(), Rc::clone(&material_rc));

			return Rc::clone(&material_rc);
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

			return Rc::clone(texture_rc);
		} else {
			println!("Loading GPU texture '{}'...", key);

			let texture_rc = Rc::new(load_texture(texture));
			self.textures.insert(key.to_owned(), Rc::clone(&texture_rc));

			return Rc::clone(&texture_rc);
		}
	}
}
