use super::{
	context::DeferredPipelineContextInner, geom_fshader, geom_vshader, light_fshader, light_vshader, swap_fshader,
	swap_vshader, Vert2D, DEPTH_FORMAT, DIFFUSE_FORMAT, LIGHT_FORMAT, NORMAL_FORMAT, POSITION_FORMAT,
};
use crate::{
	camera::Camera,
	direct_light::DirectLight,
	mesh::MeshInner,
	mesh_data::{IndexBuffer, Pntl_32F},
	pipelines::Pipeline,
};
use std::sync::Arc;
use vulkano::{
	buffer::BufferAccess,
	command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
	descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet, PipelineLayoutAbstract},
	device::Device,
	framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
	image::{AttachmentImage, ImageViewAccess},
	instance::QueueFamily,
	pipeline::{
		blend::{AttachmentBlend, BlendFactor, BlendOp},
		input_assembly::PrimitiveTopology,
		viewport::Viewport,
		GraphicsPipeline, GraphicsPipelineAbstract,
	},
};

pub(super) struct DeferredPipeline {
	ctx: Arc<DeferredPipelineContextInner>,
	geom_pipeline_soup: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	geom_pipeline_strip: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	light_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	swap_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
	framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
	gbuffers_desc: Arc<dyn DescriptorSet + Send + Sync>,
	dimensions: [u32; 2],
}
impl DeferredPipeline {
	pub(super) fn new(
		ctx: Arc<DeferredPipelineContextInner>,
		images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
		dimensions: [u32; 2],
	) -> Self {
		let (geom_pipeline_soup, geom_pipeline_strip) =
			create_geom_pipelines(&ctx.geom_vshader, &ctx.geom_fshader, &ctx.render_pass, dimensions);
		let light_pipeline =
			create_light_pipeline(&ctx.light_vshader, &ctx.light_fshader, ctx.render_pass.clone(), dimensions);
		let swap_pipeline =
			create_swap_pipeline(&ctx.swap_vshader, &ctx.swap_fshader, ctx.render_pass.clone(), dimensions);

		let gbuffers = create_gbuffers(ctx.render_pass.device(), dimensions);
		let framebuffers = create_framebuffers(&ctx.render_pass, &gbuffers, images);
		let gbuffers_desc = make_gbuffers_desc(ctx.light_layout_desc.clone(), &gbuffers);

		Self {
			ctx,
			geom_pipeline_soup,
			geom_pipeline_strip,
			swap_pipeline,
			light_pipeline,
			framebuffers,
			gbuffers_desc,
			dimensions,
		}
	}
}
impl Pipeline for DeferredPipeline {
	fn draw(&self, image_num: usize, qfam: QueueFamily, cam: &Camera, lights: &[DirectLight]) -> AutoCommandBuffer {
		let clear_values = vec![
			1.0.into(),
			[0.0, 0.0, 0.0, 1.0].into(),
			[0.0; 4].into(),
			[0.0; 4].into(),
			[0.0; 4].into(),
			[0.0; 4].into(),
		];

		let make_pc = |mesh: &MeshInner| geom_vshader::ty::PushConsts {
			cam_proj: cam.projection().into(),
			cam_pos: cam.transform().pos.into(),
			cam_rot: cam.transform().rot.into(),
			mesh_pos: mesh.transform().pos.into(),
			mesh_rot: mesh.transform().rot.into(),
		};

		let mut command_buffer =
			AutoCommandBufferBuilder::primary_one_time_submit(self.ctx.render_pass.device().clone(), qfam)
				.unwrap()
				.begin_render_pass(self.framebuffers[image_num].clone(), false, clear_values)
				.unwrap();
		for mesh in cam.mesh_group().lock().unwrap().values() {
			let mut mesh = mesh.lock().unwrap();
			mesh.refresh();

			let mesh_data = mesh.mesh_data().as_ref().unwrap();
			for mat in mesh.texture_descs() {
				let pipeline = match mesh_data.topology() {
					PrimitiveTopology::TriangleList => self.geom_pipeline_soup.clone(),
					PrimitiveTopology::TriangleStrip => self.geom_pipeline_strip.clone(),
					_ => unimplemented!(),
				};
				let dynamic = Default::default();
				let vertex_buffer = vec![mesh_data.vertices().clone()];
				let sets = mat.tex1().clone();
				let pc = make_pc(&mesh);
				match mesh_data.indices() {
					IndexBuffer::U16(buf) => {
						command_buffer = command_buffer
							.draw_indexed(
								pipeline,
								&dynamic,
								vertex_buffer,
								buf.clone().into_buffer_slice().slice(mat.range().clone()).unwrap(),
								sets,
								pc,
							)
							.unwrap()
					},
					IndexBuffer::U32(buf) => {
						command_buffer = command_buffer
							.draw_indexed(
								pipeline,
								&dynamic,
								vertex_buffer,
								buf.clone().into_buffer_slice().slice(mat.range().clone()).unwrap(),
								sets,
								pc,
							)
							.unwrap()
					},
				}
			}
		}

		command_buffer = command_buffer.next_subpass(false).unwrap();
		for light in lights {
			let light_cutoff = 0.003035269835488375;
			let radius_squared = light.radius * light.radius;

			command_buffer = command_buffer
				.draw_indexed(
					self.light_pipeline.clone(),
					&Default::default(),
					vec![self.ctx.vertices.clone()],
					self.ctx.indices.clone(),
					self.gbuffers_desc.clone(),
					light_fshader::ty::PushConsts {
						Resolution: [
							self.dimensions[0] as f32,
							self.dimensions[1] as f32,
							1.0 / (self.dimensions[0] as f32),
							1.0 / (self.dimensions[1] as f32),
						],
						Projection: cam.projection().into(),
						CameraRotation: cam.transform().rot.into(),
						CameraOffset: cam.transform().pos.into(),
						LightPosition: [light.position.x, light.position.y, light.position.z, 1.0 / radius_squared],
						LightColor: [light.color.x, light.color.y, light.color.z, light_cutoff * radius_squared],
					},
				)
				.unwrap();
		}

		command_buffer = command_buffer
			.next_subpass(false)
			.unwrap()
			.draw_indexed(
				self.swap_pipeline.clone(),
				&Default::default(),
				vec![self.ctx.vertices.clone()],
				self.ctx.indices.clone(),
				self.gbuffers_desc.clone(),
				swap_fshader::ty::PushConsts { Exposure: cam.exposure },
			)
			.unwrap();

		command_buffer.end_render_pass().unwrap().build().unwrap()
	}

