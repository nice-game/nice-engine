use crate::Context;
use std::sync::Arc;
use vulkano::{
	format::{AcceptsPixels, Format, FormatDesc},
	image::{Dimensions, ImageCreationError, ImageViewAccess, ImmutableImage},
	sync::GpuFuture,
};

pub struct Texture {
	image: Arc<dyn ImageViewAccess + Send + Sync>,
}
impl Texture {
	pub fn from_iter<F, P, I>(
		ctx: &Context,
		iter: I,
		dimensions: Dimensions,
		format: F,
	) -> Result<(Self, impl GpuFuture), ImageCreationError>
	where
		P: Send + Sync + Clone + 'static,
		F: FormatDesc + AcceptsPixels<P> + 'static + Send + Sync,
		I: ExactSizeIterator<Item = P>,
		Format: AcceptsPixels<P>,
	{
		let (image, future) = ImmutableImage::from_iter(iter, dimensions, format, ctx.queue().clone())?;

		Ok((Self { image }, future))
	}
}
