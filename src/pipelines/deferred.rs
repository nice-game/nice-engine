mod context;
mod pipeline;

use self::context::DeferredPipelineContext;
use vulkano::sync::GpuFuture;

use crate::pipelines::{PipelineContext, PipelineDef};
use std::sync::Arc;
use vulkano::{
	device::{Device, Queue},
	format::Format,
};

const DEPTH_FORMAT: Format = Format::D32Sfloat;
// const ALT_DEPTH_FORMAT: Format = Format::X8_D24UnormPack32;
const COLOR_FORMAT: Format = Format::A2B10G10R10UnormPack32;
const POSITION_FORMAT: Format = Format::R32G32B32A32Sfloat;
const NORMAL_FORMAT: Format = Format::R32G32B32A32Sfloat;
const LIGHT_FORMAT: Format = Format::R32G32B32A32Sfloat;

pub(crate) struct DeferredPipelineDef;
impl PipelineDef for DeferredPipelineDef {
	fn make_context(device: &Arc<Device>, queue: &Arc<Queue>) -> (Box<dyn PipelineContext>, Box<dyn GpuFuture>) {
		let (pctx, future) = DeferredPipelineContext::new(device, queue);
		(Box::new(pctx), Box::new(future))
	}
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
struct Vert2D {
	pos: [f32; 2],
	texc: [f32; 2],
}
vulkano::impl_vertex!(Vert2D, pos, texc);

mod geom_vshader {
	vulkano_shaders::shader! { ty: "vertex", path: "src/pipelines/shaders/geom.glslv" }
}
mod geom_fshader {
	vulkano_shaders::shader! { ty: "fragment", path: "src/pipelines/shaders/geom.glslf" }
}
mod swap_vshader {
	vulkano_shaders::shader! { ty: "vertex", path: "src/pipelines/shaders/swap.glslv" }
}
mod swap_fshader {
	vulkano_shaders::shader! { ty: "fragment", path: "src/pipelines/shaders/swap.glslf" }
}
mod light_vshader {
	vulkano_shaders::shader! { ty: "vertex", path: "src/pipelines/shaders/light.glslv" }
}
mod light_fshader {
	vulkano_shaders::shader! { ty: "fragment", path: "src/pipelines/shaders/light.glslf" }
}
