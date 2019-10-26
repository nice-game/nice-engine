pub mod camera;
pub mod direct_light;
pub mod mesh;
pub mod mesh_data;
pub mod mesh_group;
pub mod pipelines;
pub mod resources;
pub mod surface;
pub mod texture;
pub mod threads;
pub mod transform;
#[cfg(feature = "window")]
pub mod window;

use crate::{
	pipelines::{deferred::DeferredPipelineDef, PipelineContext, PipelineDef},
	resources::Resources,
};
use log::info;
use maplit::hashset;
use std::{collections::HashSet, sync::Arc};
#[cfg(debug_assertions)]
use vulkano::instance::debug::DebugCallback;
use vulkano::{
	device::{Device, DeviceExtensions, Features, Queue},
	instance::{self, ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice},
};
pub use vulkano::{
	instance::{InstanceCreationError, Version},
	sync::GpuFuture,
};

/// Root struct for this library. Any windows that are created using the same context will share some resources.
pub struct Context {
	instance: Arc<Instance>,
	// `debug_callback` isn't used within rust code, but must stay in scope so the validation layers can use it
	#[allow(dead_code)]
	#[cfg(debug_assertions)]
	debug_callback: DebugCallback,
	device: Arc<Device>,
	queue: Arc<Queue>,
	pipeline_ctx: Box<dyn PipelineContext>,
	resources: Resources,
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
			#[cfg(debug_assertions)]
			ext_debug_utils: true,
			..InstanceExtensions::none()
		};

		let exts = match InstanceExtensions::supported_by_core() {
			Ok(supported) => supported.intersection(&exts),
			Err(_) => InstanceExtensions::none(),
		};

		#[cfg(debug_assertions)]
		let layers = hashset! {
			// "VK_LAYER_KHRONOS_validation".to_owned(),
			"VK_LAYER_LUNARG_monitor".to_owned(),
		};
		#[cfg(not(debug_assertions))]
		let layers = hashset! {
			"VK_LAYER_LUNARG_monitor".to_owned(),
		};

		let instance_layers =
			instance::layers_list().unwrap().map(|l| l.name().to_owned()).collect::<HashSet<String>>();
		let layers = instance_layers.intersection(&layers).map(|s| s as &str);

		let instance = Instance::new(Some(&app_info), &exts, layers)?;

		#[cfg(debug_assertions)]
		let debug_callback = DebugCallback::errors_and_warnings(&instance, |msg| {
			if msg.severity.error {
				log::error!("[{}]{}", msg.layer_prefix, msg.description);
			} else {
				log::warn!("[{}]{}", msg.layer_prefix, msg.description);
			}
		})
		.unwrap();

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

		let (pipeline_ctx, pipeline_ctx_future) = DeferredPipelineDef::make_context(&device, &queue);

		let (resources, resources_future) = Resources::new(queue.clone(), pipeline_ctx.layout_desc().clone());

		Ok((
			Arc::new(Self {
				instance,
				#[cfg(debug_assertions)]
				debug_callback,
				device,
				queue,
				pipeline_ctx,
				resources,
			}),
			pipeline_ctx_future.join(resources_future),
		))
	}

	pub fn resources(&self) -> &Resources {
		&self.resources
	}

	pub fn device(&self) -> &Arc<Device> {
		&self.device
	}

	pub fn queue(&self) -> &Arc<Queue> {
		&self.queue
	}

	fn pipeline_ctx(&self) -> &dyn PipelineContext {
		self.pipeline_ctx.as_ref()
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
