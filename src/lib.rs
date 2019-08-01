pub mod camera;
pub mod mesh;
pub mod mesh_data;
pub mod surface;
pub mod texture;
pub mod transform;
#[cfg(feature = "window")]
pub mod window;

use crate::surface::{DEPTH_FORMAT, SWAP_FORMAT};
use log::info;
use std::sync::Arc;
use vulkano::{
	descriptor::{
		descriptor::ShaderStages,
		pipeline_layout::{PipelineLayout, PipelineLayoutDesc, PipelineLayoutDescUnion},
	},
	device::{Device, DeviceExtensions, Features, Queue},
	format::Format,
	framebuffer::RenderPassAbstract,
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
	context_3d: Context3D,
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

		let context_3d = Context3D::new(&device);

		let (white_pixel, white_pixel_future) = texture::Texture::from_iter_vk(
			queue.clone(),
			vec![[0u8, 0, 255, 255], [0, 0, 0, 255], [0, 0, 0, 255], [0, 0, 255, 255]].into_iter(),
			Dimensions::Dim2d { width: 2, height: 2 },
			Format::R8G8B8A8Unorm,
		)
		.unwrap();

		Ok((Arc::new(Self { instance, device, queue, sampler, context_3d, white_pixel }), white_pixel_future))
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

	fn context_3d(&self) -> &Context3D {
		&self.context_3d
	}

	fn white_pixel(&self) -> &texture::Texture {
		&self.white_pixel
	}
}

struct Context3D {
	depth_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	depth_vs: depth_vs::Shader,
	depth_fs: depth_fs::Shader,
	render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	vs: vs3d::Shader,
	fs: fs3d::Shader,
	layout_desc: Arc<PipelineLayout<PipelineLayoutDescUnion<vs3d::Layout, fs3d::Layout>>>,
}
impl Context3D {
	fn new(device: &Arc<Device>) -> Self {
		let depth_pass = Arc::new(
			vulkano::single_pass_renderpass!(
				device.clone(),
				attachments: { depth: { load: Clear, store: Store, format: DEPTH_FORMAT, samples: 1, } },
				pass: { color: [], depth_stencil: {depth} }
			)
			.unwrap(),
		);
		let depth_vs = depth_vs::Shader::load(device.clone()).unwrap();
		let depth_fs = depth_fs::Shader::load(device.clone()).unwrap();

		let render_pass = Arc::new(
			vulkano::single_pass_renderpass!(
				device.clone(),
				attachments: { color: { load: Clear, store: Store, format: SWAP_FORMAT, samples: 1, } },
				pass: { color: [color], depth_stencil: {} }
			)
			.unwrap(),
		);
		let vs = vs3d::Shader::load(device.clone()).unwrap();
		let fs = fs3d::Shader::load(device.clone()).unwrap();

		let vs_layout = vs3d::Layout(ShaderStages { vertex: true, ..ShaderStages::none() });
		let fs_layout = fs3d::Layout(ShaderStages { fragment: true, ..ShaderStages::none() });
		let layout_desc = Arc::new(vs_layout.union(fs_layout).build(device.clone()).unwrap());

		Self { depth_pass, depth_vs, depth_fs, render_pass, vs, fs, layout_desc }
	}

	fn depth_pass(&self) -> &Arc<dyn RenderPassAbstract + Send + Sync> {
		&self.depth_pass
	}

	fn depth_vs(&self) -> &depth_vs::Shader {
		&self.depth_vs
	}

	fn depth_fs(&self) -> &depth_fs::Shader {
		&self.depth_fs
	}

	fn render_pass(&self) -> &Arc<dyn RenderPassAbstract + Send + Sync> {
		&self.render_pass
	}

	fn vertex_shader(&self) -> &vs3d::Shader {
		&self.vs
	}

	fn fragment_shader(&self) -> &fs3d::Shader {
		&self.fs
	}

	fn layout_desc(&self) -> &Arc<PipelineLayout<PipelineLayoutDescUnion<vs3d::Layout, fs3d::Layout>>> {
		&self.layout_desc
	}
}

pub(crate) mod depth_vs {
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

pub(crate) mod depth_fs {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450

void main() {
	gl_FragDepth = gl_FragCoord.z;
}
"
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

layout(location = 0) out vec2 out_texc;

layout(push_constant) uniform PushConsts {
	vec4 cam_proj;
	vec3 cam_pos;
	vec4 cam_rot;
	vec3 mesh_pos;
	vec4 mesh_rot;
} pc;

layout(set = 0, binding = 0) uniform sampler2D tex1;

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

	out_texc = tex;
	gl_Position = perspective(pc.cam_proj, pos_cs);
}"
	}
}

pub(crate) mod fs3d {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450
layout(location = 0) in vec2 texc;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
	f_color = texture(tex, texc);
}
"
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
