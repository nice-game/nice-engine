use crate::{
	mesh_data::MeshData,
	mesh_group::MeshGroup,
	texture::{ImmutableTexture, Texture},
	transform::Transform,
	Context,
};
use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Arc, LockResult, Mutex, MutexGuard,
};
use vulkano::{
	descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet, PipelineLayoutAbstract},
	image::ImageViewAccess,
	sampler::Sampler,
	VulkanObject,
};

const LAYERS: usize = 3;
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Mesh {
	id: usize,
	mesh_group: Arc<MeshGroup>,
	inner: Arc<Mutex<MeshInner>>,
}
impl Mesh {
	pub fn new(ctx: &Context, mesh_group: Arc<MeshGroup>) -> Self {
		let layout_desc = ctx.pipeline_ctx().layout_desc().clone();
		let white_pixel = ctx.resources().lock().unwrap().white_pixel().clone();
		let sampler = ctx.resources().lock().unwrap().sampler().clone();
		Self::new_inner(mesh_group, layout_desc, white_pixel, sampler)
	}

	pub(crate) fn new_inner(
		mesh_group: Arc<MeshGroup>,
		layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
		white_pixel: ImmutableTexture,
		sampler: Arc<Sampler>,
	) -> Self {
		let textures = [
			Arc::new(white_pixel.clone()) as _,
			Arc::new(white_pixel.clone()) as _,
			Arc::new(white_pixel.clone()) as _,
		];
		let descs = [
			make_desc_set(layout_desc.clone(), white_pixel.image().clone(), sampler.clone()),
			make_desc_set(layout_desc.clone(), white_pixel.image().clone(), sampler.clone()),
			make_desc_set(layout_desc.clone(), white_pixel.image().clone(), sampler.clone()),
		];
		let inner = Arc::new(Mutex::new(MeshInner {
			layout_desc,
			sampler,
			mesh_data: None,
			transform: Transform::default(),
			textures,
			descs,
		}));

		let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
		println!("{}", id);
		mesh_group.lock().unwrap().insert(id, inner.clone());
		Self { id, mesh_group, inner }
	}

	pub fn lock(&self) -> LockResult<MutexGuard<MeshInner>> {
		self.inner.lock()
	}
}
impl Drop for Mesh {
	fn drop(&mut self) {
		self.mesh_group.lock().unwrap().remove(&self.id);
	}
}

pub struct MeshInner {
	layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	sampler: Arc<Sampler>,
	mesh_data: Option<Arc<MeshData>>,
	transform: Transform,
	textures: [Arc<dyn Texture>; LAYERS],
	descs: [Arc<dyn DescriptorSet + Send + Sync>; LAYERS],
}
impl MeshInner {
	pub fn transform(&self) -> &Transform {
		&self.transform
	}

	pub fn transform_mut(&mut self) -> &mut Transform {
		&mut self.transform
	}

	pub fn mesh_data(&self) -> Option<&Arc<MeshData>> {
		self.mesh_data.as_ref()
	}

	pub fn set_mesh_data(&mut self, mesh_data: Option<Arc<MeshData>>) {
		self.mesh_data = mesh_data;
	}

	pub fn set_tex(&mut self, tex_i: usize, tex: Arc<dyn Texture + Send + Sync>) {
		self.descs[tex_i] = make_desc_set(self.layout_desc.clone(), tex.image().clone(), self.sampler.clone());
		self.textures[tex_i] = tex;
	}

	pub(crate) fn refresh(&mut self) {
		for i in 0..2 {
			let lhs_id = self.textures[i].image().inner().internal_object();
			let rhs_id = self.descs[i].image(0).unwrap().0.inner().internal_object();
			if lhs_id != rhs_id {
				self.descs[i] =
					make_desc_set(self.layout_desc.clone(), self.textures[i].image().clone(), self.sampler.clone());
			}
		}
	}

	pub(crate) fn descs(&self) -> &[Arc<dyn DescriptorSet + Send + Sync>; LAYERS] {
		&self.descs
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
	Arc::new(
		PersistentDescriptorSet::start(layout, 0)
			.add_sampled_image(image_view, sampler.clone())
			.unwrap()
			.build()
			.unwrap(),
	)
}
