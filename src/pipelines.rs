pub mod deferred;
pub mod forward;

use crate::{camera::Camera, mesh::Mesh};
use std::sync::Arc;
use vulkano::{
	command_buffer::AutoCommandBuffer,
	descriptor::PipelineLayoutAbstract,
	device::{Device, Queue},
	image::ImageViewAccess,
	instance::QueueFamily,
	sync::GpuFuture,
};

pub trait PipelineDef {
	fn make_context(device: &Arc<Device>, queue: &Arc<Queue>) -> (Box<dyn PipelineContext>, Box<dyn GpuFuture>);
}

pub trait PipelineContext {
	fn make_pipeline(
		&self,
		images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
		dimensions: [u32; 2],
	) -> Box<dyn Pipeline>;
	fn layout_desc(&self) -> &Arc<dyn PipelineLayoutAbstract + Send + Sync>;
}

pub trait Pipeline {
	fn draw(&self, image_num: usize, qfam: QueueFamily, cam: &Camera, meshes: &[Mesh]) -> AutoCommandBuffer;
	fn resize(&mut self, images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>, dimensions: [u32; 2]);
}
