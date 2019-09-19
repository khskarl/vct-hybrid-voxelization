use std::str;
use std::time::Instant;

mod gl_utils;
mod renderer;

use nalgebra_glm as glm;

mod scene;
use scene::camera::*;
use scene::model::Model;

fn main() {
	const WINDOW_TITLE: &str = "Lunar Renderer";

	let event_loop = glutin::event_loop::EventLoop::new();

	let window_builder = glutin::window::WindowBuilder::new()
		.with_title(WINDOW_TITLE)
		.with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));

	let window_gl = {
		let window_gl = glutin::ContextBuilder::new()
			.build_windowed(window_builder, &event_loop)
			.unwrap();
		unsafe { window_gl.make_current() }.unwrap()
	};

	let renderer = renderer::Renderer::new(&window_gl);

	let mut camera = Camera::new(glm::vec3(0.0, 0.0, -3.0), 0.0, 0.0);

	let model = Model::new("assets/models/box.gltf");
	// renderer.submit_model(&model);

	let target_dt = 0.01666666666;
	let mut start_frame_time = Instant::now();

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait;

		use glutin::event::*;
		use glutin::event_loop::*;

		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state: ElementState::Pressed,
							virtual_keycode: Some(key),
							..
						},
					..
				} => {
					use glutin::event::VirtualKeyCode::*;

					let dt = target_dt;
					let move_rate = 5.0; // m/s
					let rotation_rate = 60.0; // Degrees/s

					match key {
						Escape => *control_flow = ControlFlow::Exit,
						A => camera.move_right(-move_rate * dt),
						D => camera.move_right(move_rate * dt),
						S => camera.move_forward(-move_rate * dt),
						W => camera.move_forward(move_rate * dt),

						J => camera.rotate_right(-rotation_rate * dt),
						L => camera.rotate_right(rotation_rate * dt),
						K => camera.rotate_up(-rotation_rate * dt),
						I => camera.rotate_up(rotation_rate * dt),

						// Z => aux.time -= 0.05 * dt,
						// X => aux.time += 0.05 * dt,
						_ => (),
					}
				}
				WindowEvent::RedrawRequested => {
					renderer.render(&camera);
					window_gl.swap_buffers().unwrap();
				}
				_ => (),
			},
			Event::EventsCleared => {
				let dt = Instant::now()
					.duration_since(start_frame_time)
					.as_secs_f32();

				window_gl
					.window()
					.set_title(&format!("{} | {:.6}", WINDOW_TITLE, dt));

				window_gl.window().request_redraw();

				start_frame_time = Instant::now();

				*control_flow = ControlFlow::Poll;
			}
			Event::DeviceEvent { event, .. } => match event {
				_ => (),
			},
			_ => *control_flow = ControlFlow::Poll,
		}
	});
}
