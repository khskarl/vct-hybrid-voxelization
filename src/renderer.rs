use gl_helpers::*;

use crate::gl_utils;
use crate::scene::camera::*;
use crate::scene::model::Model;
use crate::gpu_model::GpuModel;

#[derive(Debug)]
pub struct Renderer {
	models: Vec<GpuModel>,
	pbr_program: GLProgram,
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

		let program = GLProgram::new(gl_utils::VS_SRC, gl_utils::FS_SRC);

		program.get_uniform("time").set_1f(1.0_f32);

		Renderer {
			models: Vec::<GpuModel>::new(),
			pbr_program: program,
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

		for model in &self.models {
			model.bind();
			self
				.pbr_program
				.get_uniform("albedo")
				.set_sampler_2d(&model.color_texture(), 0);

			gl_draw_elements(
				DrawMode::Triangles,
				model.count_vertices(),
				IndexKind::UnsignedInt,
				0,
			);
		}
	}

	pub fn submit_model(&mut self, model: &Model) {
		let gpu_model = GpuModel::new(&model, &self.pbr_program);
		self.models.push(gpu_model);
	}
}