	fn resize(&mut self, images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>, dimensions: [u32; 2]) {
		let (geom_pipeline_soup, geom_pipeline_strip) =
			create_geom_pipelines(&self.ctx.geom_vshader, &self.ctx.geom_fshader, &self.ctx.render_pass, dimensions);
		self.geom_pipeline_soup = geom_pipeline_soup;
		self.geom_pipeline_strip = geom_pipeline_strip;

		self.light_pipeline = create_light_pipeline(
			&self.ctx.light_vshader,
			&self.ctx.light_fshader,
			self.ctx.render_pass.clone(),
			dimensions,
		);
		self.swap_pipeline = create_swap_pipeline(
			&self.ctx.swap_vshader,
			&self.ctx.swap_fshader,
			self.ctx.render_pass.clone(),
			dimensions,
		);

		let gbuffers = create_gbuffers(self.ctx.render_pass.device(), dimensions);
		self.framebuffers = create_framebuffers(&self.ctx.render_pass, &gbuffers, images);
		self.gbuffers_desc = make_gbuffers_desc(self.ctx.swap_layout_desc.clone(), &gbuffers);
	}
}

fn create_framebuffers(
	swap_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
	gbuffers: &GBuffers,
	images: Vec<Arc<dyn ImageViewAccess + Send + Sync>>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
	images
		.into_iter()
		.map(|image| {
			Arc::new(
				Framebuffer::start(swap_pass.clone())
					.add(gbuffers.depth.clone())
					.unwrap()
					.add(gbuffers.diffuse.clone())
					.unwrap()
					.add(gbuffers.normal.clone())
					.unwrap()
					.add(gbuffers.position.clone())
					.unwrap()
					.add(gbuffers.light.clone())
					.unwrap()
					.add(image)
					.unwrap()
					.build()
					.unwrap(),
			) as Arc<dyn FramebufferAbstract + Send + Sync>
		})
		.collect()
}

