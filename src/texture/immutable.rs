use super::Texture;
use crate::Context;
use std::sync::Arc;
use vulkano::{
	buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
	command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
	device::Queue,
	format::{AcceptsPixels, Format, FormatDesc},
	image::{Dimensions, ImageCreationError, ImageLayout, ImageUsage, ImageViewAccess, ImmutableImage, MipmapsCount},
	sampler::Filter,
	sync::GpuFuture,
};

#[derive(Clone)]
pub struct ImmutableTexture {
	image: Arc<dyn ImageViewAccess + Send + Sync>,
}
impl ImmutableTexture {
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
		let buffer =
			CpuAccessibleBuffer::from_iter(ctx.queue().device().clone(), BufferUsage::transfer_source(), iter).unwrap();
		Self::from_buffer(ctx.queue().clone(), buffer, dimensions, format)
	}

	pub(crate) fn from_buffer<F, B, P>(
		queue: Arc<Queue>,
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
		let device = queue.device();
		let family = queue.family();

		let dimensions = Dimensions::Dim2d { width: dimensions[0], height: dimensions[1] };

		let (image, init) = ImmutableImage::uninitialized(
			device.clone(),
			dimensions,
			format,
			MipmapsCount::Log2,
			ImageUsage { transfer_destination: true, transfer_source: true, sampled: true, ..ImageUsage::none() },
			ImageLayout::ShaderReadOnlyOptimal,
			device.active_queue_families(),
		)
		.unwrap();

		let mut cmds = AutoCommandBufferBuilder::new(device.clone(), family).unwrap();
		cmds = cmds
			.copy_buffer_to_image_dimensions(
				buffer,
				init,
				[0, 0, 0],
				dimensions.width_height_depth(),
				0,
				dimensions.array_layers_with_cube(),
				0,
			)
			.unwrap();

		let image_dimensions = dimensions.to_image_dimensions();

		let mut last_mip_dimensions = image_dimensions;
		for mip in 1..image.mipmap_levels() {
			let mip_dimensions = image_dimensions.mipmap_dimensions(mip).unwrap();

			let source_bottom_right = [
				last_mip_dimensions.width() as i32,
				last_mip_dimensions.height() as i32,
				last_mip_dimensions.depth() as i32,
			];
			let destination_bottom_right =
				[mip_dimensions.width() as i32, mip_dimensions.height() as i32, mip_dimensions.depth() as i32];

			cmds = cmds
				.blit_image(
					image.clone(),
					[0; 3],
					source_bottom_right,
					0,
					mip - 1,
					image.clone(),
					[0; 3],
					destination_bottom_right,
					0,
					mip,
					1,
					Filter::Linear,
				)
				.unwrap();

			last_mip_dimensions = mip_dimensions;
		}
		let future = cmds.build().unwrap().execute(queue).unwrap();

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
