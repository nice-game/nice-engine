use super::{
	geom_fshader, geom_vshader, pipeline::DeferredPipeline, swap_fshader, swap_vshader, Vert2D, DEPTH_FORMAT,
	DIFFUSE_FORMAT, NORMAL_FORMAT,
};
use crate::{
	pipelines::{Pipeline, PipelineContext},
	surface::SWAP_FORMAT,
};
use std::sync::Arc;
use vulkano::{
	buffer::{BufferAccess, BufferUsage, ImmutableBuffer, TypedBufferAccess},
	descriptor::{descriptor::ShaderStages, pipeline_layout::PipelineLayoutDesc, PipelineLayoutAbstract},
	device::{Device, Queue},
	framebuffer::RenderPassAbstract,
	image::ImageViewAccess,
	sync::GpuFuture,
};

pub struct DeferredPipelineContext {
	inner: Arc<DeferredPipelineContextInner>,
}
impl DeferredPipelineContext {
	pub(crate) fn new(device: &Arc<Device>, queue: &Arc<Queue>) -> Self {
		let render_pass = Arc::new(
			vulkano::ordered_passes_renderpass!(
				device.clone(),
				attachments: {
					diffuse: { load: Clear, store: DontCare, format: DIFFUSE_FORMAT, samples: 1, },
					normal: { load: Clear, store: DontCare, format: NORMAL_FORMAT, samples: 1, },
					depth: { load: Clear, store: DontCare, format: DEPTH_FORMAT, samples: 1, },
					color: { load: Clear, store: Store, format: SWAP_FORMAT, samples: 1, }
				},
				passes: [
					{ color: [diffuse, normal], depth_stencil: {depth}, input: [] },
					{ color: [color], depth_stencil: {}, input: [diffuse, normal, depth] }
				]
			)
			.unwrap(),
		);

		let geom_vshader = geom_vshader::Shader::load(device.clone()).unwrap();
		let geom_fshader = geom_fshader::Shader::load(device.clone()).unwrap();
		let vs_layout = geom_vshader::Layout(ShaderStages { vertex: true, ..ShaderStages::none() });
		let fs_layout = geom_fshader::Layout(ShaderStages { fragment: true, ..ShaderStages::none() });
		let layout_desc = Arc::new(vs_layout.union(fs_layout).build(device.clone()).unwrap());

		let swap_vshader = swap_vshader::Shader::load(device.clone()).unwrap();
		let swap_fshader = swap_fshader::Shader::load(device.clone()).unwrap();
		let swap_vs_layout = swap_vshader::Layout(ShaderStages { vertex: true, ..ShaderStages::none() });
		let swap_fs_layout = swap_fshader::Layout(ShaderStages { fragment: true, ..ShaderStages::none() });
		let swap_layout_desc = Arc::new(swap_vs_layout.union(swap_fs_layout).build(device.clone()).unwrap());

		let vertdata = [
			Vert2D { pos: [-1.0, 1.0], texc: [0.0, 0.0] },
			Vert2D { pos: [1.0, 1.0], texc: [1.0, 0.0] },
			Vert2D { pos: [1.0, -1.0], texc: [1.0, 1.0] },
			Vert2D { pos: [-1.0, -1.0], texc: [0.0, 1.0] },
		];
		let (vertices, vertices_future) =
			ImmutableBuffer::from_data(vertdata, BufferUsage::vertex_buffer(), queue.clone()).unwrap();
		let (indices, indices_future) =
			ImmutableBuffer::from_iter(vec![0, 1, 2, 2, 3, 0].into_iter(), BufferUsage::index_buffer(), queue.clone())
				.unwrap();
		vertices_future.join(indices_future).then_signal_fence_and_flush().unwrap().wait(None).unwrap();

		Self {
			inner: Arc::new(DeferredPipelineContextInner {
				render_pass,
				geom_vshader,
				geom_fshader,
				layout_desc,
				swap_vshader,
				swap_fshader,
				swap_layout_desc,
				vertices,
				indices,
			}),
		}
	}
}
impl PipelineContext for DeferredPipelineContext {
	fn make_pipeline(
		&self,
		images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
		dimensions: [u32; 2],
	) -> Box<dyn Pipeline> {
		Box::new(DeferredPipeline::new(self.inner.clone(), images, dimensions))
	}

	fn layout_desc(&self) -> &Arc<dyn PipelineLayoutAbstract + Send + Sync> {
		&self.inner.layout_desc
	}
}

pub(crate) struct DeferredPipelineContextInner {
	pub(crate) render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	pub(crate) geom_vshader: geom_vshader::Shader,
	pub(crate) geom_fshader: geom_fshader::Shader,
	pub(crate) layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	pub(crate) swap_vshader: swap_vshader::Shader,
	pub(crate) swap_fshader: swap_fshader::Shader,
	pub(crate) swap_layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	pub(crate) vertices: Arc<dyn BufferAccess + Send + Sync>,
	pub(crate) indices: Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>,
}