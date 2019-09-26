use std::str;
use std::time::Instant;

mod gl_utils;
mod gpu_model;
mod renderer;

use nalgebra_glm as glm;

mod scene;
use scene::camera::*;
use scene::model::{Mesh, Resources};

use imgui_winit_support::{HiDpiMode, WinitPlatform};

fn main() {
	const WINDOW_TITLE: &str = "Lunar Renderer 🐶";
	let logical_size = glutin::dpi::LogicalSize::new(1280.0, 720.0);

	let event_loop = glutin::event_loop::EventLoop::new();

	let window_builder = glutin::window::WindowBuilder::new()
		.with_title(WINDOW_TITLE)
		.with_inner_size(logical_size);

	let window_gl = {
		let window_gl = glutin::ContextBuilder::new()
			.build_windowed(window_builder, &event_loop)
			.unwrap();
		unsafe { window_gl.make_current() }.unwrap()
	};

	let mut imgui = imgui::Context::create();
	let mut platform = WinitPlatform::init(&mut imgui);
	platform.attach_window(imgui.io_mut(), &window_gl.window(), HiDpiMode::Default);

	// Renderer setup
	let mut renderer = renderer::Renderer::new(&window_gl, logical_size);
	{
		let logical_size = window_gl.window().inner_size();
		let dpi_factor = window_gl.window().hidpi_factor();
		let physical_size = logical_size.to_physical(dpi_factor);

		imgui.io_mut().display_size = [physical_size.width as f32, physical_size.height as f32];
		window_gl.resize(logical_size.to_physical(dpi_factor));
	}
	let imgui_renderer =
		imgui_opengl_renderer::Renderer::new(&mut imgui, |s| window_gl.get_proc_address(s) as _);

	let mut camera = Camera::new(glm::vec3(-5.0, 2.0, 0.0), 0.0, 0.0);
	{
		let mut resources = Resources::new();
		renderer.submit_mesh(&Mesh::new("assets/models/sphere.glb", &mut resources));
		// renderer.submit_mesh(&Mesh::new("assets/models/debug_plane.glb", &mut resources));
		renderer.submit_mesh(&Mesh::new("assets/models/sponza.glb", &mut resources));
	}

	let mut key_states = KeyStates::new();

	let target_dt = 0.01666666666;
	let mut dt = target_dt;
	let mut start_frame_time = Instant::now();

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait;
		platform.handle_event(imgui.io_mut(), &window_gl.window(), &event);

		use glutin::event::*;
		use glutin::event_loop::*;

		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
				WindowEvent::Resized(logical_size) => {
					let dpi_factor = window_gl.window().hidpi_factor();
					window_gl.resize(logical_size.to_physical(dpi_factor));
				}
				WindowEvent::KeyboardInput {
					input: KeyboardInput {
						state,
						virtual_keycode: Some(key),
						..
					},
					..
				} => {
					use glutin::event::ElementState::*;
					use glutin::event::VirtualKeyCode::*;

					match (key, state) {
						(Escape, Released) => *control_flow = ControlFlow::Exit,
						(A, _) => key_states.A = state,
						(D, _) => key_states.D = state,
						(S, _) => key_states.S = state,
						(W, _) => key_states.W = state,

						(J, _) => key_states.J = state,
						(L, _) => key_states.L = state,
						(K, _) => key_states.K = state,
						(I, _) => key_states.I = state,

						_ => (),
					}
				}
				WindowEvent::RedrawRequested => {
					imgui.io_mut().update_delta_time(start_frame_time);

					let ui = imgui.frame();

					renderer.render(&camera);

					ui.show_demo_window(&mut true);
					imgui_renderer.render(ui);
					window_gl.swap_buffers().unwrap();
				}
				_ => (),
			},
			Event::EventsCleared => {
				update_camera(&mut camera, dt, &key_states);

				dt = Instant::now()
					.duration_since(start_frame_time)
					.as_secs_f32();
				imgui.io_mut().delta_time = dt;

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

#[derive(Debug)]
#[allow(non_snake_case)]
struct KeyStates {
	A: glutin::event::ElementState,
	D: glutin::event::ElementState,
	S: glutin::event::ElementState,
	W: glutin::event::ElementState,

	J: glutin::event::ElementState,
	L: glutin::event::ElementState,
	K: glutin::event::ElementState,
	I: glutin::event::ElementState,
}

impl KeyStates {
	pub fn new() -> KeyStates {
		KeyStates {
			A: glutin::event::ElementState::Released,
			D: glutin::event::ElementState::Released,
			S: glutin::event::ElementState::Released,
			W: glutin::event::ElementState::Released,

			J: glutin::event::ElementState::Released,
			L: glutin::event::ElementState::Released,
			K: glutin::event::ElementState::Released,
			I: glutin::event::ElementState::Released,
		}
	}
}

fn update_camera(camera: &mut Camera, dt: f32, key_states: &KeyStates) {
	let move_rate = 5.0; // m/s
	let rotation_rate = 60.0; // Degrees/s

	use glutin::event::ElementState::Pressed;
	if key_states.A == Pressed {
		camera.move_right(-move_rate * dt)
	}
	if key_states.D == Pressed {
		camera.move_right(move_rate * dt)
	}
	if key_states.S == Pressed {
		camera.move_forward(-move_rate * dt)
	}
	if key_states.W == Pressed {
		camera.move_forward(move_rate * dt)
	}

	if key_states.J == Pressed {
		camera.rotate_right(-rotation_rate * dt)
	}
	if key_states.L == Pressed {
		camera.rotate_right(rotation_rate * dt)
	}
	if key_states.K == Pressed {
		camera.rotate_up(-rotation_rate * dt)
	}
	if key_states.I == Pressed {
		camera.rotate_up(rotation_rate * dt)
	}
}
