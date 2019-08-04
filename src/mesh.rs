use crate::{mesh_data::MeshData, texture::Texture, transform::Transform, Context};
use std::{ops::Range, sync::Arc};
use vulkano::{
	descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet, PipelineLayoutAbstract},
	image::ImageViewAccess,
	sampler::Sampler,
};

pub struct Mesh {
	ctx: Arc<Context>,
	mesh_data: Option<Arc<MeshData>>,
	transform: Transform,
	texture_descs: Vec<(Material, Arc<dyn DescriptorSet + Send + Sync>)>,
}
impl Mesh {
	pub fn new(ctx: Arc<Context>) -> Self {
		let texture_descs = vec![];
		Self { ctx, mesh_data: None, transform: Transform::default(), texture_descs }
	}

	pub fn mesh_data(&self) -> &Option<Arc<MeshData>> {
		&self.mesh_data
	}

	pub fn set_mesh_data(&mut self, mesh_data: Option<Arc<MeshData>>) {
		self.mesh_data = mesh_data;
		if let Some(data) = self.mesh_data.as_ref() {
			self.texture_descs = vec![(
				Material::new(0..data.indices().len(), self.ctx.white_pixel().clone(), 0, 0, 0, [0; 3]),
				make_desc_set(
					self.ctx.pipeline_ctx().layout_desc().clone(),
					self.ctx.white_pixel().image().clone(),
					self.ctx.sampler().clone(),
				),
			)];
		}
	}

	pub fn transform(&self) -> &Transform {
		&self.transform
	}

	pub fn transform_mut(&mut self) -> &mut Transform {
		&mut self.transform
	}

	pub fn texture_descs(&self) -> &Vec<(Material, Arc<dyn DescriptorSet + Send + Sync>)> {
		&self.texture_descs
	}

	pub fn set_materials(&mut self, materials: Vec<Material>) {
		self.texture_descs = materials
			.into_iter()
			.map(|mat| {
				let image = mat.tex1.image().clone();
				(mat, make_desc_set(self.ctx.pipeline_ctx().layout_desc().clone(), image, self.ctx.sampler().clone()))
			})
			.collect();
	}
}

pub struct Material {
	pub(crate) range: Range<usize>,
	pub(crate) tex1: Texture,
	// pub(crate) tex2: Texture,
	pub(crate) light_penetration: u8,
	pub(crate) subsurface_scattering: u8,
	pub(crate) emissive_brightness: u16,
	pub(crate) base_color: [u8; 3],
}
impl Material {
	pub(crate) fn new(
		range: Range<usize>,
		tex1: Texture,
		// tex2: Texture,
		light_penetration: u8,
		subsurface_scattering: u8,
		emissive_brightness: u16,
		base_color: [u8; 3],
	) -> Self {
		Self { range, tex1, light_penetration, subsurface_scattering, emissive_brightness, base_color }
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
