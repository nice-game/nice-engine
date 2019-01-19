use crate::Context;
use std::{ os::raw::c_ulong, sync::Arc };
use vulkano::{
	device::DeviceOwned,
	format::Format,
	image::SwapchainImage,
	swapchain::{
		PresentMode,
		Surface as VkSurface,
		SurfaceCreationError,
		SurfaceTransform,
		Swapchain,
		SwapchainCreationError
	},
};

pub struct Surface {
	surface: Arc<VkSurface<()>>,
	swapchain: Arc<Swapchain<()>>,
	images: Vec<Arc<SwapchainImage<()>>>,
}
impl Surface {
	pub unsafe fn from_hwnd<T, U>(ctx: &mut Context, hinstance: *const T, hwnd: *const U) -> Result<Self, SurfaceCreationError> {
		Ok(Self::new_inner(ctx, VkSurface::from_hwnd(ctx.instance.clone(), hinstance, hwnd, ())?))
	}

	pub unsafe fn from_xlib<D>(ctx: &mut Context, display: *const D, surface: c_ulong) -> Result<Self, SurfaceCreationError> {
		Ok(Self::new_inner(ctx, VkSurface::from_xlib(ctx.instance.clone(), display, surface, ())?))
	}

	pub unsafe fn from_wayland<D, S>(ctx: &mut Context, display: *const D, surface: *const S) -> Result<Self, SurfaceCreationError> {
		Ok(Self::new_inner(ctx, VkSurface::from_wayland(ctx.instance.clone(), display, surface, ())?))
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		let dimensions = self.surface.capabilities(self.swapchain.device().physical_device())
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

	fn new_inner(ctx: &mut Context, surface: Arc<VkSurface<()>>) -> Self {
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
			None
		).expect("failed to create swapchain");

		Self { surface: surface, swapchain: swapchain, images: images }
	}
}
