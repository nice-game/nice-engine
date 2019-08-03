pub mod camera;
pub mod mesh;
pub mod mesh_data;
pub mod pipelines;
pub mod surface;
pub mod texture;
pub mod transform;
#[cfg(feature = "window")]
pub mod window;

use crate::pipelines::{forward::ForwardPipelineDef, PipelineContext, PipelineDef};
use log::info;
use std::sync::Arc;
use vulkano::{
	device::{Device, DeviceExtensions, Features, Queue},
	format::Format,
	image::Dimensions,
	instance::{ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice},
	sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
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
	sampler: Arc<Sampler>,
	pipeline_ctxs: Vec<Box<dyn PipelineContext>>,
	active_pipeline: usize,
	white_pixel: texture::Texture,
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

		let sampler = Sampler::new(
			device.clone(),
			Filter::Linear,
			Filter::Linear,
			MipmapMode::Nearest,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			0.0,
			1.0,
			0.0,
			0.0,
		)
		.unwrap();

		let pipeline_ctxs = vec![ForwardPipelineDef::make_context(&device)];
		let active_pipeline = 0;

		let (white_pixel, white_pixel_future) = texture::Texture::from_iter_vk(
			queue.clone(),
			vec![[0u8, 0, 255, 255], [0, 0, 0, 255], [0, 0, 0, 255], [0, 0, 255, 255]].into_iter(),
			Dimensions::Dim2d { width: 2, height: 2 },
			Format::R8G8B8A8Unorm,
		)
		.unwrap();

		Ok((
			Arc::new(Self { instance, device, queue, sampler, pipeline_ctxs, active_pipeline, white_pixel }),
			white_pixel_future,
		))
	}

	fn device(&self) -> &Arc<Device> {
		&self.device
	}

	fn queue(&self) -> &Arc<Queue> {
		&self.queue
	}

	fn sampler(&self) -> &Arc<Sampler> {
		&self.sampler
	}

	fn pipeline_ctx(&self) -> &dyn PipelineContext {
		self.pipeline_ctxs[self.active_pipeline].as_ref()
	}

	fn white_pixel(&self) -> &texture::Texture {
		&self.white_pixel
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
