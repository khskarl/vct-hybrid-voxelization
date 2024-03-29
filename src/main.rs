use std::str;
use std::time::Instant;

mod gl_timer;
mod gl_utils;
mod gpu_model;
mod renderer;
mod renderer_utils;
mod textures;

use nalgebra_glm as glm;

mod scene;
use renderer::*;
use scene::camera::*;
use scene::model::{Mesh, Resources};

use imgui_winit_support::{HiDpiMode, WinitPlatform};

struct ImGuiState {
	resolution_index: usize,
}

fn test_scene(renderer: &mut Renderer, camera: &mut Camera, scene_name: &mut &'static str) {
	*scene_name = "test";

	use glm::vec3;

	let mut resources = Resources::new();
	renderer.submit_mesh(&Mesh::new(
		"assets/models/test.glb",
		vec3(0.0, 2.0, 0.0),
		vec3(2.0, 2.0, 2.0),
		&mut resources,
	));

	let volume = renderer.volume_mut();
	*volume.translation_mut() = vec3(0.0, 5.0, 0.0);
	*volume.scaling_mut() = vec3(10.0, 10.0, 10.0);
	*volume.view_translation_mut() = vec3(10.15, 5.0, 0.0);
	*volume.view_scaling_mut() = vec3(10.0, 10.0, 10.0);

	camera.position = vec3(5.0, 2.0, 10.0);
	camera.yaw = -90.0;
	camera.pitch = 0.0;
}

fn sponza_scene(renderer: &mut Renderer, camera: &mut Camera, scene_name: &mut &'static str) {
	*scene_name = "sponza";

	use glm::vec3;

	let mut resources = Resources::new();
	renderer.submit_mesh(&Mesh::new(
		"assets/models/sponza.glb",
		vec3(0.0, 0.0, 0.0),
		vec3(1.0, 1.0, 1.0),
		&mut resources,
	));

	let volume = renderer.volume_mut();
	*volume.translation_mut() = vec3(0.0, 5.0, 0.0);
	*volume.scaling_mut() = vec3(24.0, 10.1, 12.0);
	*volume.view_translation_mut() = vec3(0.0, 5.0, 0.0);
	*volume.view_scaling_mut() = vec3(24.0, 10.1, 12.0);

	camera.position = vec3(4.0, 2.0, 0.0);
	camera.yaw = 0.0;
	camera.pitch = 0.0;
}

fn cornell_scene(renderer: &mut Renderer, camera: &mut Camera, scene_name: &mut &'static str) {
	*scene_name = "cornell";

	use glm::vec3;

	let mut resources = Resources::new();
	renderer.submit_mesh(&Mesh::new(
		"assets/models/sphere.glb",
		vec3(0.0, 1.4, 5.0),
		vec3(1.0, 1.0, 1.0),
		&mut resources,
	));
	renderer.submit_mesh(&Mesh::new(
		"assets/models/cornell_box.glb",
		vec3(0.0, 0.0, 0.0),
		vec3(1.0, 1.0, 1.0),
		&mut resources,
	));

	let volume = renderer.volume_mut();
	*volume.translation_mut() = vec3(0.0, 5.0, 0.0);
	*volume.scaling_mut() = vec3(10.0, 10.0, 10.0);
	*volume.view_translation_mut() = vec3(10.15, 5.0, 0.0);
	*volume.view_scaling_mut() = vec3(10.0, 10.0, 10.0);

	camera.position = vec3(0.0, 5.0, 10.0);
	camera.yaw = -90.0;
	camera.pitch = 0.0;
}

