use crate::{
	camera::Camera,
	mesh::Mesh,
	mesh_data,
	pipelines::{Pipeline, PipelineContext, PipelineDef},
	surface::SWAP_FORMAT,
};
use std::sync::Arc;
use vulkano::{
	buffer::{BufferAccess, BufferUsage, ImmutableBuffer, TypedBufferAccess},
	command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
	descriptor::{
		descriptor::ShaderStages, descriptor_set::PersistentDescriptorSet, pipeline_layout::PipelineLayoutDesc,
		DescriptorSet, PipelineLayoutAbstract,
	},
	device::{Device, Queue},
	format::Format,
	framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
	image::{AttachmentImage, ImageViewAccess},
	instance::QueueFamily,
	pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract},
	sync::GpuFuture,
};

pub const DIFFUSE_FORMAT: Format = Format::A2B10G10R10UnormPack32;
pub const NORMAL_FORMAT: Format = Format::R16G16B16A16Sfloat;
pub const DEPTH_FORMAT: Format = Format::D16Unorm;

pub struct DeferredPipelineDef;
impl PipelineDef for DeferredPipelineDef {
	fn make_context(device: &Arc<Device>, queue: &Arc<Queue>) -> Box<dyn PipelineContext> {
		Box::new(DeferredPipelineContext::new(device, queue))
	}
}

pub struct DeferredPipelineContext {
	inner: Arc<DeferredPipelineContextInner>,
}
impl DeferredPipelineContext {
	fn new(device: &Arc<Device>, queue: &Arc<Queue>) -> Self {
		let render_pass = Arc::new(
			vulkano::ordered_passes_renderpass!(
				device.clone(),
				attachments: {
					diffuse: { load: Clear, store: DontCare, format: DIFFUSE_FORMAT, samples: 1, },
					normal: { load: Clear, store: DontCare, format: NORMAL_FORMAT, samples: 1, },
					depth: { load: Clear, store: DontCare, format: DEPTH_FORMAT, samples: 1, },
					color: { load: Clear, store: Store, format: SWAP_FORMAT, samples: 1, }
				},
				passes: [
					{ color: [diffuse, normal], depth_stencil: {depth}, input: [] },
					{ color: [color], depth_stencil: {}, input: [diffuse, normal, depth] }
				]
			)
			.unwrap(),
		);

		let geom_vshader = geom_vshader::Shader::load(device.clone()).unwrap();
		let geom_fshader = geom_fshader::Shader::load(device.clone()).unwrap();
		let vs_layout = geom_vshader::Layout(ShaderStages { vertex: true, ..ShaderStages::none() });
		let fs_layout = geom_fshader::Layout(ShaderStages { fragment: true, ..ShaderStages::none() });
		let layout_desc = Arc::new(vs_layout.union(fs_layout).build(device.clone()).unwrap());

		let swap_vshader = swap_vshader::Shader::load(device.clone()).unwrap();
		let swap_fshader = swap_fshader::Shader::load(device.clone()).unwrap();
		let swap_vs_layout = swap_vshader::Layout(ShaderStages { vertex: true, ..ShaderStages::none() });
		let swap_fs_layout = swap_fshader::Layout(ShaderStages { fragment: true, ..ShaderStages::none() });
		let swap_layout_desc = Arc::new(swap_vs_layout.union(swap_fs_layout).build(device.clone()).unwrap());

		let vertdata = [
			Vert2D { pos: [-1.0, 1.0], tex: [0.0, 0.0] },
			Vert2D { pos: [1.0, 1.0], tex: [1.0, 0.0] },
			Vert2D { pos: [1.0, -1.0], tex: [1.0, 1.0] },
			Vert2D { pos: [-1.0, -1.0], tex: [0.0, 1.0] },
		];
		let (vertices, vertices_future) =
			ImmutableBuffer::from_data(vertdata, BufferUsage::vertex_buffer(), queue.clone()).unwrap();
		let (indices, indices_future) =
			ImmutableBuffer::from_iter(vec![0, 1, 2, 2, 3, 0].into_iter(), BufferUsage::index_buffer(), queue.clone())
				.unwrap();
		vertices_future.join(indices_future).then_signal_fence_and_flush().unwrap().wait(None).unwrap();

		Self {
			inner: Arc::new(DeferredPipelineContextInner {
				render_pass,
				geom_vshader,
				geom_fshader,
				layout_desc,
				swap_vshader,
				swap_fshader,
				swap_layout_desc,
				vertices,
				indices,
			}),
		}
	}
}
impl PipelineContext for DeferredPipelineContext {
	fn make_pipeline(
		&self,
		images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
		dimensions: [u32; 2],
	) -> Box<dyn Pipeline> {
		Box::new(DeferredPipeline::new(self.inner.clone(), images, dimensions))
	}

	fn layout_desc(&self) -> &Arc<dyn PipelineLayoutAbstract + Send + Sync> {
		&self.inner.layout_desc
	}
}

