mod mipmaps_command_buffer;

use self::mipmaps_command_buffer::MipmapsCommandBuffer;
use super::Texture;
use crate::Context;
use std::sync::Arc;
use vulkano::{
	buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
	command_buffer::CommandBuffer,
	device::Queue,
	format::{AcceptsPixels, Format, FormatDesc},
	image::{Dimensions, ImageCreationError, ImageLayout, ImageUsage, ImageViewAccess, ImmutableImage, MipmapsCount},
	sync::GpuFuture,
};

#[derive(Clone)]
pub struct ImmutableTexture {
	image: Arc<dyn ImageViewAccess + Send + Sync>,
}
impl ImmutableTexture {
	pub(crate) fn from_buffer<F, B, P>(
		queue: Arc<Queue>,
		buffer: B,
		dimensions: [u32; 2],
		format: F,
	) -> Result<(Self, impl GpuFuture), ImageCreationError>
	where
		F: FormatDesc + AcceptsPixels<P> + Send + Sync + 'static,
		B: BufferAccess + TypedBufferAccess<Content = [P]> + Clone + Send + Sync + 'static,
		P: Send + Sync + Clone + 'static,
		Format: AcceptsPixels<P>,
	{
		let device = queue.device();

		let dimensions = Dimensions::Dim2d { width: dimensions[0], height: dimensions[1] };

		let (image, init) = ImmutableImage::uninitialized(
			device.clone(),
			dimensions,
			format,
			MipmapsCount::Log2,
			ImageUsage { transfer_destination: true, transfer_source: true, sampled: true, ..ImageUsage::none() },
			ImageLayout::ShaderReadOnlyOptimal,
			device.active_queue_families(),
		)?;

		let future = MipmapsCommandBuffer::new(device.clone(), queue.family(), buffer, init).execute(queue).unwrap();

		Ok((Self { image }, future))
	}

	pub(crate) fn from_iter_vk<F, P, I>(
		queue: Arc<Queue>,
		iter: I,
		dimensions: [u32; 2],
		format: F,
	) -> Result<(Self, impl GpuFuture), ImageCreationError>
	where
		P: Send + Sync + Clone + 'static,
		F: FormatDesc + AcceptsPixels<P> + 'static + Send + Sync,
		I: ExactSizeIterator<Item = P>,
		Format: AcceptsPixels<P>,
	{
		let buffer =
			CpuAccessibleBuffer::from_iter(queue.device().clone(), BufferUsage::transfer_source(), iter).unwrap();
		Self::from_buffer(queue, buffer, dimensions, format)
	}
}
impl Texture for ImmutableTexture {
	fn image(&self) -> &Arc<dyn ImageViewAccess + Send + Sync> {
		&self.image
	}
}
