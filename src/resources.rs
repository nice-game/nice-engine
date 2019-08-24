mod model;
mod texture;

use crate::{
	mesh::Mesh,
	mesh_data::MeshData,
	mesh_group::MeshGroup,
	texture::{ImmutableTexture, Texture},
	threads::FILE_THREAD,
};
use atom::AtomSetOnce;
use futures::{future::lazy, task::SpawnExt};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::Arc,
};
use vulkano::{
	descriptor::PipelineLayoutAbstract,
	device::Queue,
	format::Format,
	image::ImageViewAccess,
	sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
	sync::GpuFuture,
};

pub struct Resources {
	queue: Arc<Queue>,
	layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	// TODO: move to Context
	sampler: Arc<Sampler>,
	// TODO: move to Context
	white_pixel: ImmutableTexture,
	meshes: HashMap<PathBuf, Arc<Model>>,
	textures: HashMap<PathBuf, Arc<TextureResource>>,
}
impl Resources {
	pub(crate) fn new(
		queue: Arc<Queue>,
		layout_desc: Arc<dyn PipelineLayoutAbstract + Send + Sync>,
	) -> (Self, impl GpuFuture) {
		let device = queue.device();

		let sampler = Sampler::new(
			device.clone(),
			Filter::Linear,
			Filter::Linear,
			MipmapMode::Linear,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			SamplerAddressMode::Repeat,
			0.0,
			if device.enabled_features().sampler_anisotropy { 16.0 } else { 1.0 },
			0.0,
			1337.0,
		)
		.unwrap();

		let (white_pixel, white_pixel_future) = ImmutableTexture::from_iter_vk(
			queue.clone(),
			vec![[255u8, 255, 255, 255]].into_iter(),
			[1, 1],
			Format::R8G8B8A8Unorm,
		)
		.unwrap();

		let meshes = HashMap::new();
		let textures = HashMap::new();
		(Self { queue, layout_desc, sampler, white_pixel, meshes, textures }, white_pixel_future)
	}

	pub fn get_model(&mut self, mesh_group: Arc<MeshGroup>, path: impl AsRef<Path> + Clone + Send + 'static) -> Mesh {
		let model = self.meshes.get(path.as_ref()).cloned().unwrap_or_else(|| {
			let (mesh_data, _mats, mesh_data_future) = model::from_nice_model(&self.queue, path.clone());
			mesh_data_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
			// let mats = mats
			// 	.into_iter()
			// 	.map(|mat| Material {
			// 		range: mat.range,
			// 		textures: [self.get_texture(mat.tex1), self.get_texture(mat.tex2)],
			// 	})
			// 	.collect();
			let model = Arc::new(Model { mesh_data /* mats */ });
			self.meshes.insert(path.as_ref().to_owned(), model.clone());
			model
		});

		let mesh =
			Mesh::new_inner(mesh_group, self.layout_desc.clone(), self.white_pixel.clone(), self.sampler.clone());
		{
			let mut mesh_inner = mesh.lock().unwrap();
			mesh_inner.set_mesh_data(Some(model.mesh_data.clone()));
			// mesh_inner.set_materials(&model.mats);
		}
		mesh
	}

	pub fn get_texture(&mut self, path: impl AsRef<Path> + Clone + Send + 'static) -> Arc<dyn Texture> {
		self.textures.get(path.as_ref()).cloned().unwrap_or_else(|| {
			let tex = Arc::new(TextureResource { tex: AtomSetOnce::empty(), white_pixel: self.white_pixel.clone() });
			load_tex(self.queue.clone(), tex.clone(), path.clone());
			self.textures.insert(path.as_ref().to_owned(), tex.clone());
			tex
		})
	}

	pub(crate) fn sampler(&self) -> &Arc<Sampler> {
		&self.sampler
	}

	pub(crate) fn white_pixel(&self) -> &ImmutableTexture {
		&self.white_pixel
	}
}

fn load_tex(queue: Arc<Queue>, res: Arc<TextureResource>, path: impl AsRef<Path> + Clone + Send + 'static) {
	FILE_THREAD
		.lock()
		.unwrap()
		.spawn(lazy(move |_| {
			let (tex, tex_future) = texture::from_nice_texture(&queue, path);
			tex_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
			res.tex.set_if_none(Arc::new(tex));
			log::debug!("loaded image");
		}))
		.unwrap();
}

struct Model {
	mesh_data: Arc<MeshData>,
	// mats: Vec<Material>,
}

struct TextureResource {
	tex: AtomSetOnce<Arc<ImmutableTexture>>,
	white_pixel: ImmutableTexture,
}
impl Texture for TextureResource {
	fn image(&self) -> &Arc<dyn ImageViewAccess + Send + Sync> {
		self.tex.get().unwrap_or(&self.white_pixel).image()
	}
}
