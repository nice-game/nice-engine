use crate::mesh_data::{MeshData, Pntl_32F};
use byteorder::{ReadBytesExt, LE};
use std::{
	fs::File,
	io::{prelude::*, SeekFrom},
	mem::drop,
	ops::Range,
	path::{Path, PathBuf},
	sync::Arc,
};
use vulkano::{
	buffer::{BufferUsage, CpuAccessibleBuffer, ImmutableBuffer},
	device::Queue,
	pipeline::input_assembly::PrimitiveTopology,
	sync::GpuFuture,
};

pub(crate) fn from_nice_model(
	queue: &Arc<Queue>,
	path: impl AsRef<Path> + Clone + Send,
) -> (Arc<MeshData>, Vec<MaterialInfo>, impl GpuFuture + Send + Sync + 'static) {
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

	// println!("vertex_count: {}", vertex_count);
	// println!("positions_offset: {}", positions_offset);
	// println!("normals_offset: {}", normals_offset);
	// println!("texcoords_main_offset: {}", texcoords_main_offset);
	// println!("texcoords_lightmap_offset: {}", texcoords_lightmap_offset);
	// println!("index_count: {}", index_count);
	// println!("indices_offset: {}", indices_offset);
	// println!("material_count: {}", material_count);
	// println!("materials_offset: {}", materials_offset);

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

		mat_infos.push(MaterialRaw {
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

	let mut read_path = |path_offset: u64, path_size: usize| {
		file.seek(SeekFrom::Start(path_offset)).unwrap();
		let mut buf = vec![0; path_size];
		file.read_exact(&mut buf).unwrap();
		let mut path_str = String::from_utf8(buf).unwrap();
		// println!("read_path({}):", path_str);
		if path_str.is_empty() {
			path_str = "default.ntx".to_string();
		};
		path.as_ref().parent().unwrap().join(path_str)
	};

	let mut mats = vec![];
	for mat_info in mat_infos {
		mats.push(MaterialInfo {
			range: mat_info.range,
			tex1: read_path(mat_info.texture1_name_offset as u64, mat_info.texture1_name_size as usize),
			tex2: read_path(mat_info.texture2_name_offset as u64, mat_info.texture2_name_size as usize),
			light_penetration: mat_info.light_penetration,
			subsurface_scattering: mat_info.subsurface_scattering,
			emissive_brightness: mat_info.emissive_brightness,
			base_color: mat_info.base_color,
		});
	}

	let mesh_data = MeshData::from_bufs_u32(vertices, indices, PrimitiveTopology::TriangleList);
	(mesh_data, mats, vertices_future.join(indices_future))
}

pub(crate) struct MaterialInfo {
	pub(crate) range: Range<usize>,
	pub(crate) tex1: PathBuf,
	pub(crate) tex2: PathBuf,
	pub(crate) light_penetration: u8,
	pub(crate) subsurface_scattering: u8,
	pub(crate) emissive_brightness: u16,
	pub(crate) base_color: [u8; 3],
}

struct MaterialRaw {
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
