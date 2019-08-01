use crate::{camera::Camera, mesh::Mesh, mesh_data, vs3d, Context};
use std::{os::raw::c_ulong, sync::Arc};
use vulkano::{
	command_buffer::AutoCommandBufferBuilder,
	device::{Device, DeviceOwned, Queue},
	format::Format,
	framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
	image::{AttachmentImage, ImageViewAccess, SwapchainImage},
	pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract},
	swapchain::{
		acquire_next_image, AcquireError, PresentMode, Surface as VkSurface, SurfaceCreationError, SurfaceTransform,
		Swapchain, SwapchainCreationError,
	},
	sync::{self, GpuFuture},
};

pub const DEPTH_FORMAT: Format = Format::D16Unorm;
pub const SWAP_FORMAT: Format = Format::B8G8R8A8Srgb;

pub struct Surface<W: Send + Sync + 'static = ()> {
	device: Arc<Device>,
	queue: Arc<Queue>,
	surface: Arc<VkSurface<W>>,
	swapchain: Arc<Swapchain<W>>,
	pipeline_3d: Pipeline3D,
	prev_frame_end: Option<Box<dyn GpuFuture>>,
}
impl<W: Send + Sync + 'static> Surface<W> {
	#[cfg(feature = "window")]
	pub fn from_vk(ctx: &Arc<Context>, surface: Arc<VkSurface<W>>) -> Self {
		Self::new_inner(ctx, surface)
	}

	pub fn draw(&mut self, cam: &Camera, meshes: &[&Mesh]) {
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

		let clear_values = vec![[0.0, 0.0, 0.25, 1.0].into()];

		let make_pc = |mesh: &Mesh| vs3d::ty::PushConsts {
			cam_proj: cam.projection().into(),
			cam_pos: cam.transform().pos.into(),
			cam_rot: cam.transform().rot.into(),
			mesh_pos: mesh.transform().pos.into(),
			mesh_rot: mesh.transform().rot.into(),
			_dummy0: unsafe { std::mem::uninitialized() },
			_dummy1: unsafe { std::mem::uninitialized() },
		};

		let mut command_buffer =
			AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family())
				.unwrap()
				.begin_render_pass(self.pipeline_3d.depth_framebuffer.clone(), false, vec![1.0.into()])
				.unwrap();
		for mesh in meshes {
			let mesh_data = mesh.mesh_data().as_ref().unwrap();
			command_buffer = command_buffer
				.draw_indexed(
					self.pipeline_3d.depth_pipeline.clone(),
					&Default::default(),
					vec![mesh_data.vertices().clone()],
					mesh_data.indices().clone(),
					(),
					make_pc(mesh),
				)
				.unwrap();
		}

		command_buffer = command_buffer
			.end_render_pass()
			.unwrap()
			.begin_render_pass(self.pipeline_3d.framebuffers[image_num].clone(), false, clear_values)
			.unwrap();
		for mesh in meshes {
			let mesh_data = mesh.mesh_data().as_ref().unwrap();
			command_buffer = command_buffer
				.draw_indexed(
					self.pipeline_3d.pipeline.clone(),
					&Default::default(),
					vec![mesh_data.vertices().clone()],
					mesh_data.indices().clone(),
					mesh.texture_desc().clone(),
					make_pc(mesh),
				)
				.unwrap();
		}

		let command_buffer = command_buffer.end_render_pass().unwrap().build().unwrap();

		let future = prev_frame_end
			.join(acquire_future)
			.then_execute(self.queue.clone(), command_buffer)
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
				self.pipeline_3d.recreate(images, dimensions);
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

		let pipeline_3d = Pipeline3D::new(ctx.clone(), images, dimensions);
		let prev_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

		Self { device, queue, surface, swapchain, pipeline_3d, prev_frame_end }
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

fn create_framebuffers<T: ImageViewAccess + Send + Sync + 'static>(
	render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
	images: Vec<Arc<T>>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
	images
		.into_iter()
		.map(move |image| {
			Arc::new(Framebuffer::start(render_pass.clone()).add(image).unwrap().build().unwrap())
				as Arc<dyn FramebufferAbstract + Send + Sync>
		})
		.collect()
}

fn create_depth_pipeline(ctx: &Arc<Context>, dimensions: [u32; 2]) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
	let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
	Arc::new(
		GraphicsPipeline::start()
			.vertex_input_single_buffer::<mesh_data::Pntl_32F>()
			.vertex_shader(ctx.context_3d().depth_vs().main_entry_point(), ())
			.fragment_shader(ctx.context_3d().depth_fs().main_entry_point(), ())
			.triangle_list()
			.viewports(vec![Viewport { origin: [0.0, 0.0], dimensions, depth_range: 0.0..1.0 }])
			.render_pass(Subpass::from(ctx.context_3d().depth_pass().clone(), 0).unwrap())
			.build(ctx.device().clone())
			.unwrap(),
	)
}

fn create_pipeline_3d(ctx: &Arc<Context>, dimensions: [u32; 2]) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
	let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
	Arc::new(
		GraphicsPipeline::start()
			.vertex_input_single_buffer::<mesh_data::Pntl_32F>()
			.vertex_shader(ctx.context_3d().vertex_shader().main_entry_point(), ())
			.triangle_list()
			.viewports(vec![Viewport { origin: [0.0, 0.0], dimensions, depth_range: 0.0..1.0 }])
			.fragment_shader(ctx.context_3d().fragment_shader().main_entry_point(), ())
			.render_pass(Subpass::from(ctx.context_3d().render_pass().clone(), 0).unwrap())
			.build(ctx.device().clone())
			.unwrap(),
	)
}

struct Pipeline3D {
	ctx: Arc<Context>,
	depth_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	depth_framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
	pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}
impl Pipeline3D {
	fn new<T: ImageViewAccess + Send + Sync + 'static>(
		ctx: Arc<Context>,
		images: Vec<Arc<T>>,
		dimensions: [u32; 2],
	) -> Self {
		let depth_pipeline = create_depth_pipeline(&ctx, dimensions);
		let depth_image = Arc::new(AttachmentImage::new(ctx.device().clone(), dimensions, DEPTH_FORMAT).unwrap());
		let depth_framebuffer = Arc::new(
			Framebuffer::start(ctx.context_3d().depth_pass().clone())
				.add(depth_image.clone())
				.unwrap()
				.build()
				.unwrap(),
		);
		let pipeline = create_pipeline_3d(&ctx, dimensions);
		let framebuffers = create_framebuffers(ctx.context_3d().render_pass(), images);
		Pipeline3D { ctx, depth_pipeline, depth_framebuffer, pipeline, framebuffers }
	}

	fn recreate<T: ImageViewAccess + Send + Sync + 'static>(&mut self, images: Vec<Arc<T>>, dimensions: [u32; 2]) {
		self.pipeline = create_pipeline_3d(&self.ctx, dimensions);
		self.framebuffers = create_framebuffers(self.ctx.context_3d().render_pass(), images);
	}
}
