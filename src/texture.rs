use crate::Context;
use std::sync::Arc;
use vulkano::{
	device::Queue,
	format::{AcceptsPixels, Format, FormatDesc},
	image::{Dimensions, ImageCreationError, ImageViewAccess, ImmutableImage},
	buffer::{BufferAccess, TypedBufferAccess},
	sync::GpuFuture,
};

#[derive(Clone)]
pub struct Texture {
	image: Arc<dyn ImageViewAccess + Send + Sync>,
}
impl Texture {
	pub fn from_iter<F, P, I>(
		ctx: &Context,
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
		let (image, future) = ImmutableImage::from_iter(
			iter,
			Dimensions::Dim2d { width: dimensions[0], height: dimensions[1] },
			format,
			ctx.queue().clone(),
		)?;
		Ok((Self { image }, future))
	}

	pub(crate) fn from_buffer<F, B, P>(
		ctx: &Context,
		buffer: B,
		dimensions: [u32; 2],
		format: F,
	) -> Result<(Self, impl GpuFuture), ImageCreationError>
	where
		F: FormatDesc + AcceptsPixels<P> + 'static + Send + Sync,
		B: BufferAccess + TypedBufferAccess<Content = [P]> + 'static + Clone + Send + Sync,
		P: Send + Sync + Clone + 'static,
		Format: AcceptsPixels<P>,
	{
		let (image, future) = ImmutableImage::from_buffer(
		    buffer,
		    Dimensions::Dim2d { width: dimensions[0], height: dimensions[1] },
		    format,
		    ctx.queue().clone(),
		)?;
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
		let (image, future) = ImmutableImage::from_iter(
			iter,
			Dimensions::Dim2d { width: dimensions[0], height: dimensions[1] },
			format,
			queue,
		)?;
		Ok((Self { image }, future))
	}

	pub(crate) fn image(&self) -> &Arc<dyn ImageViewAccess + Send + Sync> {
		&self.image
	}
}
