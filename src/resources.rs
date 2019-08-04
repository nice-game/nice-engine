pub mod model;

use crate::{
	mesh::{Material, Mesh},
	mesh_data::MeshData,
	texture::Texture,
};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::Arc,
};
use vulkano::{
	descriptor::PipelineLayoutAbstract,
	device::Queue,
	format::Format,
	sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
	sync::GpuFuture,
};

pub struct Resources {
	queue: Arc<Queue>,
	layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	sampler: Arc<Sampler>,
	white_pixel: Texture,
	meshes: HashMap<PathBuf, Model>,
}
impl Resources {
	pub(crate) fn new(
		queue: Arc<Queue>,
		layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	) -> (Self, impl GpuFuture) {
		let sampler = Sampler::new(
			queue.device().clone(),
			Filter::Linear,
			Filter::Linear,
			MipmapMode::Nearest,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			0.0,
			1.0,
			0.0,
			0.0,
		)
		.unwrap();

		let (white_pixel, white_pixel_future) = Texture::from_iter_vk(
			queue.clone(),
			vec![[255u8, 255, 255, 255]].into_iter(),
			[1, 1],
			Format::R8G8B8A8Unorm,
		)
		.unwrap();

		(Self { queue, layout_desc, sampler, white_pixel, meshes: HashMap::new() }, white_pixel_future)
	}

	pub fn get_model(&mut self, path: impl AsRef<Path> + Clone + Send + 'static) -> Mesh {
		let queue = self.queue.clone();
		let model = self.meshes.entry(path.as_ref().to_owned()).or_insert_with(|| {
			let (mesh_data, mats, mesh_data_future) = model::from_nice_model(&queue, path);
			mesh_data_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
			Model { mesh_data, mats }
		});

		let mut mesh = Mesh::new(self.layout_desc.clone(), self.white_pixel.clone(), self.sampler.clone());
		mesh.set_mesh_data(Some(model.mesh_data.clone()));
		mesh.set_materials(model.mats.clone());
		mesh
	}
}

struct Model {
	mesh_data: Arc<MeshData>,
	mats: Arc<Vec<Material>>,
}
