use rendy::{
	command::{QueueId, RenderPassEncoder},
	factory::Factory,
	graph::{render::*, GraphContext, NodeBuffer, NodeImage},
	memory::MemoryUsageValue,
	mesh::{AsVertex, Mesh, PosColor},
	shader::{ShaderKind, SourceLanguage, StaticShaderInfo},
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
	dynamic_set: B::DescriptorSet,
	mesh: Mesh<B>,
}

impl<B> SimpleGraphicsPipelineDesc<B, Aux> for TrianglePassDesc
where
	B: hal::Backend,
{
	type Pipeline = TrianglePass<B>;

	fn layout(&self) -> Layout {
		let dynamic_ubo_layout = SetLayout {
			bindings: vec![hal::pso::DescriptorSetLayoutBinding {
				binding: 0,
				ty: hal::pso::DescriptorType::UniformBuffer,
				count: 1,
				stage_flags: hal::pso::ShaderStageFlags::GRAPHICS,
				immutable_samplers: false,
			}],
		};

		Layout {
			sets: vec![dynamic_ubo_layout],
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
		vec![PosColor::vertex().gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)]
	}

	fn depth_stencil(&self) -> Option<hal::pso::DepthStencilDesc> {
		None
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
		assert_eq!(set_layouts.len(), 1);

		let uniform_buffer = factory
			.create_buffer(
				BufferInfo {
					size: size_of::<UniformArgs>() as u64,
					usage: hal::buffer::Usage::UNIFORM,
				},
				MemoryUsageValue::Dynamic,
			)
			.unwrap();

		let cube_generator = genmesh::generators::Cube::new();
		let cube_indices: Vec<_> =
			genmesh::Vertices::vertices(cube_generator.indexed_polygon_iter().triangulate())
				.map(|i| i as u32)
				.collect();
		let cube_vertices: Vec<_> = cube_generator
			.shared_vertex_iter()
			.map(|v| {
				let pos = v.pos;
				let color = [pos.x, pos.y, pos.z, 1.0];

				PosColor {
					position: pos.into(),
					color: color.into(),
				}
			})
			.collect();

		let cube_mesh = Mesh::<B>::builder()
			.with_indices(&cube_indices[..])
			.with_vertices(&cube_vertices[..])
			.build(queue, factory)
			.unwrap();

		let mut descriptor_pool = unsafe {
			factory.create_descriptor_pool(
				1,
				vec![hal::pso::DescriptorRangeDesc {
					ty: hal::pso::DescriptorType::UniformBuffer,
					count: 1,
				}],
				hal::pso::DescriptorPoolCreateFlags::empty(),
			)
		}
		.unwrap();

		let mut dynamic_set;
		unsafe {
			dynamic_set = descriptor_pool.allocate_set(&set_layouts[0].raw()).unwrap();
			factory.write_descriptor_sets(vec![hal::pso::DescriptorSetWrite {
				set: &dynamic_set,
				binding: 0,
				array_offset: 0,
				descriptors: Some(hal::pso::Descriptor::Buffer(
					uniform_buffer.raw(),
					Some(0_u64)..Some(0_u64 + size_of::<UniformArgs> as u64),
				)),
			}]);
		}

		Ok(TrianglePass {
			uniform_buffer,
			descriptor_pool,
			dynamic_set,
			mesh: cube_mesh,
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
		_index: usize,
		_aux: &Aux,
	) {
		encoder.bind_graphics_descriptor_sets(layout, 0, Some(&self.dynamic_set), std::iter::empty());

		assert!(self
			.mesh
			.bind(0, &[PosColor::vertex()], &mut encoder)
			.is_ok());

		encoder.draw_indexed(0..self.mesh.len(), 0, 0..1);
	}

	fn dispose(self, _factory: &mut Factory<B>, _aux: &Aux) {}
}
