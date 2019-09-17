use std::str;
use std::time::Instant;

mod gl_utils;
mod renderer;

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

	let _target_dt = 0.01666666666;
	event_loop.run(move |event, _, control_flow| {
		use glutin::event::*;
		use glutin::event_loop::*;

		let start_frame_time = Instant::now();

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

					match key {
						Escape => *control_flow = ControlFlow::Exit,
						_ => (),
					}
				}
				_ => (),
			},
			Event::DeviceEvent { event, .. } => match event {
				_ => (),
			},
			_ => (),
		}

		renderer.render();
		window_gl.swap_buffers().unwrap();

		let dt = Instant::now()
			.duration_since(start_frame_time)
			.as_secs_f32();

		window_gl
			.window()
			.set_title(&format!("{} | {:.6}", WINDOW_TITLE, dt));
	});
}
