use crate::{
	mesh_data::{MeshData, Pntl_32F},
	Context,
};
use byteorder::{ReadBytesExt, LE};
use log::debug;
use std::{
	fs::File,
	io::{prelude::*, SeekFrom},
	mem::drop,
	path::Path,
	sync::Arc,
};
use vulkano::{
	buffer::{BufferUsage, CpuAccessibleBuffer, ImmutableBuffer},
	sync::GpuFuture,
};

pub fn from_nice_model(
	ctx: &Context,
	path: impl AsRef<Path> + Clone + Send + 'static,
) -> (Arc<MeshData>, impl GpuFuture + Send + Sync + 'static) {
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

	(MeshData::from_bufs(vertices, indices, queue.clone()), vertices_future.join(indices_future))
}
