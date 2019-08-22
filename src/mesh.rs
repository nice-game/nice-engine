use crate::{
	mesh_data::MeshData,
	mesh_group::MeshGroup,
	texture::{ImmutableTexture, TargetTexture, Texture},
	transform::Transform,
	Context,
};
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

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Mesh {
	id: usize,
	mesh_group: Arc<MeshGroup>,
	inner: Arc<Mutex<MeshInner>>,
}
impl Mesh {
	pub fn new(ctx: &Context, mesh_group: &Arc<MeshGroup>) -> Self {
		let layout_desc = ctx.pipeline_ctx().layout_desc().clone();
		let white_pixel = ctx.resources().lock().unwrap().white_pixel().clone();
		let sampler = ctx.resources().lock().unwrap().sampler().clone();
		Self::new_inner(mesh_group, layout_desc, white_pixel, sampler)
	}

	pub(crate) fn new_inner(
		mesh_group: &Arc<MeshGroup>,
		layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
		white_pixel: ImmutableTexture,
		sampler: Arc<Sampler>,
	) -> Self {
		let mesh_group = mesh_group.clone();
		let texture_descs = vec![];
		let inner = Arc::new(Mutex::new(MeshInner {
			layout_desc,
			white_pixel,
			sampler,
			mesh_data: None,
			transform: Transform::default(),
			texture_descs,
			lightmap: None,
		}));

		let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
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
	white_pixel: ImmutableTexture,
	sampler: Arc<Sampler>,
	mesh_data: Option<Arc<MeshData>>,
	transform: Transform,
	texture_descs: Vec<MaterialDesc>,
	lightmap: Option<TargetTexture>,
}
impl MeshInner {
	pub fn transform(&self) -> &Transform {
		&self.transform
	}

	pub fn transform_mut(&mut self) -> &mut Transform {
		&mut self.transform
	}

	pub(crate) fn mesh_data(&self) -> &Option<Arc<MeshData>> {
		&self.mesh_data
	}

	pub(crate) fn refresh(&mut self) {
		for mat in &mut self.texture_descs {
			if mat.tex1_tex.image().inner().internal_object() != mat.tex1.image(0).unwrap().0.inner().internal_object()
			{
				mat.tex1 = make_desc_set(
					self.layout_desc.clone(),
					mat.tex1_tex.image().clone(),
					self.lightmap.as_ref(),
					self.sampler.clone(),
				);
			}
		}
	}

	pub(crate) fn set_mesh_data(&mut self, mesh_data: Option<Arc<MeshData>>) {
		self.mesh_data = mesh_data;
		if let Some(data) = self.mesh_data.as_ref() {
			self.texture_descs = vec![MaterialDesc::new(
				0..data.indices().len(),
				Arc::new(self.white_pixel.clone()),
				Arc::new(self.white_pixel.clone()),
				make_desc_set(
					self.layout_desc.clone(),
					self.white_pixel.image().clone(),
					self.lightmap.as_ref(),
					self.sampler.clone(),
				),
				make_desc_set(
					self.layout_desc.clone(),
					self.white_pixel.image().clone(),
					self.lightmap.as_ref(),
					self.sampler.clone(),
				),
				0,
				0,
				0,
				[0; 3],
			)];
		}
	}

	pub(crate) fn texture_descs(&self) -> &Vec<MaterialDesc> {
		&self.texture_descs
	}

	pub(crate) fn set_materials(&mut self, materials: &[Material]) {
		self.texture_descs = materials
			.iter()
			.map(|mat| {
				MaterialDesc::new(
					mat.range.clone(),
					mat.tex1.clone(),
					mat.tex2.clone(),
					make_desc_set(
						self.layout_desc.clone(),
						mat.tex1.image().clone(),
						self.lightmap.as_ref(),
						self.sampler.clone(),
					),
					make_desc_set(
						self.layout_desc.clone(),
						mat.tex2.image().clone(),
						self.lightmap.as_ref(),
						self.sampler.clone(),
					),
					mat.light_penetration,
					mat.subsurface_scattering,
					mat.emissive_brightness,
					mat.base_color,
				)
			})
			.collect();
	}
}

pub(crate) struct Material {
	pub(crate) range: Range<usize>,
	pub(crate) tex1: Arc<dyn Texture>,
	pub(crate) tex2: Arc<dyn Texture>,
	pub(crate) light_penetration: u8,
	pub(crate) subsurface_scattering: u8,
	pub(crate) emissive_brightness: u16,
	pub(crate) base_color: [u8; 3],
}

pub(crate) struct MaterialDesc {
	range: Range<usize>,
	tex1_tex: Arc<dyn Texture>,
	#[allow(dead_code)]
	tex2_tex: Arc<dyn Texture>,
	tex1: Arc<dyn DescriptorSet + Send + Sync>,
	#[allow(dead_code)]
	tex2: Arc<dyn DescriptorSet + Send + Sync>,
	#[allow(dead_code)]
	light_penetration: u8,
	#[allow(dead_code)]
	subsurface_scattering: u8,
	#[allow(dead_code)]
	emissive_brightness: u16,
	#[allow(dead_code)]
	base_color: [u8; 3],
}
impl MaterialDesc {
	fn new(
		range: Range<usize>,
		tex1_tex: Arc<dyn Texture>,
		tex2_tex: Arc<dyn Texture>,
		tex1: Arc<dyn DescriptorSet + Send + Sync>,
		tex2: Arc<dyn DescriptorSet + Send + Sync>,
		light_penetration: u8,
		subsurface_scattering: u8,
		emissive_brightness: u16,
		base_color: [u8; 3],
	) -> Self {
		Self {
			range,
			tex1_tex,
			tex2_tex,
			tex1,
			tex2,
			light_penetration,
			subsurface_scattering,
			emissive_brightness,
			base_color,
		}
	}

	pub(crate) fn range(&self) -> &Range<usize> {
		&self.range
	}

	pub(crate) fn tex1(&self) -> &Arc<dyn DescriptorSet + Send + Sync> {
		&self.tex1
	}

	#[allow(dead_code)]
	pub(crate) fn tex2(&self) -> &Arc<dyn DescriptorSet + Send + Sync> {
		&self.tex2
	}
}

fn make_desc_set<L, T: ImageViewAccess>(
	layout: L,
	image_view: T,
	lightmap: Option<&TargetTexture>,
	sampler: Arc<Sampler>,
) -> Arc<dyn DescriptorSet + Send + Sync>
where
	L: PipelineLayoutAbstract + Send + Sync + 'static,
	T: ImageViewAccess + Send + Sync + 'static,
{
	Arc::new(
		PersistentDescriptorSet::start(layout, 0)
		.add_sampled_image(image_view, sampler.clone()).unwrap()
//		.add_sampled_image(lightmap.unwrap().image().clone(), sampler.clone()).unwrap()
		.build().unwrap(),
	)
}
