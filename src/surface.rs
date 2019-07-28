use crate::{camera::Camera, mesh_data, Context};
use std::{os::raw::c_ulong, sync::Arc};
use vulkano::{
	command_buffer::AutoCommandBufferBuilder,
	device::{Device, DeviceOwned, Queue},
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
	device: Arc<Device>,
	queue: Arc<Queue>,
	surface: Arc<VkSurface<W>>,
	swapchain: Arc<Swapchain<W>>,
	images: Vec<Arc<SwapchainImage<W>>>,
	pipeline_3d: Pipeline3D,
	prev_frame_end: Option<Box<dyn GpuFuture>>,
}
impl<W: Send + Sync + 'static> Surface<W> {
	#[cfg(feature = "window")]
	pub fn from_vk(ctx: &mut Context, surface: Arc<VkSurface<W>>) -> Self {
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

		let clear_values = vec![[0.0, 0.0, 0.25, 1.0].into()];

		let mut command_buffer =
			AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family())
				.unwrap()
				.begin_render_pass(self.pipeline_3d.framebuffers[image_num].clone(), false, clear_values)
				.unwrap();
		for mesh in &*cam.mesh_batch().unwrap().meshes().lock().unwrap() {
			let verts = mesh.mesh_data().unwrap().vertices().clone();
			let pc = vs3d::ty::PushConsts {
				cam_proj: cam.projection().into(),
				cam_pos: cam.transform().pos.into(),
				cam_rot: cam.transform().rot.into(),
				mesh_pos: mesh.transform().pos.into(),
				mesh_rot: mesh.transform().rot.into(),
				_dummy0: unsafe { std::mem::uninitialized() },
				_dummy1: unsafe { std::mem::uninitialized() },
			};
			command_buffer = command_buffer
				.draw(self.pipeline_3d.pipeline.clone(), &Default::default(), vec![verts], (), pc)
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
			.capabilities(self.swapchain.device().physical_device())
			.expect("failed to get surface capabilities")
			.current_extent
			.unwrap_or([width, height]);

		match self.swapchain.recreate_with_dimension(dimensions) {
			Ok((swapchain, images)) => {
				let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
				self.pipeline_3d.recreate(&images, dimensions);
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
		let device = ctx.device().clone();
		let queue = ctx.queue().clone();
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
			&queue,
			SurfaceTransform::Identity,
			caps.supported_composite_alpha.iter().next().unwrap(),
			PresentMode::Fifo,
			true,
			None,
		)
		.expect("failed to create swapchain");

		let dimensions = [dims[0] as f32, dims[1] as f32];
		let pipeline_3d = Pipeline3D::new(ctx.render_pass_3d().clone(), &images, dimensions);
		let prev_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

		Self { device, queue, surface, swapchain, images, pipeline_3d, prev_frame_end }
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
) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
	let device = render_pass.device();
	let vs = vs3d::Shader::load(device.clone()).unwrap();
	let fs = fs3d::Shader::load(device.clone()).unwrap();

	Arc::new(
		GraphicsPipeline::start()
			.vertex_input_single_buffer::<mesh_data::Pntl_32F>()
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
	pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
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

pub(crate) mod vs3d {
	vulkano_shaders::shader! {
		ty: "vertex",
		src: "
#version 450
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 nor;
layout(location = 2) in vec2 tex;
layout(location = 3) in vec2 lmap;

layout(push_constant) uniform PushConsts {
	vec4 cam_proj;
	vec3 cam_pos;
	vec4 cam_rot;
	vec3 mesh_pos;
	vec4 mesh_rot;
} pc;

vec4 perspective(vec4 proj, vec3 pos) {
	return vec4(pos.xy * proj.xy, pos.z * proj.z + proj.w, -pos.z);
}

vec4 quat_inv(vec4 quat) {
	return vec4(-quat.xyz, quat.w) / dot(quat, quat);
}

vec3 quat_mul(vec4 quat, vec3 vec) {
	return cross(quat.xyz, cross(quat.xyz, vec) + vec * quat.w) * 2.0 + vec;
}

void main() {
	// stupid math library puts w first, so we flip it here
	vec4 cam_rot = pc.cam_rot.yzwx;
	vec4 mesh_rot = pc.mesh_rot.yzwx;

	vec3 pos_ws = quat_mul(mesh_rot, pos) + pc.mesh_pos;
	vec3 pos_cs = quat_mul(quat_inv(cam_rot), pos_ws - pc.cam_pos);
	gl_Position = perspective(pc.cam_proj, pos_cs);
}"
	}
}

pub(crate) mod fs3d {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450
layout(location = 0) out vec4 f_color;
void main() {
	f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"
	}
}
