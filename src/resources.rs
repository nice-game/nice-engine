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
	ops::Range,
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
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
	sampler: Arc<Sampler>,
	white_pixel: Arc<dyn Texture + Send + Sync>,
	meshes: Mutex<HashMap<PathBuf, Arc<Model>>>,
	textures: Mutex<HashMap<PathBuf, Arc<TextureResource>>>,
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
		let white_pixel = Arc::new(white_pixel);

		let meshes = Mutex::default();
		let textures = Mutex::default();
		(Self { queue, layout_desc, sampler, white_pixel, meshes, textures }, white_pixel_future)
	}

	pub fn get_model(&self, mesh_group: Arc<MeshGroup>, path: impl AsRef<Path> + Clone + Send + 'static) -> Vec<Mesh> {
		let path = path.as_ref();
		let model = self.meshes.lock().unwrap().get(path).cloned();
		let model = model.unwrap_or_else(|| {
			let (mesh_data, mats, mesh_data_future) = model::from_nice_model(&self.queue, path.clone());
			mesh_data_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
			let mats = mats
				.into_iter()
				.map(|mat| Material {
					range: mat.range,
					textures: [self.get_texture(mat.tex1), self.get_texture(mat.tex2)],
				})
				.collect();
			let model = Arc::new(Model { mesh_data, mats });
			self.meshes.lock().unwrap().insert(path.to_owned(), model.clone());
			model
		});

		model.mats.iter().map(|mat| {
			let mesh = Mesh::new_inner(mesh_group.clone(), self.layout_desc.clone(), &self.white_pixel, self.sampler.clone());
			{
				let mut mesh_inner = mesh.inner().write().unwrap();
				mesh_inner.set_mesh_data(Some(model.mesh_data.clone()));
				mesh_inner.set_range(mat.range.clone());
				mesh_inner.set_tex(0, mat.textures[0].clone());
				mesh_inner.set_tex(1, mat.textures[1].clone());
			}
			mesh
		}).collect()
	}

	pub fn get_texture(&self, path: impl AsRef<Path> + Clone + Send + 'static) -> Arc<dyn Texture + Send + Sync> {
		let tex = self.textures.lock().unwrap().get(path.as_ref()).cloned();
		tex.unwrap_or_else(|| {
			let tex = TextureResource::new(self.white_pixel.clone());
			load_tex(self.queue.clone(), tex.clone(), path.clone());
			self.textures.lock().unwrap().insert(path.as_ref().to_owned(), tex.clone());
			tex
		})
	}

	pub fn white_pixel(&self) -> &Arc<dyn Texture + Send + Sync> {
		&self.white_pixel
	}

	pub(crate) fn sampler(&self) -> &Arc<Sampler> {
		&self.sampler
	}
}

fn load_tex(queue: Arc<Queue>, res: Arc<TextureResource>, path: impl AsRef<Path> + Clone + Send + 'static) {
	FILE_THREAD
		.lock()
		.unwrap()
		.spawn(lazy(move |_| {
			let (tex, tex_future) = texture::from_nice_texture(&queue, path);
			tex_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
			res.tex.set_if_none(Box::new(tex));
			log::debug!("loaded image");
		}))
		.unwrap();
}

struct Model {
	mesh_data: Arc<MeshData>,
	mats: Vec<Material>,
}

struct Material {
	range: Range<usize>,
	textures: [Arc<dyn Texture + Send + Sync>; 2],
}

pub struct TextureResource {
	tex: AtomSetOnce<Box<Arc<dyn Texture + Send + Sync>>>,
	white_pixel: Arc<dyn Texture + Send + Sync>,
}
impl TextureResource {
	pub fn new(white_pixel: Arc<dyn Texture + Send + Sync>) -> Arc<Self> {
		Arc::new(Self { tex: AtomSetOnce::empty(), white_pixel })
	}

	pub fn set_texture(&self, tex: Arc<dyn Texture + Send + Sync>) -> Option<Arc<dyn Texture + Send + Sync>> {
		self.tex.set_if_none(Box::new(tex)).map(|tex| *tex)
	}
}
impl Texture for TextureResource {
	fn image(&self) -> &Arc<dyn ImageViewAccess + Send + Sync> {
		self.tex.get().unwrap_or(&self.white_pixel).image()
	}
}
