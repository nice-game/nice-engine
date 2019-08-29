use crate::{mesh_data::MeshData, mesh_group::MeshGroup, texture::Texture, transform::Transform, Context};
use array_init::array_init;
use log::trace;
use std::{
	ops::Range,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc, RwLock,
	},
};
use vulkano::{
	descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet, PipelineLayoutAbstract},
	sampler::Sampler,
};

const LAYERS: usize = 7;
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Mesh {
	id: usize,
	mesh_group: Arc<MeshGroup>,
	inner: Arc<RwLock<MeshInner>>,
}
impl Mesh {
	pub fn new(ctx: &Context, mesh_group: Arc<MeshGroup>) -> Self {
		trace!("Mesh::new");

		let layout_desc = ctx.pipeline_ctx().layout_desc().clone();
		let resources = ctx.resources();
		let sampler = resources.sampler().clone();
		Self::new_inner(mesh_group, layout_desc, resources.white_pixel(), sampler)
	}

	pub(crate) fn new_inner(
		mesh_group: Arc<MeshGroup>,
		layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
		white_pixel: &Arc<dyn Texture + Send + Sync>,
		sampler: Arc<Sampler>,
	) -> Self {
		let textures = array_init(|_| white_pixel.clone() as _);
		let desc = make_desc_set(layout_desc.clone(), &textures, sampler.clone());
		let inner = Arc::new(RwLock::new(MeshInner {
			layout_desc,
			sampler,
			mesh_data: None,
			range: 0..0,
			transform: Transform::default(),
			textures,
			desc,
		}));

		let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
		mesh_group.meshes().lock().unwrap().insert(id, inner.clone());
		Self { id, mesh_group, inner }
	}

	pub fn inner(&self) -> &Arc<RwLock<MeshInner>> {
		&self.inner
	}
}
impl Drop for Mesh {
	fn drop(&mut self) {
		self.mesh_group.meshes().lock().unwrap().remove(&self.id);
	}
}

pub struct MeshInner {
	layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	sampler: Arc<Sampler>,
	mesh_data: Option<Arc<MeshData>>,
	range: Range<usize>,
	transform: Transform,
	textures: [Arc<dyn Texture + Send + Sync + 'static>; LAYERS],
	desc: Arc<dyn DescriptorSet + Send + Sync>,
}
impl MeshInner {
	pub fn transform(&self) -> &Transform {
		&self.transform
	}

	pub fn set_transform(&mut self, transform: Transform) {
		self.transform = transform;
	}

	pub fn mesh_data(&self) -> Option<&Arc<MeshData>> {
		self.mesh_data.as_ref()
	}

	pub fn set_mesh_data(&mut self, mesh_data: Option<Arc<MeshData>>) {
		self.mesh_data = mesh_data;
	}

	pub fn range(&self) -> Range<usize> {
		self.range.clone()
	}

	pub fn set_range(&mut self, range: Range<usize>) {
		self.range = range;
	}

	pub fn desc(&self) -> &Arc<dyn DescriptorSet + Send + Sync> {
		&self.desc
	}

	pub fn set_tex(&mut self, tex_i: usize, tex: Arc<dyn Texture + Send + Sync>) {
		self.textures[tex_i] = tex;
		self.desc = make_desc_set(self.layout_desc.clone(), &self.textures, self.sampler.clone());
	}
}

fn make_desc_set<L>(
	layout: L,
	image_views: &[Arc<dyn Texture + Send + Sync>; LAYERS],
	sampler: Arc<Sampler>,
) -> Arc<dyn DescriptorSet + Send + Sync>
where
	L: PipelineLayoutAbstract + Send + Sync + 'static,
{
	Arc::new(
		PersistentDescriptorSet::start(layout, 0)
			.add_sampled_image(image_views[0].image().clone(), sampler.clone())
			.unwrap()
			.add_sampled_image(image_views[1].image().clone(), sampler.clone())
			.unwrap()
			.add_sampled_image(image_views[2].image().clone(), sampler.clone())
			.unwrap()
			.add_sampled_image(image_views[3].image().clone(), sampler.clone())
			.unwrap()
			.add_sampled_image(image_views[4].image().clone(), sampler.clone())
			.unwrap()
			.add_sampled_image(image_views[5].image().clone(), sampler.clone())
			.unwrap()
			.add_sampled_image(image_views[6].image().clone(), sampler.clone())
			.unwrap()
			.build()
			.unwrap(),
	)
}
