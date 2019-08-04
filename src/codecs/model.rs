use crate::{
	mesh::Material,
	mesh_data::{MeshData, Pntl_32F},
	texture::Texture,
	Context,
};
use byteorder::{ReadBytesExt, LE};
use image::{GenericImageView, ImageDecoder};
use log::debug;
use std::{
	fs::File,
	io::{prelude::*, BufReader, SeekFrom},
	mem::drop,
	ops::Range,
	path::{Path, PathBuf},
	sync::Arc,
};
use vulkano::{
	buffer::{BufferUsage, CpuAccessibleBuffer, ImmutableBuffer},
	format::Format,
	sync::{self, GpuFuture},
};

pub fn from_nice_model(
	ctx: &Context,
	path: impl AsRef<Path> + Clone + Send + 'static,
) -> (Arc<MeshData>, Vec<Material>, impl GpuFuture + Send + Sync + 'static) {
	let queue = ctx.queue();
	let device = queue.device();

	let mut file = File::open(path.clone()).unwrap();

	let mut magic_number = [0; 4];
	file.read_exact(&mut magic_number).unwrap();
	assert_eq!(&magic_number, b"nmdl");

	// skip version for now
	file.seek(SeekFrom::Current(4)).unwrap();

	let vertex_count = file.read_u32::<LE>().unwrap() as usize;
	let positions_offset = file.read_u32::<LE>().unwrap() as u64;
	let normals_offset = file.read_u32::<LE>().unwrap() as u64;
	let texcoords_main_offset = file.read_u32::<LE>().unwrap() as u64;
	let texcoords_lightmap_offset = file.read_u32::<LE>().unwrap() as u64;
	let index_count = file.read_u32::<LE>().unwrap() as usize;
	let indices_offset = file.read_u32::<LE>().unwrap() as u64;
	let material_count = file.read_u8().unwrap() as usize;
	let materials_offset = file.read_u32::<LE>().unwrap() as u64;

	println!("vertex_count: {}", vertex_count);
	println!("positions_offset: {}", positions_offset);
	println!("normals_offset: {}", normals_offset);
	println!("texcoords_main_offset: {}", texcoords_main_offset);
	println!("texcoords_lightmap_offset: {}", texcoords_lightmap_offset);
	println!("index_count: {}", index_count);
	println!("indices_offset: {}", indices_offset);
	println!("material_count: {}", material_count);
	println!("materials_offset: {}", materials_offset);

	let tmpbuf: Arc<CpuAccessibleBuffer<[Pntl_32F]>> = unsafe {
		CpuAccessibleBuffer::uninitialized_array(device.clone(), vertex_count, BufferUsage::transfer_source()).unwrap()
	};
	let mut tmpbuf_lock = tmpbuf.write().unwrap();

	file.seek(SeekFrom::Start(positions_offset)).unwrap();
	for i in 0..vertex_count {
		tmpbuf_lock[i].pos =
			[file.read_f32::<LE>().unwrap(), file.read_f32::<LE>().unwrap(), file.read_f32::<LE>().unwrap()];
	}

	file.seek(SeekFrom::Start(normals_offset)).unwrap();
	for i in 0..vertex_count {
		tmpbuf_lock[i].nor =
			[file.read_f32::<LE>().unwrap(), file.read_f32::<LE>().unwrap(), file.read_f32::<LE>().unwrap()];
	}

	file.seek(SeekFrom::Start(texcoords_main_offset)).unwrap();
	for i in 0..vertex_count {
		tmpbuf_lock[i].texc = [file.read_f32::<LE>().unwrap(), file.read_f32::<LE>().unwrap()];
	}

	file.seek(SeekFrom::Start(texcoords_lightmap_offset)).unwrap();
	for i in 0..vertex_count {
		tmpbuf_lock[i].lmap = [file.read_f32::<LE>().unwrap(), file.read_f32::<LE>().unwrap()];
	}

	drop(tmpbuf_lock);

	let (vertices, vertices_future) =
		ImmutableBuffer::from_buffer(tmpbuf, BufferUsage::vertex_buffer(), queue.clone()).unwrap();

	let tmpbuf: Arc<CpuAccessibleBuffer<[u32]>> = unsafe {
		CpuAccessibleBuffer::uninitialized_array(device.clone(), index_count, BufferUsage::transfer_source()).unwrap()
	};
	let mut tmpbuf_lock = tmpbuf.write().unwrap();

	file.seek(SeekFrom::Start(indices_offset)).unwrap();
	for i in 0..index_count {
		tmpbuf_lock[i] = file.read_u32::<LE>().unwrap();
	}

	drop(tmpbuf_lock);

	let (indices, indices_future) =
		ImmutableBuffer::from_buffer(tmpbuf, BufferUsage::index_buffer(), queue.clone()).unwrap();

	file.seek(SeekFrom::Start(materials_offset)).unwrap();
	let mut index = 0;
	let mut mat_infos = vec![];
	for _ in 0..material_count {
		let index_count = file.read_u32::<LE>().unwrap() as usize;
		let nextindex = index + index_count;

		mat_infos.push(MatInfo {
			range: index..nextindex,
			texture1_name_size: file.read_u16::<LE>().unwrap(),
			texture1_name_offset: file.read_u32::<LE>().unwrap(),
			texture2_name_size: file.read_u16::<LE>().unwrap(),
			texture2_name_offset: file.read_u32::<LE>().unwrap(),
			light_penetration: file.read_u8().unwrap(),
			subsurface_scattering: file.read_u8().unwrap(),
			emissive_brightness: file.read_u16::<LE>().unwrap(),
			base_color: [file.read_u8().unwrap(), file.read_u8().unwrap(), file.read_u8().unwrap()],
		});

		index = nextindex;
	}

	let mut import_tex = |path: PathBuf| {
		let img = image::open(path).unwrap();
		println!(" => loader: image importer");
		let (width, height) = img.dimensions();
		println!(" => resolution: {}x{}", width, height);
		Some(Texture::from_iter(
			ctx,
			img.to_rgba().into_raw().into_iter(),
			[width as u32, height as u32],
			Format::R8G8B8A8Srgb,
		).unwrap())
	};

	let mut native_tex = |path: PathBuf| {
		let mut fp = File::open(path).unwrap();
		let mut magic_number = [0; 4];
		fp.read_exact(&mut magic_number).unwrap();
		if &magic_number != b"ntex" {
			return None;
		}
		println!(" => loader: native");
		let width = fp.read_u16::<LE>().unwrap();
		let height = fp.read_u16::<LE>().unwrap();
		let pony = fp.read_u8().unwrap();
		println!(" => resolution: {}x{}", width, height);
			
		let (bpp, fmt) = match pony {
			0 => (32, Format::R8G8B8A8Srgb),
			1 => (32, Format::R8G8B8A8Unorm),
			2 => (24, Format::R8G8B8Srgb),
			3 => (24, Format::R8G8B8Unorm),
			4 => (64, Format::R16G16B16A16Sfloat),
			5 => (128, Format::R32G32B32A32Sfloat),
			6 => (32, Format::A2R10G10B10UnormPack32),
			7 => (32, Format::A2R10G10B10UnormPack32),
			Any => { return None; }
		};
		let bytes = ((width as u64) * (height as u64) * (bpp as u64) + 7) / 8;

		let pixbuf: Arc<CpuAccessibleBuffer<[u8]>> = unsafe {
			CpuAccessibleBuffer::uninitialized_array(
				device.clone(),
				bytes as usize,
				BufferUsage::transfer_source()
			).unwrap()
		};
		{
			let mut pixels = pixbuf.write().unwrap();
			fp.read_exact(&mut pixels).unwrap();
		};
		return Some(Texture::from_buffer(
			ctx,
			pixbuf,
			[width as u32, height as u32],
			fmt,
		).unwrap());
	};
	
	let mut load_tex = |path_offset: u64, path_size: usize| {
		file.seek(SeekFrom::Start(path_offset)).unwrap();
		let mut buf = vec![0; path_size];
		file.read_exact(&mut buf).unwrap();
		let path_str = String::from_utf8(buf).unwrap();
		println!("load_tex({}):", path_str);
		let path = path.as_ref().parent().unwrap().join(path_str);
		match native_tex(path.clone()) {
			Some((ntex, nfut)) => {
				let nfutd: Box<dyn GpuFuture + Send + Sync> = Box::new(nfut);
				return Some((ntex, nfutd));
			},
			None => {}
		};
		match import_tex(path) {
			Some((ntex, nfut)) => {
				let nfutd: Box<dyn GpuFuture + Send + Sync> = Box::new(nfut);
				return Some((ntex, nfutd));
			},
			None => {}
		};
		println!(" => fail!");
		None
	};

	let mut mats = vec![];
	let mut mats_future: Box<dyn GpuFuture + Send + Sync> = Box::new(sync::now(device.clone()));
	for mat_info in mat_infos {
		let (tex1, tex1_future) = load_tex(mat_info.texture1_name_offset as u64, mat_info.texture1_name_size as usize).unwrap();
		
		//let (tex1, tex1_future) = import_tex(mat_info.texture1_name_offset as u64, mat_info.texture1_name_size as usize).unwrap();
		mats_future = Box::new(mats_future.join(tex1_future));
		
		/*
		let tex2_attempt = load_tex(mat_info.texture2_name_offset as u64, mat_info.texture2_name_size as usize);
		let tex2 = match tex2_attempt {
			None => ctx.white_pixel().clone(),
			Some(x) => {
				let (tex2_value, tex2_future) = x;
				mats_future = Box::new(mats_future.join(tex2_future));
				tex2_value
			}
		};
		*/
		let tex2 = ctx.white_pixel().clone();

		mats.push(Material::new(
			mat_info.range,
			tex1,
			tex2,
			mat_info.light_penetration,
			mat_info.subsurface_scattering,
			mat_info.emissive_brightness,
			mat_info.base_color,
		));
	}

	(
		MeshData::from_bufs(vertices, indices, queue.clone()),
		mats,
		vertices_future.join(indices_future).join(mats_future),
	)
}

struct MatInfo {
	range: Range<usize>,
	texture1_name_size: u16,
	texture1_name_offset: u32,
	texture2_name_size: u16,
	texture2_name_offset: u32,
	light_penetration: u8,
	subsurface_scattering: u8,
	emissive_brightness: u16,
	base_color: [u8; 3],
}