fn main() {
	const WINDOW_TITLE: &str = "Potato Renderer 🥟";
	let (width, height) = (1280, 720);

	let logical_size = glutin::dpi::LogicalSize::from((width, height));
	let window_builder = glutin::window::WindowBuilder::new()
		.with_title(WINDOW_TITLE)
		.with_inner_size(logical_size);

	let event_loop = glutin::event_loop::EventLoop::new();
	let window_gl = {
		let window_gl = glutin::ContextBuilder::new()
			.with_gl_profile(glutin::GlProfile::Core)
			.build_windowed(window_builder, &event_loop)
			.unwrap();
		unsafe { window_gl.make_current() }.unwrap()
	};

	let mut imgui = imgui::Context::create();
	let mut platform = WinitPlatform::init(&mut imgui);
	platform.attach_window(imgui.io_mut(), &window_gl.window(), HiDpiMode::Default);

	let resolutions = [64, 128, 256];
	let res_index = 0;
	let conservative = true;

	// Renderer setup
	let mut renderer = renderer::Renderer::new(
		&window_gl,
		logical_size,
		resolutions[res_index],
		conservative,
	);

	let imgui_renderer =
		imgui_opengl_renderer::Renderer::new(&mut imgui, |s| window_gl.get_proc_address(s) as _);

	let mut imgui_state = ImGuiState {
		resolution_index: res_index,
	};

	let mut camera = Camera::new(glm::vec3(0.0, 0.0, 0.0), 0.0, 0.0);
	let mut scene_name = "INVALID";
	// cornell_scene(&mut renderer, &mut camera, &mut scene_name);
	sponza_scene(&mut renderer, &mut camera, &mut scene_name);
	// test_scene(&mut renderer, &mut camera, &mut scene_name);

	let mut key_states = KeyStates::new();

	const MAX_DELTAS: usize = 60;
	let target_dt = 0.016_666_668;
	let mut curr_frame: usize = 0;
	let mut delta_times = [target_dt; MAX_DELTAS];
	let mut dt: f32 = delta_times.iter().sum();
	dt /= MAX_DELTAS as f32;
	let initial_time = Instant::now();
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
					let physical_size = logical_size.to_physical(dpi_factor);

					window_gl.resize(physical_size);
					imgui.io_mut().display_size = [physical_size.width as f32, physical_size.height as f32];
					renderer.set_viewport_size((physical_size.width as usize, physical_size.height as usize));
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

						(P, Released) => renderer.save_diagnostics(scene_name),

						_ => (),
					}
				}
				WindowEvent::RedrawRequested => {
					imgui.io_mut().update_delta_time(start_frame_time);

					let ui = imgui.frame();
					{
						use imgui::*;
						use std::borrow::Cow;

						Window::new(im_str!("Diagnostics"))
							.size([300.0, 100.0], Condition::FirstUseEver)
							.build(&ui, || {
								ui.text(format!("Frame rate: {:.2} frames/s", 1.0 / dt));
								ui.text(format!("Frame time: {:.2} ms", dt * 1000.0));
								ui.separator();

								let mouse_pos = ui.io().mouse_pos;
								ui.text(format!(
									"Mouse Position: ({:.1},{:.1})",
									mouse_pos[0], mouse_pos[1]
								));
							});

						Window::new(im_str!("Lights")).build(&ui, || {
							let lights = &mut renderer.lights;
							for (i, light) in lights.iter_mut().enumerate() {
								ColorEdit::new(&im_str!("Color#{}", i), light.color.as_mut()).build(&ui);
								ui.drag_float3(&im_str!("Position#{}", i), light.position.as_mut())
									.build();
							}
						});

						Window::new(im_str!("Voxels")).build(&ui, || {
							let volume = renderer.volume_mut();
							ui.text(im_str!("Voxels:"));

							ui.drag_float3(im_str!("Translation"), volume.translation_mut().as_mut())
								.build();
							ui.drag_float3(im_str!("Scale"), volume.scaling_mut().as_mut())
								.build();
							ui.drag_float3(
								im_str!("ViewTranslation"),
								volume.view_translation_mut().as_mut(),
							)
							.build();
							ui.drag_float3(im_str!("ViewScale"), volume.view_scaling_mut().as_mut())
								.build();

							let index = &mut imgui_state.resolution_index;
							ComboBox::new(im_str!("Resolution")).build_simple(&ui, index, &resolutions, &|x| {
								Cow::from(im_str!("{}x{}x{}", x, x, x))
							});
							*volume.resolution_mut() = resolutions[*index];
							ui.separator();

							ui.radio_button(
								im_str!("Albedo"),
								&mut renderer.rendering_mode,
								RenderingMode::Albedo,
							);
							ui.same_line(100.0);

							ui.radio_button(
								im_str!("Normal"),
								&mut renderer.rendering_mode,
								RenderingMode::Normal,
							);
							ui.same_line(200.0);

							ui.radio_button(
								im_str!("Emission"),
								&mut renderer.rendering_mode,
								RenderingMode::Emission,
							);
							ui.radio_button(
								im_str!("Radiance"),
								&mut renderer.rendering_mode,
								RenderingMode::Radiance,
							);
							ui.same_line(100.0);

							ui.radio_button(
								im_str!("None"),
								&mut renderer.rendering_mode,
								RenderingMode::Scene,
							);

							ui.separator();
							ui.radio_button(
								im_str!("Fragment"),
								&mut renderer.voxelization_mode,
								VoxelizationMode::FragmentOnly,
							);
							ui.same_line(100.0);
							ui.radio_button(
								im_str!("Hybrid"),
								&mut renderer.voxelization_mode,
								VoxelizationMode::Hybrid,
							);
							ui.separator();

							Slider::new(im_str!("Cutoff"), 0.1..=10.0)
								.display_format(im_str!("%.1f"))
								.build(&ui, &mut renderer.cutoff);

							ui.drag_float(&im_str!("Cutoff"), &mut renderer.cutoff)
								.min(0.1)
								.max(18000.0)
								.build();
							ui.checkbox(
								im_str!("GL_NV_conversative_raster"),
								&mut renderer.nv_conservative,
							);
							ui.checkbox(im_str!("Show bounds"), &mut renderer.show_bounds);
						});

						Window::new(im_str!("Transforms")).build(&ui, || {
							for (i, primitive) in renderer.primitives_mut().iter_mut().enumerate() {
								ui.drag_float3(
									&im_str!("Translation##{}", i),
									primitive.translation_mut().as_mut(),
								)
								.min(-100.0)
								.max(100.0)
								.build();
								ui.drag_float3(&im_str!("Scale##{}", i), primitive.scaling_mut().as_mut())
									.min(-100.0)
									.max(100.0)
									.build();
							}
						});
					}

					renderer.render(&camera);

					imgui_renderer.render(ui);
					window_gl.swap_buffers().unwrap();
				}
				_ => (),
			},
			Event::EventsCleared => {
				update_camera(&mut camera, dt as f32, &key_states);
				let primitives = renderer.primitives_mut();

				primitives[0].translation_mut().as_mut()[0] =
					initial_time.elapsed().as_secs_f32().cos() * 3.5;

				primitives[0].translation_mut().as_mut()[2] =
					initial_time.elapsed().as_secs_f32().sin() * 3.5;

				delta_times[curr_frame] = Instant::now()
					.duration_since(start_frame_time)
					.as_secs_f32();
				dt = delta_times.iter().sum();
				dt /= MAX_DELTAS as f32;
				curr_frame = (curr_frame + 1) % MAX_DELTAS;
				imgui.io_mut().delta_time = dt as f32;

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
		camera.move_right(move_rate * dt)
	}
	if key_states.D == Pressed {
		camera.move_right(-move_rate * dt)
	}
	if key_states.S == Pressed {
		camera.move_forward(-move_rate * dt)
	}
	if key_states.W == Pressed {
		camera.move_forward(move_rate * dt)
	}

	if key_states.J == Pressed {
		camera.rotate_right(rotation_rate * dt)
	}
	if key_states.L == Pressed {
		camera.rotate_right(-rotation_rate * dt)
	}
	if key_states.K == Pressed {
		camera.rotate_up(-rotation_rate * dt)
	}
	if key_states.I == Pressed {
		camera.rotate_up(rotation_rate * dt)
	}
}