struct DeferredPipeline {
	ctx: Arc<DeferredPipelineContextInner>,
	geom_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	swap_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
	gbuffers_desc: Arc<dyn DescriptorSet + Send + Sync>,
}
impl DeferredPipeline {
	fn new(
		ctx: Arc<DeferredPipelineContextInner>,
		images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
		dimensions: [u32; 2],
	) -> Self {
		let geom_pipeline =
			create_geom_pipeline(&ctx.geom_vshader, &ctx.geom_fshader, ctx.render_pass.clone(), dimensions);
		let swap_pipeline =
			create_swap_pipeline(&ctx.swap_vshader, &ctx.swap_fshader, ctx.render_pass.clone(), dimensions);

		let diffuse_image = Arc::new(
			AttachmentImage::transient_input_attachment(ctx.render_pass.device().clone(), dimensions, DIFFUSE_FORMAT)
				.unwrap(),
		);
		let normal_image = Arc::new(
			AttachmentImage::transient_input_attachment(ctx.render_pass.device().clone(), dimensions, NORMAL_FORMAT)
				.unwrap(),
		);
		let depth_image = Arc::new(
			AttachmentImage::transient_input_attachment(ctx.render_pass.device().clone(), dimensions, DEPTH_FORMAT)
				.unwrap(),
		);

		let framebuffers = create_framebuffers(
			&ctx.render_pass,
			diffuse_image.clone(),
			normal_image.clone(),
			depth_image.clone(),
			images,
		);
		let gbuffers_desc =
			make_gbuffers_desc_set(ctx.swap_layout_desc.clone(), diffuse_image, normal_image, depth_image);

		Self { ctx, geom_pipeline, swap_pipeline, framebuffers, gbuffers_desc }
	}
}
impl Pipeline for DeferredPipeline {
	fn draw(&self, image_num: usize, qfam: QueueFamily, cam: &Camera, meshes: &[&Mesh]) -> AutoCommandBuffer {
		let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into(), [0.0; 4].into(), 1.0.into(), [0.0; 4].into()];

		let make_pc = |mesh: &Mesh| geom_vshader::ty::PushConsts {
			cam_proj: cam.projection().into(),
			cam_pos: cam.transform().pos.into(),
			cam_rot: cam.transform().rot.into(),
			mesh_pos: mesh.transform().pos.into(),
			mesh_rot: mesh.transform().rot.into(),
			_dummy0: unsafe { std::mem::uninitialized() },
			_dummy1: unsafe { std::mem::uninitialized() },
		};

		let mut command_buffer =
			AutoCommandBufferBuilder::primary_one_time_submit(self.ctx.render_pass.device().clone(), qfam)
				.unwrap()
				.begin_render_pass(self.framebuffers[image_num].clone(), false, clear_values)
				.unwrap();
		for mesh in meshes {
			let mesh_data = mesh.mesh_data().as_ref().unwrap();
			command_buffer = command_buffer
				.draw_indexed(
					self.geom_pipeline.clone(),
					&Default::default(),
					vec![mesh_data.vertices().clone()],
					mesh_data.indices().clone(),
					mesh.texture_desc().clone(),
					make_pc(mesh),
				)
				.unwrap();
		}

		command_buffer
			.next_subpass(false)
			.unwrap()
			.draw_indexed(
				self.swap_pipeline.clone(),
				&Default::default(),
				vec![self.ctx.vertices.clone()],
				self.ctx.indices.clone(),
				self.gbuffers_desc.clone(),
				(),
			)
			.unwrap()
			.end_render_pass()
			.unwrap()
			.build()
			.unwrap()
	}

	fn resize(&mut self, images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>, dimensions: [u32; 2]) {
		self.geom_pipeline = create_geom_pipeline(
			&self.ctx.geom_vshader,
			&self.ctx.geom_fshader,
			self.ctx.render_pass.clone(),
			dimensions,
		);
		self.swap_pipeline = create_swap_pipeline(
			&self.ctx.swap_vshader,
			&self.ctx.swap_fshader,
			self.ctx.render_pass.clone(),
			dimensions,
		);

		let diffuse_image = Arc::new(
			AttachmentImage::transient_input_attachment(
				self.ctx.render_pass.device().clone(),
				dimensions,
				DIFFUSE_FORMAT,
			)
			.unwrap(),
		);
		let normal_image = Arc::new(
			AttachmentImage::transient_input_attachment(
				self.ctx.render_pass.device().clone(),
				dimensions,
				NORMAL_FORMAT,
			)
			.unwrap(),
		);
		let depth_image = Arc::new(
			AttachmentImage::transient_input_attachment(
				self.ctx.render_pass.device().clone(),
				dimensions,
				DEPTH_FORMAT,
			)
			.unwrap(),
		);
		self.framebuffers = create_framebuffers(
			&self.ctx.render_pass,
			diffuse_image.clone(),
			normal_image.clone(),
			depth_image.clone(),
			images,
		);

		self.gbuffers_desc =
			make_gbuffers_desc_set(self.ctx.swap_layout_desc.clone(), diffuse_image, normal_image, depth_image);
	}
}

