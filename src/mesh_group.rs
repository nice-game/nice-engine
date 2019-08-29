use crate::{mesh::MeshInner, texture::Texture, Context};
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use vulkano::{
	descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet, PipelineLayoutAbstract},
	sampler::Sampler,
};

pub struct MeshGroup {
	meshes: Mutex<HashMap<usize, Arc<MeshInner>>>,
	skybox: Mutex<Arc<dyn DescriptorSet + Send + Sync>>,
	swap_layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	white_pixel: Arc<dyn Texture + Send + Sync>,
	sampler: Arc<Sampler>,
}
impl MeshGroup {
	pub fn new(ctx: &Context) -> Arc<Self> {
		let resources = ctx.resources();
		let swap_layout_desc = ctx.pipeline_ctx().swap_layout_desc().clone();
		let white_pixel = resources.white_pixel().clone();
		let sampler = resources.sampler().clone();
		let skybox = Mutex::new(make_desc_set(swap_layout_desc.clone(), &white_pixel, sampler.clone()));
		Arc::new(Self { meshes: Mutex::default(), skybox, swap_layout_desc, white_pixel, sampler })
	}

	pub fn set_skybox(&self, skybox: Option<&Arc<dyn Texture + Send + Sync>>) {
		*self.skybox.lock().unwrap() =
			make_desc_set(self.swap_layout_desc.clone(), skybox.unwrap_or(&self.white_pixel), self.sampler.clone());
	}

	pub(crate) fn meshes(&self) -> &Mutex<HashMap<usize, Arc<MeshInner>>> {
		&self.meshes
	}

	pub(crate) fn skybox(&self) -> &Mutex<Arc<dyn DescriptorSet + Send + Sync>> {
		&self.skybox
	}
}

fn make_desc_set<L>(
	layout: L,
	image_view: &Arc<dyn Texture + Send + Sync>,
	sampler: Arc<Sampler>,
) -> Arc<dyn DescriptorSet + Send + Sync>
where
	L: PipelineLayoutAbstract + Send + Sync + 'static,
{
	Arc::new(
		PersistentDescriptorSet::start(layout, 1)
			.add_sampled_image(image_view.image().clone(), sampler.clone())
			.unwrap()
			.build()
			.unwrap(),
	)
}
