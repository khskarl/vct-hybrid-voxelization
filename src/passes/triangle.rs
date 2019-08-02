use rendy::{
	command::{QueueId, RenderPassEncoder},
	factory::{Factory, ImageState},
	graph::{render::*, GraphContext, NodeBuffer, NodeImage},
	memory::MemoryUsageValue,
	mesh::{AsVertex, Mesh, PosTex},
	shader::{ShaderKind, SourceLanguage, StaticShaderInfo},
	texture::{pixel::Rgba8Srgb, Texture, TextureBuilder},
};

use rendy::hal;
use rendy::hal::{pso::DescriptorPool, Device};

use rendy::{
	resource::{Buffer, BufferInfo, DescriptorSetLayout, Escape, Handle},
};

use nalgebra_glm as glm;

use genmesh::{
	generators::{IndexedPolygon, SharedVertex},
	Triangulate,
};

use gltf;

use image;
use image::Pixel;

use std::mem::size_of;

lazy_static::lazy_static! {
	static ref VERTEX: StaticShaderInfo = StaticShaderInfo::new(
		concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/triangle.vs"),
		ShaderKind::Vertex,
		SourceLanguage::GLSL,
		"main",
	);

	static ref FRAGMENT: StaticShaderInfo = StaticShaderInfo::new(
		concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/triangle.fs"),
		ShaderKind::Fragment,
		SourceLanguage::GLSL,
		"main",
	);

	static ref SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
		.with_vertex(&*VERTEX).unwrap()
		.with_fragment(&*FRAGMENT).unwrap();
}

#[derive(Clone, Copy)]
#[repr(C)]
struct UniformArgs {
	proj: glm::Mat4,
	view: glm::Mat4,
	time: f32,
}

pub struct Aux {
	pub proj: glm::Mat4,
	pub view: glm::Mat4,
	pub time: f32,
}

#[derive(Debug, Default)]
pub struct TrianglePassDesc;

#[derive(Debug)]
pub struct TrianglePass<B: hal::Backend> {
	uniform_buffer: Escape<Buffer<B>>,
	descriptor_pool: B::DescriptorPool,
	frame_sets: Vec<B::DescriptorSet>,
	mesh: Mesh<B>,
	texture: Texture<B>,
}

