use super::{
	context::ForwardPipelineContextInner, depth_fshader, depth_vshader, swap_fshader, swap_vshader, DEPTH_FORMAT,
};
use crate::{camera::Camera, mesh::Mesh, mesh_data, direct_light::DirectLight, pipelines::Pipeline};
use std::sync::Arc;
use vulkano::{
	buffer::BufferAccess,
	command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
	framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
	image::{AttachmentImage, ImageViewAccess},
	instance::QueueFamily,
	pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract},
};

pub(super) struct ForwardPipeline {
	ctx: Arc<ForwardPipelineContextInner>,
	depth_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	depth_framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
	pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}
impl ForwardPipeline {
	pub(super) fn new(
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
	fn draw(&self, image_num: usize, qfam: QueueFamily, cam: &Camera, meshes: &[&Mesh], lights: &[&DirectLight]) -> AutoCommandBuffer {
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
			for mat in mesh.texture_descs() {
				command_buffer = command_buffer
					.draw_indexed(
						self.pipeline.clone(),
						&Default::default(),
						vec![mesh_data.vertices().clone()],
						mesh_data.indices().clone().into_buffer_slice().slice(mat.range.clone()).unwrap(),
						mat.tex1.clone(),
						make_pc(mesh),
					)
					.unwrap();
			}
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
