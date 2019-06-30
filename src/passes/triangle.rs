use rendy::{
  command::{QueueId, RenderPassEncoder},
  factory::Factory,
  graph::{render::*, GraphContext, NodeBuffer, NodeImage},
  mesh::{AsVertex, PosColor},
  shader::{ShaderKind, SourceLanguage, StaticShaderInfo},
};

use rendy::hal;
use rendy::{
  memory::Dynamic,
  resource::{Buffer, BufferInfo, DescriptorSetLayout, Escape, Handle},
};


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

#[derive(Debug, Default)]
pub struct TrianglePassDesc;

#[derive(Debug)]
pub struct TrianglePass<B: hal::Backend> {
  vertex: Escape<Buffer<B>>,
}

impl<B, T> SimpleGraphicsPipelineDesc<B, T> for TrianglePassDesc
where
  B: hal::Backend,
  T: ?Sized,
{
  type Pipeline = TrianglePass<B>;

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

  fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &T) -> rendy::shader::ShaderSet<B> {
    SHADERS.build(factory, Default::default()).unwrap()
  }

  fn build<'a>(
    self,
    _ctx: &GraphContext<B>,
    factory: &mut Factory<B>,
    queue: QueueId,
    aux: &T,
    buffers: Vec<NodeBuffer>,
    images: Vec<NodeImage>,
    set_layouts: &[Handle<DescriptorSetLayout<B>>],
  ) -> Result<TrianglePass<B>, failure::Error> {
    assert!(buffers.is_empty());
    assert!(images.is_empty());
    assert!(set_layouts.is_empty());

    let mut vbuf = factory
      .create_buffer(
        BufferInfo {
          size: PosColor::vertex().stride as u64 * 3,
          usage: hal::buffer::Usage::VERTEX,
        },
        Dynamic,
      )
      .unwrap();

    unsafe {
      factory
        .upload_visible_buffer(
          &mut vbuf,
          0,
          &[
            PosColor {
              position: [0.0, -0.5, 0.0].into(),
              color: [1.0, 0.0, 0.0, 1.0].into(),
            },
            PosColor {
              position: [0.5, 0.5, 0.0].into(),
              color: [0.0, 1.0, 0.0, 1.0].into(),
            },
            PosColor {
              position: [-0.5, 0.5, 0.0].into(),
              color: [0.0, 0.0, 1.0, 1.0].into(),
            },
          ],
        )
        .unwrap();
    }

    Ok(TrianglePass { vertex: vbuf })
  }
}

impl<B, T> SimpleGraphicsPipeline<B, T> for TrianglePass<B>
where
  B: hal::Backend,
  T: ?Sized,
{
  type Desc = TrianglePassDesc;

  fn prepare(
    &mut self,
    factory: &Factory<B>,
    _queue: QueueId,
    _set_layouts: &[Handle<DescriptorSetLayout<B>>],
    index: usize,
    aux: &T,
  ) -> PrepareResult {
    PrepareResult::DrawReuse
  }

  fn draw(
    &mut self,
    _layout: &B::PipelineLayout,
    mut encoder: RenderPassEncoder<'_, B>,
    _index: usize,
    _aux: &T,
  ) {
    encoder.bind_vertex_buffers(0, Some((self.vertex.raw(), 0)));
    encoder.draw(0..3, 0..1);
  }

  fn dispose(self, _factory: &mut Factory<B>, _aux: &T) {}
}
