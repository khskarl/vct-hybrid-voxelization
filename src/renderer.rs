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
	dimensions: (usize, usize),
	primitives: Vec<GpuPrimitive>,
	materials: HashMap<String, Rc<GpuMaterial>>,
	textures: HashMap<String, Rc<GLTexture>>,
	pbr_program: GLProgram,
	lights: Vec<Light>,
	depth_map: GLTexture,
	depth_map_framebuffer: GLFramebuffer,
	depth_program: GLProgram,
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

		let depth_map = GLTexture::new_null_2d(
			2048,
			2048,
			InternalFormat::DepthComponent24,
			DataFormat::DepthComponent,
			DataKind::Float,
			FilterMode::Linear,
			Wrap::Clamp,
			false,
		);

		let depth_map_framebuffer = GLFramebuffer::new(&depth_map, &[Attachment::Depth], 0);

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
			direction: glm::vec3(0.05, -0.9, -0.2),
			position: glm::vec3(0.0, 2.0, 0.0),
			color: glm::vec3(1.0, 1.0, 1.0),
			intensity: 4.0,
		});
		lights.push(Light {
			direction: glm::vec3(0.0, 0.0, 0.0),
			position: glm::vec3(9.0, 2.0, 0.0),
			color: glm::vec3(0.5, 0.2, 0.3),
			intensity: 1.0,
		});
		lights.push(Light {
			direction: glm::vec3(0.0, 0.0, 0.0),
			position: glm::vec3(-9.0, 2.0, 0.0),
			color: glm::vec3(0.2, 0.3, 0.5),
			intensity: 1.0,
		});

		let pbr_program = {
			let vs_src =
				fs::read_to_string("src/shaders/pbr.vert").expect("Couldn't read the vertex shader :(");
			let fs_src =
				fs::read_to_string("src/shaders/pbr.frag").expect("Couldn't read the fragment shader :(");

			GLProgram::new(&vs_src[..], &fs_src[..])
		};

		let depth_program = {
			let vs_src = fs::read_to_string("src/shaders/depth_pass.vert")
				.expect("Couldn't read the vertex shader :(");
			let fs_src = fs::read_to_string("src/shaders/depth_pass.frag")
				.expect("Couldn't read the fragment shader :(");

			GLProgram::new(&vs_src[..], &fs_src[..])
		};

		Renderer {
			dimensions: (logical_size.width as usize, logical_size.height as usize),
			primitives: Vec::new(),
			materials: HashMap::new(),
			textures: HashMap::new(),
			pbr_program,
			lights,
			depth_map,
			depth_map_framebuffer,
			depth_program,
			voxelized_scene,
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
		gl_set_viewport(0, 0, self.dimensions.0, self.dimensions.1);
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

		self
			.pbr_program
			.get_uniform("light_matrix")
			.set_mat4f(&light_matrix(&self.lights[0]));
		self
			.pbr_program
			.get_uniform("shadow_map")
			.set_sampler_2d(&self.depth_map, 4);

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

fn light_matrix(light: &Light) -> [f32; 16] {
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
