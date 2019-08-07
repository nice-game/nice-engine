pub mod camera;
pub mod mesh;
pub mod mesh_data;
pub mod pipelines;
pub mod resources;
pub mod surface;
pub mod texture;
pub mod transform;
#[cfg(feature = "window")]
pub mod window;
mod threads;

use crate::{
	pipelines::{deferred::DeferredPipelineDef, forward::ForwardPipelineDef, PipelineContext, PipelineDef},
	resources::Resources,
};
use log::info;
use std::sync::{Arc, Mutex};
use vulkano::{
	device::{Device, DeviceExtensions, Features, Queue},
	instance::{ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice},
};
pub use vulkano::{
	instance::{InstanceCreationError, Version},
	sync::GpuFuture,
};

/// Root struct for this library. Any windows that are created using the same context will share some resources.
pub struct Context {
	instance: Arc<Instance>,
	device: Arc<Device>,
	queue: Arc<Queue>,
	pipeline_ctxs: Vec<Box<dyn PipelineContext>>,
	active_pipeline: usize,
	resources: Mutex<Resources>,
}
impl Context {
	pub fn new(
		name: Option<&str>,
		version: Option<Version>,
	) -> Result<(Arc<Self>, impl GpuFuture), InstanceCreationError> {
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

		let features = Features { sampler_anisotropy: true, ..Features::none() };
		let pdevice = PhysicalDevice::enumerate(&instance)
			.max_by_key(|pd| pd.supported_features().superset_of(&features))
			.unwrap();
		info!("Using device: {} ({:?})", pdevice.name(), pdevice.ty());

		let features = pdevice.supported_features().intersection(&features);
		let qfam =
			pdevice.queue_families().find(|&q| q.supports_graphics()).expect("failed to find a graphical queue family");
		let (device, mut queues) = Device::new(
			pdevice,
			&features,
			&DeviceExtensions { khr_swapchain: true, ..DeviceExtensions::none() },
			[(qfam, 1.0)].iter().cloned(),
		)
		.expect("failed to create device");
		let queue = queues.next().unwrap();

		let (deferred_def, deferred_def_future) = DeferredPipelineDef::make_context(&device, &queue);
		let (forward_def, forward_def_future) = ForwardPipelineDef::make_context(&device, &queue);
		let pipeline_ctxs = vec![deferred_def, forward_def];
		let active_pipeline = 0;

		let (resources, resources_future) = Resources::new(queue.clone(), pipeline_ctxs[0].layout_desc().clone());
		let resources = Mutex::new(resources);

		Ok((
			Arc::new(Self { instance, device, queue, pipeline_ctxs, active_pipeline, resources }),
			deferred_def_future.join(forward_def_future).join(resources_future),
		))
	}

	pub fn resources(&self) -> &Mutex<Resources> {
		&self.resources
	}

	fn device(&self) -> &Arc<Device> {
		&self.device
	}

	fn queue(&self) -> &Arc<Queue> {
		&self.queue
	}

	fn pipeline_ctx(&self) -> &dyn PipelineContext {
		self.pipeline_ctxs[self.active_pipeline].as_ref()
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
