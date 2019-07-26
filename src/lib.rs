pub mod camera;
pub mod device;
pub mod mesh_batch;
pub mod mesh_data;
pub mod surface;
#[cfg(feature = "window")]
pub mod window;

pub use vulkano::{
	instance::{InstanceCreationError, Version},
	sync::GpuFuture,
};

use self::device::DeviceCtx;
use crate::surface::SWAP_FORMAT;
use log::info;
use std::sync::Arc;
use vulkano::{
	command_buffer::DynamicState,
	device::{Device, DeviceExtensions, Features, Queue},
	framebuffer::{RenderPassAbstract, Subpass},
	instance::{ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice},
	pipeline::{GraphicsPipeline, GraphicsPipelineAbstract},
	swapchain::Surface,
};

/// Root struct for this library. Any windows that are created using the same context will share some resources.
pub struct Context {
	instance: Arc<Instance>,
	device: Arc<Device>,
	queue: Arc<Queue>,
	pipeline_3d: Pipeline3D,
}
impl Context {
	pub fn new(name: Option<&str>, version: Option<Version>) -> Result<Self, InstanceCreationError> {
		let app_info = ApplicationInfo {
			application_name: name.map(|x| x.into()),
			application_version: version,
			engine_name: Some("nIce Game".into()),
			engine_version: Some(Version {
				major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
				minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
				patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
			}),
		};

		let exts = InstanceExtensions {
			khr_surface: true,
			khr_xlib_surface: true,
			khr_xcb_surface: true,
			khr_wayland_surface: true,
			khr_android_surface: true,
			khr_win32_surface: true,
			mvk_ios_surface: true,
			mvk_macos_surface: true,
			..InstanceExtensions::none()
		};

		let exts = match InstanceExtensions::supported_by_core() {
			Ok(supported) => supported.intersection(&exts),
			Err(_) => InstanceExtensions::none(),
		};

		let instance = Instance::new(Some(&app_info), &exts, None)?;

		let pdevice = PhysicalDevice::enumerate(&instance).next().expect("no device available");
		info!("Using device: {} ({:?})", pdevice.name(), pdevice.ty());

		let qfam =
			pdevice.queue_families().find(|&q| q.supports_graphics()).expect("failed to find a graphical queue family");

		let (device, mut queues) = Device::new(
			pdevice,
			&Features::none(),
			&DeviceExtensions { khr_swapchain: true, ..DeviceExtensions::none() },
			[(qfam, 1.0)].iter().cloned(),
		)
		.expect("failed to create device");
		let queue = queues.next().unwrap();

		let pipeline_3d = Pipeline3D::new(&device);

		Ok(Self { instance, device, queue, pipeline_3d })
	}

	fn device(&self) -> &Arc<Device> {
		&self.device
	}

	fn pipeline_3d(&self) -> &Pipeline3D {
		&self.pipeline_3d
	}

	fn queue(&self) -> &Arc<Queue> {
		&self.queue
	}
}

struct Pipeline3D {
	render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	pipeline: Arc<dyn GraphicsPipelineAbstract>,
}
impl Pipeline3D {
	pub fn new(device: &Arc<Device>) -> Self {
		let render_pass = Arc::new(
			vulkano::single_pass_renderpass!(
				device.clone(),
				attachments: {
					color: { load: Clear, store: Store, format: SWAP_FORMAT, samples: 1, }
				},
				pass: { color: [color], depth_stencil: {} }
			)
			.unwrap(),
		);

		let vs = mesh_batch::vs::Shader::load(device.clone()).unwrap();
		let fs = mesh_batch::fs::Shader::load(device.clone()).unwrap();

		let pipeline = Arc::new(
			GraphicsPipeline::start()
				.vertex_input_single_buffer::<mesh_batch::Vertex>()
				.vertex_shader(vs.main_entry_point(), ())
				.triangle_list()
				.viewports_dynamic_scissors_irrelevant(1)
				.fragment_shader(fs.main_entry_point(), ())
				.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
				.build(device.clone())
				.unwrap(),
		);

		Self { render_pass, pipeline }
	}

	pub fn render_pass(&self) -> &Arc<dyn RenderPassAbstract + Send + Sync> {
		&self.render_pass
	}

	pub fn pipeline(&self) -> &Arc<dyn GraphicsPipelineAbstract> {
		&self.pipeline
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
