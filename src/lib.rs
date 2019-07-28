pub mod camera;
pub mod mesh;
pub mod mesh_data;
pub mod surface;
pub mod transform;
#[cfg(feature = "window")]
pub mod window;

pub use vulkano::{
	instance::{InstanceCreationError, Version},
	sync::GpuFuture,
};

use crate::surface::SWAP_FORMAT;
use log::info;
use std::sync::Arc;
use vulkano::{
	device::{Device, DeviceExtensions, Features, Queue},
	framebuffer::RenderPassAbstract,
	instance::{ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice},
};

/// Root struct for this library. Any windows that are created using the same context will share some resources.
pub struct Context {
	instance: Arc<Instance>,
	device: Arc<Device>,
	queue: Arc<Queue>,
	render_pass_3d: Arc<dyn RenderPassAbstract + Send + Sync>,
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

		let render_pass_3d = Arc::new(
			vulkano::single_pass_renderpass!(
				device.clone(),
				attachments: {
					color: { load: Clear, store: Store, format: SWAP_FORMAT, samples: 1, }
				},
				pass: { color: [color], depth_stencil: {} }
			)
			.unwrap(),
		);

		Ok(Self { instance, device, queue, render_pass_3d })
	}

	fn device(&self) -> &Arc<Device> {
		&self.device
	}

	fn render_pass_3d(&self) -> &Arc<dyn RenderPassAbstract + Send + Sync> {
		&self.render_pass_3d
	}

	fn queue(&self) -> &Arc<Queue> {
		&self.queue
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
