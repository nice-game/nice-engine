use crate::Context;
use std::{os::raw::c_ulong, sync::Arc};
use vulkano::{
	device::DeviceOwned,
	format::Format,
	image::SwapchainImage,
	swapchain::{
		acquire_next_image, AcquireError, PresentMode, Surface as VkSurface, SurfaceCreationError, SurfaceTransform,
		Swapchain, SwapchainCreationError,
	},
	sync::{self, GpuFuture},
};

pub struct Surface<W = ()> {
	surface: Arc<VkSurface<W>>,
	swapchain: Arc<Swapchain<W>>,
	images: Vec<Arc<SwapchainImage<W>>>,
	prev_frame_end: Option<Box<dyn GpuFuture>>,
}
impl<W: 'static> Surface<W> {
	#[cfg(feature = "window")]
	pub fn from_vk(ctx: &mut Context, surface: Arc<VkSurface<W>>) -> Self {
		Self::new_inner(ctx, surface)
	}

	pub fn draw(&mut self, ctx: &Context) {
		let mut prev_frame_end = self.prev_frame_end.take().unwrap();
		prev_frame_end.cleanup_finished();

		let (image_num, acquire_future) = match acquire_next_image(self.swapchain.clone(), None) {
			Ok(r) => r,
			Err(AcquireError::OutOfDate) => {
				self.prev_frame_end = Some(Box::new(sync::now(ctx.device().clone())) as Box<_>);
				return;
			},
			Err(err) => panic!("{:?}", err),
		};

		let future = prev_frame_end
			.join(acquire_future)
			.then_swapchain_present(ctx.queue().clone(), self.swapchain.clone(), image_num)
			.then_signal_fence_and_flush();
		self.prev_frame_end = Some(match future {
			Ok(future) => Box::new(future) as Box<_>,
			Err(e) => {
				println!("{:?}", e);
				Box::new(sync::now(ctx.device().clone())) as Box<_>
			},
		});
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		let dimensions = self
			.surface
			.capabilities(self.swapchain.device().physical_device())
			.expect("failed to get surface capabilities")
			.current_extent
			.unwrap_or([width, height]);

		match self.swapchain.recreate_with_dimension(dimensions) {
			Ok((swapchain, images)) => {
				self.swapchain = swapchain;
				self.images = images;
			},
			// this normally happens when the window was resized after getting the surface capabilities, but before
			// recreating the swapchain. there should be another resize event on the next frame so we just ignore the
			// error here.
			Err(SwapchainCreationError::UnsupportedDimensions) => (),
			Err(err) => unreachable!(err),
		}
	}

	fn new_inner(ctx: &mut Context, surface: Arc<VkSurface<W>>) -> Self {
		let device = ctx.get_device_for_surface(&surface);
		let queue = device.queue();
		let caps = surface.capabilities(queue.device().physical_device()).expect("failed to get surface capabilities");

		let (swapchain, images) = Swapchain::new(
			queue.device().clone(),
			surface.clone(),
			caps.min_image_count,
			Format::B8G8R8A8Srgb,
			caps.current_extent.unwrap(),
			1,
			caps.supported_usage_flags,
			queue,
			SurfaceTransform::Identity,
			caps.supported_composite_alpha.iter().next().unwrap(),
			PresentMode::Fifo,
			true,
			None,
		)
		.expect("failed to create swapchain");

		let prev_frame_end = Some(Box::new(sync::now(device.device().clone())) as Box<dyn GpuFuture>);

		Self { surface, swapchain, images, prev_frame_end }
	}
}
impl Surface<()> {
	pub unsafe fn from_hwnd<T, U>(
		ctx: &mut Context,
		hinstance: *const T,
		hwnd: *const U,
	) -> Result<Self, SurfaceCreationError> {
		Ok(Self::new_inner(ctx, VkSurface::from_hwnd(ctx.instance.clone(), hinstance, hwnd, ())?))
	}

	pub unsafe fn from_xlib<D>(
		ctx: &mut Context,
		display: *const D,
		surface: c_ulong,
	) -> Result<Self, SurfaceCreationError> {
		Ok(Self::new_inner(ctx, VkSurface::from_xlib(ctx.instance.clone(), display, surface, ())?))
	}

	pub unsafe fn from_wayland<D, S>(
		ctx: &mut Context,
		display: *const D,
		surface: *const S,
	) -> Result<Self, SurfaceCreationError> {
		Ok(Self::new_inner(ctx, VkSurface::from_wayland(ctx.instance.clone(), display, surface, ())?))
	}
}
