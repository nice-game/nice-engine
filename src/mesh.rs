use crate::{mesh_data::MeshData, mesh_group::MeshGroup, texture::Texture, transform::Transform, Context};
use array_init::array_init;
use atom::Atom;
use log::trace;
use std::{
	ops::Range,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc, LockResult, Mutex, MutexGuard,
	},
};
use vulkano::{
	descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet, PipelineLayoutAbstract},
	image::ImageViewAccess,
	sampler::Sampler,
	VulkanObject,
};

const LAYERS: usize = 7;
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Mesh {
	id: usize,
	mesh_group: Arc<MeshGroup>,
	inner: Arc<Mutex<MeshInner>>,
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
		let descs = array_init(|_| make_desc_set(layout_desc.clone(), &textures, sampler.clone()));
		let inner = Arc::new(Mutex::new(MeshInner {
			layout_desc,
			sampler,
			mesh_data: Atom::empty(),
			range: 0..0,
			transform: Atom::empty(),
			textures,
			descs,
		}));

		let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
		mesh_group.meshes().lock().unwrap().insert(id, inner.clone());
		Self { id, mesh_group, inner }
	}

	pub fn lock(&self) -> LockResult<MutexGuard<MeshInner>> {
		self.inner.lock()
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
	range: Range<usize>,
	transform: Atom<Box<Transform>>,
	textures: [Arc<dyn Texture + Send + Sync + 'static>; LAYERS],
	descs: [Arc<dyn DescriptorSet + Send + Sync>; LAYERS],
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

	pub fn range(&self) -> Range<usize> {
		self.range.clone()
	}

	pub fn set_range(&mut self, range: Range<usize>) {
		self.range = range;
	}

	pub fn set_tex(&mut self, tex_i: usize, tex: Arc<dyn Texture + Send + Sync>) {
		self.textures[tex_i] = tex;
		self.descs[tex_i] = make_desc_set(self.layout_desc.clone(), &self.textures, self.sampler.clone());
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
	pub(crate) fn clone_transform(&self) -> Transform {
		let tmp = self.transform.take().unwrap();
		let ret = *tmp;
		self.transform.set_if_none(tmp);
		ret
	}

	pub(crate) fn refresh(&mut self) {
		for i in 0..2 {
			let lhs_id = self.textures[i].image().inner().internal_object();
			let rhs_id = self.descs[i].image(0).unwrap().0.inner().internal_object();
			if lhs_id != rhs_id {
				self.descs[i] = make_desc_set(self.layout_desc.clone(), &self.textures, self.sampler.clone());
			}
		}
	}

	pub(crate) fn descs(&self) -> &[Arc<dyn DescriptorSet + Send + Sync>; LAYERS] {
		&self.descs
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
