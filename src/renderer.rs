use crate::gl_utils;
use crate::gpu_model::{GpuMaterial, GpuPrimitive};
use crate::scene::camera::*;
use crate::scene::material::{Material, Texture};
use crate::scene::model::Mesh;
use gl;
use gl_helpers::*;

use nalgebra_glm as glm;

use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

struct Light {
	direction: glm::Vec3,
	position: glm::Vec3,
	color: glm::Vec3,
	intensity: f32,
}

pub struct Renderer {
	primitives: Vec<GpuPrimitive>,
	materials: HashMap<String, Rc<GpuMaterial>>,
	textures: HashMap<String, Rc<GLTexture>>,
	pbr_program: GLProgram,
	lights: Vec<Light>,
	voxelized_scene: GLTexture,
}

impl Renderer {
	pub fn new(
		window_gl: &glutin::WindowedContext<glutin::PossiblyCurrent>,
		logical_size: glutin::dpi::LogicalSize,
	) -> Renderer {
		gl::load_with(|symbol| window_gl.get_proc_address(symbol) as *const _);

		gl_utils::print_opengl_diagnostics();
		gl_set_defaults();
		unsafe {
			gl::FrontFace(gl::CW);
		}
		gl_set_viewport(
			0,
			0,
			logical_size.width as usize,
			logical_size.height as usize,
		);

		let vs_src =
			fs::read_to_string("src/shaders/pbr.vs").expect("Couldn't read the vertex shader :(");
		let fs_src =
			fs::read_to_string("src/shaders/pbr.fs").expect("Couldn't read the fragment shader :(");
		let program = GLProgram::new(&vs_src[..], &fs_src[..]);

		let pixels: Vec<u8> = (0..1000)
			.map(|i| {
				if 400 < i && i < 500 && i % 2 == 0 {
					255
				} else {
					0
				}
			})
			.collect();

		let voxelized_scene = GLTexture::new_3d(
			10,
			10,
			10,
			InternalFormat::R8,
			DataFormat::Red,
			DataKind::UnsignedByte,
			FilterMode::None,
			Wrap::Clamp,
			true,
			&pixels[..],
		);

		let mut lights = Vec::new();
		lights.push(Light {
			direction: glm::vec3(0.05, -0.7, -0.3),
			position: glm::vec3(0.0, 0.0, 0.0),
			color: glm::vec3(1.0, 1.0, 1.0),
			intensity: 2.0,
		});
		lights.push(Light {
			direction: glm::vec3(0.0, 0.0, 0.0),
			position: glm::vec3(9.0, 2.0, 0.0),
			color: glm::vec3(0.7, 0.1, 0.2),
			intensity: 1.0,
		});
		lights.push(Light {
			direction: glm::vec3(0.0, 0.0, 0.0),
			position: glm::vec3(-9.0, 2.0, 0.0),
			color: glm::vec3(0.15, 0.25, 0.6),
			intensity: 1.0,
		});

		Renderer {
			primitives: Vec::new(),
			materials: HashMap::new(),
			textures: HashMap::new(),
			pbr_program: program,
			lights,
			voxelized_scene,
		}
	}

	pub fn render(&self, camera: &Camera) {
		gl_set_clear_color(&[0.1, 0.1, 0.1, 1.0]);
		gl_clear(true, true, true);

		self.pbr_program.bind();

		let proj: [f32; 16] = {
			let transmute_me: [[f32; 4]; 4] = camera.projection().into();
			unsafe { std::mem::transmute(transmute_me) }
		};

		let view: [f32; 16] = {
			let transmute_me: [[f32; 4]; 4] = camera.view().into();
			unsafe { std::mem::transmute(transmute_me) }
		};

		self.pbr_program.get_uniform("proj").set_mat4f(&proj);
		self.pbr_program.get_uniform("view").set_mat4f(&view);
		self.pbr_program.get_uniform("time").set_1f(0.1_f32);

		let directions: Vec<f32> = self
			.lights
			.iter()
			.map(|l| l.direction.into_iter())
			.flatten()
			.cloned()
			.collect();

		let positions: Vec<f32> = self
			.lights
			.iter()
			.map(|l| l.position.into_iter())
			.flatten()
			.cloned()
			.collect();

		let colors_unflattened: Vec<glm::Vec3> = self
			.lights
			.iter()
			.map(|l| (l.color * l.intensity))
			.collect();

		let colors: Vec<f32> = colors_unflattened
			.iter()
			.map(|c| c)
			.flatten()
			.cloned()
			.collect();

		self
			.pbr_program
			.get_uniform("light_direction")
			.set_3fv(&directions[..]);

		self
			.pbr_program
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

			let material = &primitive.material();
			self
				.pbr_program
				.get_uniform("albedo_map")
				.set_sampler_2d(&material.albedo(), 0);
			self
				.pbr_program
				.get_uniform("metaghness_map")
				.set_sampler_2d(&material.metaghness(), 1);
			self
				.pbr_program
				.get_uniform("normal_map")
				.set_sampler_2d(&material.normal(), 2);
			self
				.pbr_program
				.get_uniform("occlusion_map")
				.set_sampler_2d(&material.occlusion(), 3);

			gl_draw_elements(
				DrawMode::Triangles,
				primitive.count_vertices(),
				IndexKind::UnsignedInt,
				0,
			);
		}
	}

	pub fn submit_mesh(&mut self, mesh: &Mesh) {
		for primitive in mesh.primitives() {
			let material = self.fetch_material(&primitive.material);
			let gpu_primitive = GpuPrimitive::new(&primitive, &self.pbr_program, material);
			self.primitives.push(gpu_primitive);
		}
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

			let texture_rc = Rc::new(self.load_texture(texture));
			self.textures.insert(key.to_owned(), Rc::clone(&texture_rc));

			return Rc::clone(&texture_rc);
		}
	}
	fn load_texture(&mut self, texture: &Texture) -> GLTexture {
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
}
