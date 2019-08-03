use super::{
	context::DeferredPipelineContextInner, geom_fshader, geom_vshader, swap_fshader, swap_vshader, Vert2D,
	DEPTH_FORMAT, DIFFUSE_FORMAT, NORMAL_FORMAT,
};
use crate::{camera::Camera, mesh::Mesh, mesh_data, pipelines::Pipeline};
use std::sync::Arc;
use vulkano::{
	command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
	descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet, PipelineLayoutAbstract},
	framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
	image::{AttachmentImage, ImageViewAccess},
	instance::QueueFamily,
	pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract},
};

pub(super) struct DeferredPipeline {
	ctx: Arc<DeferredPipelineContextInner>,
	geom_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	swap_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
	gbuffers_desc: Arc<dyn DescriptorSet + Send + Sync>,
}
impl DeferredPipeline {
	pub(super) fn new(
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
