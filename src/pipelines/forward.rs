use crate::{
	camera::Camera,
	mesh::Mesh,
	mesh_data,
	pipelines::{Pipeline, PipelineContext, PipelineDef},
	surface::SWAP_FORMAT,
};
use std::sync::Arc;
use vulkano::{
	command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
	descriptor::{descriptor::ShaderStages, pipeline_layout::PipelineLayoutDesc, PipelineLayoutAbstract},
	device::{Device, Queue},
	format::Format,
	framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
	image::{AttachmentImage, ImageViewAccess},
	instance::QueueFamily,
	pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract},
};

pub const DEPTH_FORMAT: Format = Format::D16Unorm;

pub struct ForwardPipelineDef;
impl PipelineDef for ForwardPipelineDef {
	fn make_context(device: &Arc<Device>, _queue: &Arc<Queue>) -> Box<dyn PipelineContext> {
		Box::new(ForwardPipelineContext::new(device))
	}
}

pub struct ForwardPipelineContext {
	inner: Arc<ForwardPipelineContextInner>,
}
impl ForwardPipelineContext {
	fn new(device: &Arc<Device>) -> Self {
		let depth_pass = Arc::new(
			vulkano::single_pass_renderpass!(
				device.clone(),
				attachments: { depth: { load: Clear, store: Store, format: DEPTH_FORMAT, samples: 1, } },
				pass: { color: [], depth_stencil: {depth} }
			)
			.unwrap(),
		);
		let depth_vshader = depth_vshader::Shader::load(device.clone()).unwrap();
		let depth_fshader = depth_fshader::Shader::load(device.clone()).unwrap();

		let swap_pass = Arc::new(
			vulkano::single_pass_renderpass!(
				device.clone(),
				attachments: { color: { load: Clear, store: Store, format: SWAP_FORMAT, samples: 1, } },
				pass: { color: [color], depth_stencil: {} }
			)
			.unwrap(),
		);
		let swap_vshader = swap_vshader::Shader::load(device.clone()).unwrap();
		let swap_fshader = swap_fshader::Shader::load(device.clone()).unwrap();

		let vs_layout = swap_vshader::Layout(ShaderStages { vertex: true, ..ShaderStages::none() });
		let fs_layout = swap_fshader::Layout(ShaderStages { fragment: true, ..ShaderStages::none() });
		let layout_desc = Arc::new(vs_layout.union(fs_layout).build(device.clone()).unwrap());

		Self {
			inner: Arc::new(ForwardPipelineContextInner {
				depth_pass,
				depth_vshader,
				depth_fshader,
				swap_pass,
				swap_vshader,
				swap_fshader,
				layout_desc,
			}),
		}
	}
}
impl PipelineContext for ForwardPipelineContext {
	fn make_pipeline(
		&self,
		images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
		dimensions: [u32; 2],
	) -> Box<dyn Pipeline> {
		Box::new(ForwardPipeline::new(self.inner.clone(), images, dimensions))
	}

	fn layout_desc(&self) -> &Arc<dyn PipelineLayoutAbstract + Send + Sync> {
		&self.inner.layout_desc
	}
}

struct ForwardPipeline {
	ctx: Arc<ForwardPipelineContextInner>,
	depth_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	depth_framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
	pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}
