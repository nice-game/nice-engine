use crate::{mesh_data::MeshData, texture::Texture, transform::Transform};
use std::{ops::Range, sync::Arc};
use vulkano::{
	descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet, PipelineLayoutAbstract},
	image::ImageViewAccess,
	sampler::Sampler,
};

pub struct Mesh {
	layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	white_pixel: Texture,
	sampler: Arc<Sampler>,
	mesh_data: Option<Arc<MeshData>>,
	transform: Transform,
	texture_descs: Vec<MaterialDesc>,
}
impl Mesh {
	pub(crate) fn new(
		layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
		white_pixel: Texture,
		sampler: Arc<Sampler>,
	) -> Self {
		let texture_descs = vec![];
		Self { layout_desc, white_pixel, sampler, mesh_data: None, transform: Transform::default(), texture_descs }
	}

	pub fn mesh_data(&self) -> &Option<Arc<MeshData>> {
		&self.mesh_data
	}

	pub fn set_mesh_data(&mut self, mesh_data: Option<Arc<MeshData>>) {
		self.mesh_data = mesh_data;
		if let Some(data) = self.mesh_data.as_ref() {
			self.texture_descs = vec![MaterialDesc::new(
				0..data.indices().len(),
				make_desc_set(self.layout_desc.clone(), self.white_pixel.image().clone(), self.sampler.clone()),
				make_desc_set(self.layout_desc.clone(), self.white_pixel.image().clone(), self.sampler.clone()),
				0,
				0,
				0,
				[0; 3],
			)];
		}
	}

	pub fn transform(&self) -> &Transform {
		&self.transform
	}

	pub fn transform_mut(&mut self) -> &mut Transform {
		&mut self.transform
	}

	pub(crate) fn texture_descs(&self) -> &Vec<MaterialDesc> {
		&self.texture_descs
	}

	pub(crate) fn set_materials(&mut self, materials: Arc<Vec<Material>>) {
		self.texture_descs = materials
			.iter()
			.map(|mat| {
				MaterialDesc::new(
					mat.range.clone(),
					make_desc_set(self.layout_desc.clone(), mat.tex1.image().clone(), self.sampler.clone()),
					make_desc_set(self.layout_desc.clone(), mat.tex2.image().clone(), self.sampler.clone()),
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
	pub(crate) tex1: Texture,
	pub(crate) tex2: Texture,
	pub(crate) light_penetration: u8,
	pub(crate) subsurface_scattering: u8,
	pub(crate) emissive_brightness: u16,
	pub(crate) base_color: [u8; 3],
}
impl Material {
	pub(crate) fn new(
		range: Range<usize>,
		tex1: Texture,
		tex2: Texture,
		light_penetration: u8,
		subsurface_scattering: u8,
		emissive_brightness: u16,
		base_color: [u8; 3],
	) -> Self {
		Self { range, tex1, tex2, light_penetration, subsurface_scattering, emissive_brightness, base_color }
	}
}

pub(crate) struct MaterialDesc {
	pub(crate) range: Range<usize>,
	pub(crate) tex1: Arc<dyn DescriptorSet + Send + Sync>,
	#[allow(dead_code)]
	pub(crate) tex2: Arc<dyn DescriptorSet + Send + Sync>,
	#[allow(dead_code)]
	pub(crate) light_penetration: u8,
	#[allow(dead_code)]
	pub(crate) subsurface_scattering: u8,
	#[allow(dead_code)]
	pub(crate) emissive_brightness: u16,
	#[allow(dead_code)]
	pub(crate) base_color: [u8; 3],
}
impl MaterialDesc {
	pub(crate) fn new(
		range: Range<usize>,
		tex1: Arc<dyn DescriptorSet + Send + Sync>,
		tex2: Arc<dyn DescriptorSet + Send + Sync>,
		light_penetration: u8,
		subsurface_scattering: u8,
		emissive_brightness: u16,
		base_color: [u8; 3],
	) -> Self {
		Self { range, tex1, tex2, light_penetration, subsurface_scattering, emissive_brightness, base_color }
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
