use crate::{mesh_data::MeshData, texture::Texture, transform::Transform, Context};
use std::sync::Arc;
use vulkano::{
	descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet, PipelineLayoutAbstract},
	image::ImageViewAccess,
	sampler::Sampler,
};

pub struct Mesh {
	ctx: Arc<Context>,
	mesh_data: Option<Arc<MeshData>>,
	transform: Transform,
	texture_desc: Arc<dyn DescriptorSet + Send + Sync>,
}
impl Mesh {
	pub fn new(ctx: Arc<Context>) -> Self {
		let texture_desc = make_desc_set(
			ctx.pipeline_ctx().layout_desc().clone(),
			ctx.white_pixel().image().clone(),
			ctx.sampler().clone(),
		);
		Self { ctx, mesh_data: None, transform: Transform::default(), texture_desc }
	}

	pub fn mesh_data(&self) -> &Option<Arc<MeshData>> {
		&self.mesh_data
	}

	pub fn set_mesh_data(&mut self, mesh_data: Option<Arc<MeshData>>) {
		self.mesh_data = mesh_data;
	}

	pub fn transform(&self) -> &Transform {
		&self.transform
	}

	pub fn transform_mut(&mut self) -> &mut Transform {
		&mut self.transform
	}

	pub fn texture_desc(&self) -> &Arc<dyn DescriptorSet + Send + Sync> {
		&self.texture_desc
	}

	pub fn set_texture(&mut self, texture: &Texture) {
		self.texture_desc = make_desc_set(
			self.ctx.pipeline_ctx().layout_desc().clone(),
			texture.image().clone(),
			self.ctx.sampler().clone(),
		);
	}
}

fn make_desc_set<L, T: ImageViewAccess>(
	layout: L,
	image_view: T,
	sampler: Arc<Sampler>,
) -> Arc<dyn DescriptorSet + Send + Sync>
where
	L: PipelineLayoutAbstract + Send + Sync + 'static,
	T: ImageViewAccess + Send + Sync + 'static,
{
	Arc::new(PersistentDescriptorSet::start(layout, 0).add_sampled_image(image_view, sampler).unwrap().build().unwrap())
}