impl<B> SimpleGraphicsPipelineDesc<B, Aux> for TrianglePassDesc
where
	B: hal::Backend,
{
	type Pipeline = TrianglePass<B>;

	fn layout(&self) -> Layout {
		let frame_layout = SetLayout {
			bindings: vec![
				hal::pso::DescriptorSetLayoutBinding {
					binding: 0,
					ty: hal::pso::DescriptorType::UniformBuffer,
					count: 1,
					stage_flags: hal::pso::ShaderStageFlags::GRAPHICS,
					immutable_samplers: false,
				},
				hal::pso::DescriptorSetLayoutBinding {
					binding: 1,
					ty: hal::pso::DescriptorType::SampledImage,
					count: 1,
					stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
					immutable_samplers: false,
				},
				hal::pso::DescriptorSetLayoutBinding {
					binding: 2,
					ty: hal::pso::DescriptorType::Sampler,
					count: 1,
					stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
					immutable_samplers: false,
				},
			],
		};

		Layout {
			sets: vec![frame_layout],
			push_constants: Vec::new(),
		}
	}

	fn vertices(
		&self,
	) -> Vec<(
		Vec<hal::pso::Element<hal::format::Format>>,
		hal::pso::ElemStride,
		hal::pso::VertexInputRate,
	)> {
		vec![PosTex::vertex().gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)]
	}

	fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &Aux) -> rendy::shader::ShaderSet<B> {
		SHADERS.build(factory, Default::default()).unwrap()
	}

	fn build<'a>(
		self,
		_ctx: &GraphContext<B>,
		factory: &mut Factory<B>,
		queue: QueueId,
		_aux: &Aux,
		buffers: Vec<NodeBuffer>,
		images: Vec<NodeImage>,
		set_layouts: &[Handle<DescriptorSetLayout<B>>],
	) -> Result<TrianglePass<B>, failure::Error> {
		assert!(buffers.is_empty());
		assert!(images.is_empty());
		// assert_eq!(set_layouts.len(), 3);

		let uniform_buffer = factory
			.create_buffer(
				BufferInfo {
					size: size_of::<UniformArgs>() as u64,
					usage: hal::buffer::Usage::UNIFORM,
				},
				MemoryUsageValue::Dynamic,
			)
			.unwrap();

		let (_, buffers, images) = gltf::import("assets/models/box.gltf")?;
		println!("buffers length {}:", buffers.len());
		println!("images length {}:", images.len());

		let cube_generator = genmesh::generators::Cube::new();
		let cube_indices: Vec<_> =
			genmesh::Vertices::vertices(cube_generator.indexed_polygon_iter().triangulate())
				.map(|i| i as u32)
				.collect();
		let cube_vertices: Vec<_> = cube_generator
			.shared_vertex_iter()
			.map(|v| {
				let pos = v.pos;
				let tex_coords = [pos.x, pos.y.max(pos.z)];

				PosTex {
					position: pos.into(),
					tex_coord: tex_coords.into(),
				}
			})
			.collect();

		let cube_mesh = Mesh::<B>::builder()
			.with_indices(&cube_indices[..])
			.with_vertices(&cube_vertices[..])
			.build(queue, factory)
			.unwrap();

		let cube_tex_bytes = include_bytes!("../../assets/textures/ground_color.jpg");
		let cube_tex_img = image::load_from_memory(&cube_tex_bytes[..])
			.unwrap()
			.to_rgba();

		let (w, h) = cube_tex_img.dimensions();
		let cube_tex_image_data: Vec<Rgba8Srgb> = cube_tex_img
			.pixels()
			.map(|p| {
				use std::convert::TryInto;
				Rgba8Srgb {
					repr: p
						.channels()
						.try_into()
						.expect("slice with incorrect length"),
				}
			})
			.collect::<_>();

		let cube_tex_builder = TextureBuilder::new()
			.with_kind(hal::image::Kind::D2(w, h, 1, 1))
			.with_view_kind(hal::image::ViewKind::D2)
			.with_data_width(w)
			.with_data_height(h)
			.with_data(&cube_tex_image_data);

		let texture = cube_tex_builder
			.build(
				ImageState {
					queue,
					stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
					access: hal::image::Access::SHADER_READ,
					layout: hal::image::Layout::ShaderReadOnlyOptimal,
				},
				factory,
			)
			.unwrap();

		let mut descriptor_pool = unsafe {
			factory.create_descriptor_pool(
				1,
				vec![
					hal::pso::DescriptorRangeDesc {
						ty: hal::pso::DescriptorType::UniformBuffer,
						count: 1,
					},
					hal::pso::DescriptorRangeDesc {
						ty: hal::pso::DescriptorType::Sampler,
						count: 1,
					},
					hal::pso::DescriptorRangeDesc {
						ty: hal::pso::DescriptorType::SampledImage,
						count: 1,
					},
				],
				hal::pso::DescriptorPoolCreateFlags::empty(),
			)
		}
		.unwrap();

		let mut frame_set;
		unsafe {
			frame_set = descriptor_pool.allocate_set(&set_layouts[0].raw()).unwrap();

			factory.write_descriptor_sets(vec![
				hal::pso::DescriptorSetWrite {
					set: &frame_set,
					binding: 0,
					array_offset: 0,
					descriptors: Some(hal::pso::Descriptor::Buffer(
						uniform_buffer.raw(),
						Some(0_u64)..Some(0_u64 + size_of::<UniformArgs> as u64),
					)),
				},
				hal::pso::DescriptorSetWrite {
					set: &frame_set,
					binding: 1,
					array_offset: 0,
					descriptors: Some(hal::pso::Descriptor::Image(
						texture.view().raw(),
						hal::image::Layout::ShaderReadOnlyOptimal,
					)),
				},
				hal::pso::DescriptorSetWrite {
					set: &frame_set,
					binding: 2,
					array_offset: 0,
					descriptors: Some(hal::pso::Descriptor::Sampler(texture.sampler().raw())),
				},
			]);
		}

		let frame_sets = vec![frame_set];

		Ok(TrianglePass {
			uniform_buffer,
			descriptor_pool,
			frame_sets,
			mesh: cube_mesh,
			texture,
		})
	}
}

impl<B> SimpleGraphicsPipeline<B, Aux> for TrianglePass<B>
where
	B: hal::Backend,
{
	type Desc = TrianglePassDesc;

	fn prepare(
		&mut self,
		factory: &Factory<B>,
		_queue: QueueId,
		_set_layouts: &[Handle<DescriptorSetLayout<B>>],
		_index: usize,
		aux: &Aux,
	) -> PrepareResult {
		unsafe {
			factory
				.upload_visible_buffer(
					&mut self.uniform_buffer,
					0,
					&[UniformArgs {
						proj: {
							let mut proj = aux.proj;
							proj[(0, 0)] *= -1.0;
							proj[(1, 1)] *= -1.0;
							proj
						},
						view: aux.view,
						time: aux.time,
					}],
				)
				.unwrap();
		}
		PrepareResult::DrawReuse
	}

	fn draw(
		&mut self,
		layout: &B::PipelineLayout,
		mut encoder: RenderPassEncoder<'_, B>,
		index: usize,
		_aux: &Aux,
	) {
		unsafe {
			encoder.bind_graphics_descriptor_sets(
				layout,
				0,
				Some(&self.frame_sets[0]),
				std::iter::empty(),
			);
		}

		assert!(self.mesh.bind(0, &[PosTex::vertex()], &mut encoder).is_ok());

		unsafe {
			encoder.draw_indexed(0..self.mesh.len(), 0, 0..1);
		}
	}

	fn dispose(mut self, factory: &mut Factory<B>, _aux: &Aux) {
		unsafe {
			self.descriptor_pool.free_sets(self.frame_sets.into_iter());
			factory.destroy_descriptor_pool(self.descriptor_pool);
		}
	}
}
