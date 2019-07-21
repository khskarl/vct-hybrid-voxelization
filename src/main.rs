#![feature(duration_float)]

use rendy::{
	factory::{Config, Factory},
	graph::{present::PresentNode, render::*, GraphBuilder},
	wsi::winit::{self, ElementState, Event, EventsLoop, KeyboardInput, WindowBuilder, WindowEvent},
};

use nalgebra_glm as glm;
use rendy::hal;
mod passes;
use passes::triangle::TrianglePass;

mod scene;
use scene::camera::*;

use std::time::Instant;

#[cfg(feature = "dx12")]
type Backend = rendy::dx12::Backend;
#[cfg(feature = "metal")]
type Backend = rendy::metal::Backend;
#[cfg(feature = "vulkan")]

type Backend = rendy::vulkan::Backend;

#[cfg(not(any(feature = "dx12", feature = "metal", feature = "vulkan")))]
fn main() {
	panic!("You need to specify a feature. E.g. cargo run --features=vulkan");
	Ok(())
}

#[cfg(any(feature = "dx12", feature = "metal", feature = "vulkan"))]
fn main() {
	const WINDOW_TITLE: &str = "Lunar Renderer";

	env_logger::Builder::from_default_env()
		.filter_level(log::LevelFilter::Warn)
		.filter_module("hey you", log::LevelFilter::Trace)
		.init();

	let mut event_loop = EventsLoop::new();

	let window = WindowBuilder::new()
		.with_title(WINDOW_TITLE)
		.with_dimensions(winit::dpi::LogicalSize::new(800.0, 600.0))
		.build(&event_loop)
		.unwrap();

	let config: Config = Default::default();
	let (mut factory, mut families): (Factory<Backend>, _) = rendy::factory::init(config).unwrap();

	let surface = factory.create_surface(&window);
	let size = window
		.get_inner_size()
		.unwrap()
		.to_physical(window.get_hidpi_factor());

	let mut graph_builder = GraphBuilder::<Backend, passes::triangle::Aux>::new();
	let color = graph_builder.create_image(
		hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1),
		1,
		factory.get_surface_format(&surface),
		Some(hal::command::ClearValue::Color(
			[0.01, 0.01, 0.01, 1.0].into(),
		)),
	);

	let pass = graph_builder.add_node(
		TrianglePass::builder()
			.into_subpass()
			.with_color(color)
			.into_pass(),
	);

	graph_builder.add_node(PresentNode::builder(&factory, surface, color).with_dependency(pass));

	let mut camera = Camera::new(glm::vec3(0.0, 0.0, -3.0), 0.0, 0.0);

	let mut aux = passes::triangle::Aux {
		time: 0.666,
		proj: camera.projection(),
		view: camera.view(),
	};

	let mut graph = graph_builder
		.build(&mut factory, &mut families, &mut aux)
		.unwrap();

	let target_dt = 0.01666666666;
	let mut should_exit = false;
	while should_exit == false {
		let start_frame_time = Instant::now();

		factory.maintain(&mut families);

		event_loop.poll_events(|event| match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => should_exit = true,
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state: ElementState::Pressed,
							virtual_keycode: Some(key),
							..
						},
					..
				} => {
					use winit::VirtualKeyCode::*;

					let dt = target_dt;
					let move_rate = 5.0; // m/s
					let rotation_rate = 60.0; // Degrees/s
					match key {
						Escape => should_exit = true,
						A => camera.move_right(-move_rate * dt),
						D => camera.move_right(move_rate * dt),
						S => camera.move_forward(-move_rate * dt),
						W => camera.move_forward(move_rate * dt),

						J => camera.rotate_right(-rotation_rate * dt),
						L => camera.rotate_right(rotation_rate * dt),
						K => camera.rotate_up(-rotation_rate * dt),
						I => camera.rotate_up(rotation_rate * dt),

						Z => aux.time -= 0.05 * dt,
						X => aux.time += 0.05 * dt,
						_ => (),
					}
				}
				_ => (),
			},
			Event::DeviceEvent { event, .. } => match event {
				_ => (),
			},
			_ => (),
		});

		aux.proj = camera.projection();
		aux.view = camera.view();

		// println!("----------------");
		// println!("Proj: {}", aux.proj);
		// println!("View: {}", aux.view);
		// println!("Forward: {}", camera.forward());

		graph.run(&mut factory, &mut families, &mut aux);

		let dt = Instant::now()
			.duration_since(start_frame_time)
			.as_secs_f32();
		window.set_title(&format!("{} | {:.6}", WINDOW_TITLE, dt));
	}

	graph.dispose(&mut factory, &mut aux);
}
