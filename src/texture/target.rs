use super::Texture;
use std::sync::Arc;
use vulkano::{
	command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
	device::Queue,
	format::FormatDesc,
	image::{AttachmentImage, ImageCreationError, ImageUsage, ImageViewAccess},
	sync::GpuFuture,
};

#[derive(Clone)]
pub struct TargetTexture {
	image: Arc<dyn ImageViewAccess + Send + Sync>,
}
impl TargetTexture {
	pub fn new<F>(
		queue: Arc<Queue>,
		dimensions: [u32; 2],
		format: F,
	) -> Result<(Self, impl GpuFuture), ImageCreationError>
	where
		F: FormatDesc + Send + Sync + 'static,
	{
		let device = queue.device();

		let usage = ImageUsage { transfer_destination: true, sampled: true, ..ImageUsage::none() };
		let image = AttachmentImage::with_usage(device.clone(), dimensions, format, usage)?;

		let future = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
			.unwrap()
			.clear_color_image(image.clone(), [1.0; 4].into())
			.unwrap()
			.build()
			.unwrap()
			.execute(queue)
			.unwrap();

		Ok((Self { image }, future))
	}
}
impl Texture for TargetTexture {
	fn image(&self) -> &Arc<dyn ImageViewAccess + Send + Sync> {
		&self.image
	}
}
