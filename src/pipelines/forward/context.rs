use super::{depth_fshader, depth_vshader, pipeline::ForwardPipeline, swap_fshader, swap_vshader, DEPTH_FORMAT};
use crate::{
	pipelines::{Pipeline, PipelineContext},
	surface::SWAP_FORMAT,
};
use std::sync::Arc;
use vulkano::{
	descriptor::{descriptor::ShaderStages, pipeline_layout::PipelineLayoutDesc, PipelineLayoutAbstract},
	device::Device,
	framebuffer::RenderPassAbstract,
	image::ImageViewAccess,
};

pub(super) struct ForwardPipelineContext {
	inner: Arc<ForwardPipelineContextInner>,
}
impl ForwardPipelineContext {
	pub(super) fn new(device: &Arc<Device>) -> Self {
		let depth_pass = Arc::new(
			vulkano::single_pass_renderpass!(
				device.clone(),
				attachments: { depth: { load: Clear, store: Store, format: DEPTH_FORMAT, samples: 1, } },
				pass: { color: [], depth_stencil: {depth} }
			)
			.unwrap(),
		);
		let depth_vshader = depth_vshader::Shader::load(device.clone()).unwrap();
		let depth_fshader = depth_fshader::Shader::load(device.clone()).unwrap();

		let swap_pass = Arc::new(
			vulkano::single_pass_renderpass!(
				device.clone(),
				attachments: { color: { load: Clear, store: Store, format: SWAP_FORMAT, samples: 1, } },
				pass: { color: [color], depth_stencil: {} }
			)
			.unwrap(),
		);
		let swap_vshader = swap_vshader::Shader::load(device.clone()).unwrap();
		let swap_fshader = swap_fshader::Shader::load(device.clone()).unwrap();

		let vs_layout = swap_vshader::Layout(ShaderStages { vertex: true, ..ShaderStages::none() });
		let fs_layout = swap_fshader::Layout(ShaderStages { fragment: true, ..ShaderStages::none() });
		let layout_desc = Arc::new(vs_layout.union(fs_layout).build(device.clone()).unwrap());

		Self {
			inner: Arc::new(ForwardPipelineContextInner {
				depth_pass,
				depth_vshader,
				depth_fshader,
				swap_pass,
				swap_vshader,
				swap_fshader,
				layout_desc,
			}),
		}
	}
}
impl PipelineContext for ForwardPipelineContext {
	fn make_pipeline(
		&self,
		images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
		dimensions: [u32; 2],
	) -> Box<dyn Pipeline> {
		Box::new(ForwardPipeline::new(self.inner.clone(), images, dimensions))
	}

	fn layout_desc(&self) -> &Arc<dyn PipelineLayoutAbstract + Send + Sync> {
		&self.inner.layout_desc
	}
}

pub(super) struct ForwardPipelineContextInner {
	pub(super) depth_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	pub(super) depth_vshader: depth_vshader::Shader,
	pub(super) depth_fshader: depth_fshader::Shader,
	pub(super) swap_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	pub(super) swap_vshader: swap_vshader::Shader,
	pub(super) swap_fshader: swap_fshader::Shader,
	pub(super) layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
}