fn create_gbuffers(device: &Arc<Device>, dimensions: [u32; 2]) -> GBuffers {
	let depth =
		Arc::new(AttachmentImage::transient_input_attachment(device.clone(), dimensions, DEPTH_FORMAT).unwrap());
	let diffuse =
		Arc::new(AttachmentImage::transient_input_attachment(device.clone(), dimensions, DIFFUSE_FORMAT).unwrap());
	let normal =
		Arc::new(AttachmentImage::transient_input_attachment(device.clone(), dimensions, NORMAL_FORMAT).unwrap());
	let position =
		Arc::new(AttachmentImage::transient_input_attachment(device.clone(), dimensions, POSITION_FORMAT).unwrap());
	let light =
		Arc::new(AttachmentImage::transient_input_attachment(device.clone(), dimensions, LIGHT_FORMAT).unwrap());

	GBuffers { diffuse, normal, depth, position, light }
}

struct GBuffers {
	depth: Arc<dyn ImageViewAccess + Send + Sync>,
	diffuse: Arc<dyn ImageViewAccess + Send + Sync>,
	normal: Arc<dyn ImageViewAccess + Send + Sync>,
	position: Arc<dyn ImageViewAccess + Send + Sync>,
	light: Arc<dyn ImageViewAccess + Send + Sync>,
}

fn create_geom_pipelines(
	vshader: &geom_vshader::Shader,
	fshader: &geom_fshader::Shader,
	render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
	dimensions: [u32; 2],
) -> (Arc<dyn GraphicsPipelineAbstract + Send + Sync>, Arc<dyn GraphicsPipelineAbstract + Send + Sync>) {
	(
		create_geom_pipeline(vshader, fshader, render_pass, dimensions, PrimitiveTopology::TriangleList),
		create_geom_pipeline(vshader, fshader, render_pass, dimensions, PrimitiveTopology::TriangleStrip),
	)
}

fn create_geom_pipeline(
	vshader: &geom_vshader::Shader,
	fshader: &geom_fshader::Shader,
	render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
	dimensions: [u32; 2],
	topology: PrimitiveTopology,
) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
	let dimensions = [dimensions[0] as f32, dimensions[1] as f32];
	let device = render_pass.device().clone();
	Arc::new(
		GraphicsPipeline::start()
			.vertex_input_single_buffer::<Pntl_32F>()
			.vertex_shader(vshader.main_entry_point(), ())
			.fragment_shader(fshader.main_entry_point(), ())
			.primitive_topology(topology)
			.cull_mode_back()
			.viewports(vec![Viewport { origin: [0.0, 0.0], dimensions, depth_range: 0.0..1.0 }])
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
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
			.render_pass(Subpass::from(render_pass, 2).unwrap())
			.build(device)
			.unwrap(),
	)
}

fn create_light_pipeline(
	vshader: &light_vshader::Shader,
	fshader: &light_fshader::Shader,
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
			.blend_collective(AttachmentBlend {
				enabled: true,
				color_op: BlendOp::Add,
				color_source: BlendFactor::One,
				color_destination: BlendFactor::One,
				alpha_op: BlendOp::Add,
				alpha_source: BlendFactor::One,
				alpha_destination: BlendFactor::One,
				mask_red: true,
				mask_green: true,
				mask_blue: true,
				mask_alpha: true,
			})
			.viewports(vec![Viewport { origin: [0.0, 0.0], dimensions, depth_range: 0.0..1.0 }])
			.render_pass(Subpass::from(render_pass, 1).unwrap())
			.build(device)
			.unwrap(),
	)
}

fn make_gbuffers_desc<L>(layout: L, gbuffers: &GBuffers) -> Arc<dyn DescriptorSet + Send + Sync>
where
	L: PipelineLayoutAbstract + Send + Sync + 'static,
{
	Arc::new(
		PersistentDescriptorSet::start(layout, 0)
			.add_image(gbuffers.depth.clone())
			.unwrap()
			.add_image(gbuffers.diffuse.clone())
			.unwrap()
			.add_image(gbuffers.normal.clone())
			.unwrap()
			.add_image(gbuffers.position.clone())
			.unwrap()
			.add_image(gbuffers.light.clone())
			.unwrap()
			.build()
			.unwrap(),
	)
}
