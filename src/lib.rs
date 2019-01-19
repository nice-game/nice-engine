pub mod device;
pub mod surface;

pub use vulkano::instance::{ InstanceCreationError, Version };

use self::device::DeviceCtx;
use log::info;
use std::sync::Arc;
use vulkano::{
	device::{ Device, DeviceExtensions, Features },
	instance::{ ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice },
	swapchain::Surface,
};

/// Root struct for this library. Any windows that are created using the same context will share some resources.
pub struct Context {
	instance: Arc<Instance>,
	devices: Vec<Arc<DeviceCtx>>,
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

		Ok(Self { instance: Instance::new(Some(&app_info), &exts, None)?, devices: vec![] })
	}

	fn get_device_for_surface<T>(&mut self, surface: &Surface<T>) -> Arc<DeviceCtx> {
		for device in &self.devices {
			let qfam = device.queue().family();
			if qfam.supports_graphics() && surface.is_supported(qfam).unwrap() {
				return device.clone();
			}
		}

		let pdevice = PhysicalDevice::enumerate(&self.instance).next().expect("no device available");
		info!("Using device: {} ({:?})", pdevice.name(), pdevice.ty());

		let qfam = pdevice.queue_families()
			.find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap())
			.expect("failed to find a graphical queue family");

		let (device, mut queues) = Device::new(
			pdevice,
			&Features::none(),
			&DeviceExtensions { khr_swapchain: true, ..DeviceExtensions::none() },
			[(qfam, 1.0)].iter().cloned()
		).expect("failed to create device");
		let queue = queues.next().unwrap();

		let ret = DeviceCtx::new(device, queue);
		self.devices.push(ret.clone());
		ret
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
