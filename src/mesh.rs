use crate::{mesh_data::MeshData, mesh_group::MeshGroup, texture::Texture, transform::Transform, Context};
use array_init::array_init;
use atom::Atom;
use log::trace;
use std::{
	ops::Range,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc, Mutex,
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
	inner: Arc<MeshInner>,
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
		let desc = Atom::new(Box::new(make_desc_set(layout_desc.clone(), &textures, sampler.clone())));
		let textures = Mutex::new(textures);
		let inner = Arc::new(MeshInner {
			layout_desc,
			sampler,
			mesh_data: Atom::empty(),
			range: Atom::new(Box::new(0..0)),
			transform: Atom::empty(),
			textures,
			desc,
		});

		let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
		mesh_group.meshes().lock().unwrap().insert(id, inner.clone());
		Self { id, mesh_group, inner }
	}

	pub fn inner(&self) -> &Arc<MeshInner> {
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
	mesh_data: Atom<Arc<MeshData>>,
	range: Atom<Box<Range<usize>>>,
	transform: Atom<Box<Transform>>,
	textures: Mutex<[Arc<dyn Texture + Send + Sync + 'static>; LAYERS]>,
	desc: Atom<Box<Arc<dyn DescriptorSet + Send + Sync>>>,
}
impl MeshInner {
	pub fn set_transform(&self, transform: Transform) {
		self.transform.swap(Box::new(transform));
	}

	pub fn set_mesh_data(&self, mesh_data: Option<Arc<MeshData>>) {
		if let Some(mesh_data) = mesh_data {
			self.mesh_data.swap(mesh_data);
		} else {
			self.mesh_data.take();
		}
	}

	pub fn set_range(&self, range: Range<usize>) {
		self.range.swap(Box::new(range));
	}

	pub fn set_tex(&self, tex_i: usize, tex: Arc<dyn Texture + Send + Sync>) {
		let mut textures = self.textures.lock().unwrap();
		textures[tex_i] = tex;
		self.desc.swap(Box::new(make_desc_set(self.layout_desc.clone(), &textures, self.sampler.clone())));
	}

	/// May panic if called on multiple threads
	pub(crate) fn clone_desc(&self) -> Arc<dyn DescriptorSet + Send + Sync> {
		let tmp = self.desc.take().unwrap();
		let ret = (*tmp).clone();
		self.desc.set_if_none(tmp);
		ret
	}

	/// May panic if called on multiple threads
	pub(crate) fn clone_mesh_data(&self) -> Option<Arc<MeshData>> {
		self.mesh_data.take().map(|tmp| {
			let ret = tmp.clone();
			self.mesh_data.set_if_none(tmp);
			ret
		})
	}

	/// May panic if called on multiple threads
	pub(crate) fn clone_range(&self) -> Range<usize> {
		let tmp = self.range.take().unwrap();
		let ret = (*tmp).clone();
		self.range.set_if_none(tmp);
		ret
	}

	/// May panic if called on multiple threads
	pub(crate) fn clone_transform(&self) -> Transform {
		let tmp = self.transform.take().unwrap();
		let ret = *tmp;
		self.transform.set_if_none(tmp);
		ret
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
