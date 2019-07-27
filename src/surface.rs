use crate::{mesh_batch, Context};
use std::{os::raw::c_ulong, sync::Arc};
use vulkano::{
	device::DeviceOwned,
	format::Format,
	framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
	image::{ImageViewAccess, SwapchainImage},
	pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract},
	swapchain::{
		acquire_next_image, AcquireError, PresentMode, Surface as VkSurface, SurfaceCreationError, SurfaceTransform,
		Swapchain, SwapchainCreationError,
	},
	sync::{self, GpuFuture},
};

pub const SWAP_FORMAT: Format = Format::B8G8R8A8Srgb;

pub struct Surface<W: Send + Sync + 'static = ()> {
	surface: Arc<VkSurface<W>>,
	swapchain: Arc<Swapchain<W>>,
	images: Vec<Arc<SwapchainImage<W>>>,
	framebuffers_3d: Pipeline3D,
	prev_frame_end: Option<Box<dyn GpuFuture>>,
}
impl<W: Send + Sync + 'static> Surface<W> {
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
				let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
				self.framebuffers_3d.recreate(&images, dimensions);
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

	pub fn swapchain(&self) -> &Arc<Swapchain<W>> {
		&self.swapchain
	}

	fn new_inner(ctx: &mut Context, surface: Arc<VkSurface<W>>) -> Self {
		let device = ctx.device();
		let queue = ctx.queue();
		let caps = surface.capabilities(device.physical_device()).expect("failed to get surface capabilities");

		let dims = caps.current_extent.unwrap();

		let (swapchain, images) = Swapchain::new(
			device.clone(),
			surface.clone(),
			caps.min_image_count,
			SWAP_FORMAT,
			dims,
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

		let dimensions = [dims[0] as f32, dims[1] as f32];
		let framebuffers_3d = Pipeline3D::new(ctx.render_pass_3d().clone(), &images, dimensions);
		let prev_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

		Self { surface, swapchain, images, framebuffers_3d, prev_frame_end }
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

fn create_framebuffers<T: ImageViewAccess + Send + Sync + 'static>(
	render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
	images: &Vec<Arc<T>>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
	images
		.iter()
		.map(move |image| {
			Arc::new(Framebuffer::start(render_pass.clone()).add(image.clone()).unwrap().build().unwrap())
				as Arc<dyn FramebufferAbstract + Send + Sync>
		})
		.collect()
}

fn create_pipeline_3d(
	render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
	dimensions: [f32; 2],
) -> Arc<dyn GraphicsPipelineAbstract> {
	let device = render_pass.device();
	let vs = mesh_batch::vs::Shader::load(device.clone()).unwrap();
	let fs = mesh_batch::fs::Shader::load(device.clone()).unwrap();

	Arc::new(
		GraphicsPipeline::start()
			.vertex_input_single_buffer::<mesh_batch::Vertex>()
			.vertex_shader(vs.main_entry_point(), ())
			.triangle_list()
			.viewports(vec![Viewport { origin: [0.0, 0.0], dimensions, depth_range: 0.0..1.0 }])
			.fragment_shader(fs.main_entry_point(), ())
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
			.build(device.clone())
			.unwrap(),
	)
}

struct Pipeline3D {
	render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	pipeline: Arc<dyn GraphicsPipelineAbstract>,
	framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}
impl Pipeline3D {
	fn new<T: ImageViewAccess + Send + Sync + 'static>(
		render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
		images: &Vec<Arc<T>>,
		dimensions: [f32; 2],
	) -> Self {
		let pipeline = create_pipeline_3d(&render_pass, dimensions);
		let framebuffers = create_framebuffers(&render_pass, images);
		Pipeline3D { render_pass, pipeline, framebuffers }
	}

	fn recreate<T: ImageViewAccess + Send + Sync + 'static>(&mut self, images: &Vec<Arc<T>>, dimensions: [f32; 2]) {
		self.pipeline = create_pipeline_3d(&self.render_pass, dimensions);
		self.framebuffers = create_framebuffers(&self.render_pass, images);
	}
}
