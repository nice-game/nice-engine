use super::Texture;
use std::sync::Arc;
use vulkano::{
	device::Device,
	format::FormatDesc,
	image::{AttachmentImage, ImageCreationError, ImageViewAccess},
};

#[derive(Clone)]
pub struct TargetTexture {
	image: Arc<dyn ImageViewAccess + Send + Sync>,
}
impl TargetTexture {
	pub fn new<F>(device: Arc<Device>, dimensions: [u32; 2], format: F) -> Result<Self, ImageCreationError>
	where
		F: FormatDesc + Send + Sync + 'static,
	{
		let image = AttachmentImage::sampled(device, dimensions, format)?;
		Ok(Self { image })
	}
}
impl Texture for TargetTexture {
	fn image(&self) -> &Arc<dyn ImageViewAccess + Send + Sync> {
		&self.image
	}
}