impl ForwardPipeline {
	fn new(
		ctx: Arc<ForwardPipelineContextInner>,
		images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
		dimensions: [u32; 2],
	) -> Self {
		let depth_pipeline =
			create_depth_pipeline(&ctx.depth_vshader, &ctx.depth_fshader, ctx.depth_pass.clone(), dimensions);
		let depth_image =
			Arc::new(AttachmentImage::new(ctx.depth_pass.device().clone(), dimensions, DEPTH_FORMAT).unwrap());
		let depth_framebuffer =
			Arc::new(Framebuffer::start(ctx.depth_pass.clone()).add(depth_image.clone()).unwrap().build().unwrap());
		let pipeline = create_pipeline_3d(&ctx.swap_vshader, &ctx.swap_fshader, ctx.swap_pass.clone(), dimensions);
		let framebuffers = create_framebuffers(&ctx.swap_pass, images);

		Self { ctx, depth_pipeline, depth_framebuffer, pipeline, framebuffers }
	}
}
impl Pipeline for ForwardPipeline {
	fn draw(&self, image_num: usize, qfam: QueueFamily, cam: &Camera, meshes: &[&Mesh]) -> AutoCommandBuffer {
		let clear_values = vec![[0.0, 0.0, 0.25, 1.0].into()];

		let make_pc = |mesh: &Mesh| swap_vshader::ty::PushConsts {
			cam_proj: cam.projection().into(),
			cam_pos: cam.transform().pos.into(),
			cam_rot: cam.transform().rot.into(),
			mesh_pos: mesh.transform().pos.into(),
			mesh_rot: mesh.transform().rot.into(),
			_dummy0: unsafe { std::mem::uninitialized() },
			_dummy1: unsafe { std::mem::uninitialized() },
		};

		let mut command_buffer =
			AutoCommandBufferBuilder::primary_one_time_submit(self.ctx.depth_pass.device().clone(), qfam)
				.unwrap()
				.begin_render_pass(self.depth_framebuffer.clone(), false, vec![1.0.into()])
				.unwrap();
		for mesh in meshes {
			let mesh_data = mesh.mesh_data().as_ref().unwrap();
			command_buffer = command_buffer
				.draw_indexed(
					self.depth_pipeline.clone(),
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
			.begin_render_pass(self.framebuffers[image_num].clone(), false, clear_values)
			.unwrap();
		for mesh in meshes {
			let mesh_data = mesh.mesh_data().as_ref().unwrap();
			command_buffer = command_buffer
				.draw_indexed(
					self.pipeline.clone(),
					&Default::default(),
					vec![mesh_data.vertices().clone()],
					mesh_data.indices().clone(),
					mesh.texture_desc().clone(),
					make_pc(mesh),
				)
				.unwrap();
		}

		command_buffer.end_render_pass().unwrap().build().unwrap()
	}

	fn resize(&mut self, images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>, dimensions: [u32; 2]) {
		self.depth_pipeline = create_depth_pipeline(
			&self.ctx.depth_vshader,
			&self.ctx.depth_fshader,
			self.ctx.depth_pass.clone(),
			dimensions,
		);
		self.pipeline =
			create_pipeline_3d(&self.ctx.swap_vshader, &self.ctx.swap_fshader, self.ctx.swap_pass.clone(), dimensions);
		self.framebuffers = create_framebuffers(&self.ctx.swap_pass, images);
	}
}

struct ForwardPipelineContextInner {
	depth_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	depth_vshader: depth_vshader::Shader,
	depth_fshader: depth_fshader::Shader,
	swap_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	swap_vshader: swap_vshader::Shader,
	swap_fshader: swap_fshader::Shader,
	layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
}

fn create_framebuffers(
	swap_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
	images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
	images
		.into_iter()
		.map(move |image| {
			Arc::new(Framebuffer::start(swap_pass.clone()).add(image).unwrap().build().unwrap())
				as Arc<dyn FramebufferAbstract + Send + Sync>
		})
		.collect()
}

fn create_depth_pipeline(
	vshader: &depth_vshader::Shader,
	fshader: &depth_fshader::Shader,
	swap_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	dimensions: [u32; 2],
) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
	let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
	let device = swap_pass.device().clone();
	Arc::new(
		GraphicsPipeline::start()
			.vertex_input_single_buffer::<mesh_data::Pntl_32F>()
			.vertex_shader(vshader.main_entry_point(), ())
			.fragment_shader(fshader.main_entry_point(), ())
			.triangle_list()
			.viewports(vec![Viewport { origin: [0.0, 0.0], dimensions, depth_range: 0.0..1.0 }])
			.render_pass(Subpass::from(swap_pass, 0).unwrap())
			.depth_stencil_simple_depth()
			.build(device)
			.unwrap(),
	)
}

fn create_pipeline_3d(
	vshader: &swap_vshader::Shader,
	fshader: &swap_fshader::Shader,
	swap_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	dimensions: [u32; 2],
) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
	let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
	let device = swap_pass.device().clone();
	Arc::new(
		GraphicsPipeline::start()
			.vertex_input_single_buffer::<mesh_data::Pntl_32F>()
			.vertex_shader(vshader.main_entry_point(), ())
			.fragment_shader(fshader.main_entry_point(), ())
			.triangle_list()
			.viewports(vec![Viewport { origin: [0.0, 0.0], dimensions, depth_range: 0.0..1.0 }])
			.render_pass(Subpass::from(swap_pass, 0).unwrap())
			.build(device)
			.unwrap(),
	)
}

pub(crate) mod depth_vshader {
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

pub(crate) mod depth_fshader {
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

pub(crate) mod swap_vshader {
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

pub(crate) mod swap_fshader {
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
