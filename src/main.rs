use rendy::{
    factory::{Config, Factory},
    graph::{present::PresentNode, render::*, GraphBuilder},
    wsi::winit::{
        self, ElementState, Event, EventsLoop, KeyboardInput, WindowBuilder, WindowEvent,
    },
};


use nalgebra_glm as glm;
use rendy::hal;
mod passes;
use passes::triangle::TrianglePass;

mod scene;
use scene::camera::*;

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
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("hey you", log::LevelFilter::Trace)
        .init();

    let mut event_loop = EventsLoop::new();

    let window = WindowBuilder::new()
        .with_title("Lunar Renderer")
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


    let mut should_exit = false;
    while should_exit == false {
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
                    match key {
                        Escape => should_exit = true,
                        A => camera.move_right(-0.05),
                        D => camera.move_right(0.05),
                        S => camera.move_forward(-0.05),
                        W => camera.move_forward(0.05),

                        J => camera.rotate_right(-0.05),
                        L => camera.rotate_right(0.05),
                        K => camera.rotate_up(-0.05),
                        I => camera.rotate_up(0.05),

                        Z => aux.time -= 0.05,
                        X => aux.time += 0.05,
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

        println!("----------------");
        println!("Proj: {}", aux.proj);
        println!("View: {}", aux.view);


        graph.run(&mut factory, &mut families, &mut aux);
    }

    graph.dispose(&mut factory, &mut aux);
}