struct DeferredPipelineContextInner {
	render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	geom_vshader: geom_vshader::Shader,
	geom_fshader: geom_fshader::Shader,
	layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	swap_vshader: swap_vshader::Shader,
	swap_fshader: swap_fshader::Shader,
	swap_layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	vertices: Arc<dyn BufferAccess + Send + Sync>,
	indices: Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>,
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
struct Vert2D {
	pub pos: [f32; 2],
	pub tex: [f32; 2],
}
vulkano::impl_vertex!(Vert2D, pos, tex);

fn create_framebuffers(
	swap_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
	diffuse: Arc<dyn ImageViewAccess + Send + Sync>,
	normal: Arc<dyn ImageViewAccess + Send + Sync>,
	depth: Arc<dyn ImageViewAccess + Send + Sync>,
	images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
	images
		.into_iter()
		.map(|image| {
			Arc::new(
				Framebuffer::start(swap_pass.clone())
					.add(diffuse.clone())
					.unwrap()
					.add(normal.clone())
					.unwrap()
					.add(depth.clone())
					.unwrap()
					.add(image)
					.unwrap()
					.build()
					.unwrap(),
			) as Arc<dyn FramebufferAbstract + Send + Sync>
		})
		.collect()
}

fn create_geom_pipeline(
	vshader: &geom_vshader::Shader,
	fshader: &geom_fshader::Shader,
	render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	dimensions: [u32; 2],
) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
	let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
	let device = render_pass.device().clone();
	Arc::new(
		GraphicsPipeline::start()
			.vertex_input_single_buffer::<mesh_data::Pntl_32F>()
			.vertex_shader(vshader.main_entry_point(), ())
			.fragment_shader(fshader.main_entry_point(), ())
			.triangle_list()
			.viewports(vec![Viewport { origin: [0.0, 0.0], dimensions, depth_range: 0.0..1.0 }])
			.render_pass(Subpass::from(render_pass, 0).unwrap())
			.depth_stencil_simple_depth()
			.build(device)
			.unwrap(),
	)
}

fn create_swap_pipeline(
	vshader: &swap_vshader::Shader,
	fshader: &swap_fshader::Shader,
	render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	dimensions: [u32; 2],
) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
	let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
	let device = render_pass.device().clone();
	Arc::new(
		GraphicsPipeline::start()
			.vertex_input_single_buffer::<Vert2D>()
			.vertex_shader(vshader.main_entry_point(), ())
			.fragment_shader(fshader.main_entry_point(), ())
			.triangle_list()
			.viewports(vec![Viewport { origin: [0.0, 0.0], dimensions, depth_range: 0.0..1.0 }])
			.render_pass(Subpass::from(render_pass, 1).unwrap())
			.build(device)
			.unwrap(),
	)
}

fn make_gbuffers_desc_set<L>(
	layout: L,
	diffuse: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
	normal: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
	depth: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
) -> Arc<dyn DescriptorSet + Send + Sync>
where
	L: PipelineLayoutAbstract + Send + Sync + 'static,
{
	Arc::new(
		PersistentDescriptorSet::start(layout, 0)
			.add_image(diffuse)
			.unwrap()
			.add_image(normal)
			.unwrap()
			.add_image(depth)
			.unwrap()
			.build()
			.unwrap(),
	)
}

pub(crate) mod geom_vshader {
	vulkano_shaders::shader! {
		ty: "vertex",
		src: "
#version 450
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 nor;
layout(location = 2) in vec2 tex;
layout(location = 3) in vec2 lmap;

layout(location = 0) out vec3 out_nor;
layout(location = 1) out vec2 out_texc;

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

	out_nor = nor;
	out_texc = tex;
	gl_Position = perspective(pc.cam_proj, pos_cs);
}"
	}
}

pub(crate) mod geom_fshader {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450
layout(location = 0) in vec3 nor;
layout(location = 1) in vec2 texc;

layout(location = 0) out vec4 diffuse;
layout(location = 1) out vec4 normal;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
	diffuse = texture(tex, texc);
	normal = vec4(nor, 0);
}
"
	}
}

pub(crate) mod swap_vshader {
	vulkano_shaders::shader! {
		ty: "vertex",
		src: "
#version 450
layout(location = 0) in vec2 pos;
layout(location = 2) in vec2 tex;

layout(location = 0) out vec2 out_texc;

void main() {
	out_texc = tex;
	gl_Position = vec4(pos, 0, 1);
}"
	}
}

pub(crate) mod swap_fshader {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450
layout(location = 0) in vec2 texc;

layout(location = 0) out vec4 color;

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput diffuse;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput normal;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput depth;

void main() {
	color = subpassLoad(diffuse);
}
"
	}
}
