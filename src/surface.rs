use crate::{camera::Camera, pipelines::Pipeline, world::World, Context};
use std::{os::raw::c_ulong, sync::Arc};
use vulkano::{
	device::{Device, Queue},
	format::Format,
	swapchain::{
		acquire_next_image, AcquireError, PresentMode, Surface as VkSurface, SurfaceCreationError, SurfaceTransform,
		Swapchain, SwapchainCreationError,
	},
	sync::{self, GpuFuture},
};

pub const SWAP_FORMAT: Format = Format::B8G8R8A8Srgb;

pub struct Surface<W: Send + Sync + 'static = ()> {
	device: Arc<Device>,
	queue: Arc<Queue>,
	surface: Arc<VkSurface<W>>,
	swapchain: Arc<Swapchain<W>>,
	pipeline: Box<dyn Pipeline>,
	prev_frame_end: Option<Box<dyn GpuFuture>>,
	world: Arc<World>,
}
impl<W: Send + Sync + 'static> Surface<W> {
	#[cfg(feature = "window")]
	pub fn from_vk(ctx: &Arc<Context>, surface: Arc<VkSurface<W>>) -> Self {
		if !surface.is_supported(ctx.queue().family()).unwrap() {
			panic!("Vulkan surface not supported");
		}
		Self::new_inner(ctx, surface)
	}

	pub fn draw(&mut self, cam: &Camera) {
		let mut prev_frame_end = self.prev_frame_end.take().unwrap();
		prev_frame_end.cleanup_finished();

		let (image_num, acquire_future) = match acquire_next_image(self.swapchain.clone(), None) {
			Ok(r) => r,
			Err(AcquireError::OutOfDate) => {
				self.prev_frame_end = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
				return;
			},
			Err(err) => panic!("{:?}", err),
		};

		let mut meshes = self.world.meshes().lock().unwrap();
		for mesh in &mut *meshes {
			mesh.refresh();
		}

		let lights = self.world.lights().lock().unwrap();

		let future = prev_frame_end
			.join(acquire_future)
			.then_execute(
				self.queue.clone(),
				self.pipeline.draw(image_num, self.queue.family(), cam, &*meshes, &*lights),
			)
			.unwrap()
			.then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
			.then_signal_fence_and_flush();
		self.prev_frame_end = Some(match future {
			Ok(future) => Box::new(future) as Box<_>,
			Err(e) => {
				println!("{:?}", e);
				Box::new(sync::now(self.device.clone())) as Box<_>
			},
		});
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		let dimensions = self
			.surface
			.capabilities(self.device.physical_device())
			.expect("failed to get surface capabilities")
			.current_extent
			.unwrap_or([width, height]);

		match self.swapchain.recreate_with_dimension(dimensions) {
			Ok((swapchain, images)) => {
				self.pipeline.resize(images.into_iter().map(|i| i as _).collect(), dimensions);
				self.swapchain = swapchain;
			},
			// this normally happens when the window was resized after getting the surface capabilities, but before
			// recreating the swapchain. there should be another resize event on the next frame so we just ignore the
			// error here.
			Err(SwapchainCreationError::UnsupportedDimensions) => (),
			Err(err) => unreachable!(err),
		}
	}

	pub fn swapchain(&self) -> &Arc<Swapchain<W>> {
		&self.swapchain
	}

	fn new_inner(ctx: &Arc<Context>, surface: Arc<VkSurface<W>>) -> Self {
		let device = ctx.device().clone();
		let queue = ctx.queue().clone();
		let caps = surface.capabilities(device.physical_device()).expect("failed to get surface capabilities");

		let dimensions = caps.current_extent.unwrap();

		let (swapchain, images) = Swapchain::new(
			device.clone(),
			surface.clone(),
			caps.min_image_count,
			SWAP_FORMAT,
			dimensions,
			1,
			caps.supported_usage_flags,
			&queue,
			SurfaceTransform::Identity,
			caps.supported_composite_alpha.iter().next().unwrap(),
			PresentMode::Fifo,
			true,
			None,
		)
		.expect("failed to create swapchain");

		let pipeline = ctx.pipeline_ctx().make_pipeline(images.into_iter().map(|i| i as _).collect(), dimensions);
		let prev_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

		let world = ctx.world().clone();

		Self { device, queue, surface, swapchain, pipeline, prev_frame_end, world }
	}
}
impl Surface<()> {
	pub unsafe fn from_hwnd<T, U>(
		ctx: &Arc<Context>,
		hinstance: *const T,
		hwnd: *const U,
	) -> Result<Self, SurfaceCreationError> {
		Ok(Self::new_inner(ctx, VkSurface::from_hwnd(ctx.instance.clone(), hinstance, hwnd, ())?))
	}

	pub unsafe fn from_xlib<D>(
		ctx: &Arc<Context>,
		display: *const D,
		surface: c_ulong,
	) -> Result<Self, SurfaceCreationError> {
		Ok(Self::new_inner(ctx, VkSurface::from_xlib(ctx.instance.clone(), display, surface, ())?))
	}

	pub unsafe fn from_wayland<D, S>(
		ctx: &Arc<Context>,
		display: *const D,
		surface: *const S,
	) -> Result<Self, SurfaceCreationError> {
		Ok(Self::new_inner(ctx, VkSurface::from_wayland(ctx.instance.clone(), display, surface, ())?))
	}
}
