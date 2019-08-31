use crate::{
	ctx,
	game_graph::{GGIndexFormat::*, GGVertexFormat::*, *},
	game_graph_driver::*,
};
use log::trace;
use nice_engine::{
	mesh_data::{MeshData, Pntl_32F},
	GpuFuture,
};
use std::{mem::size_of, slice, sync::Arc};
use vulkano::{
	buffer::{BufferAccess, BufferUsage, ImmutableBuffer},
	pipeline::input_assembly::PrimitiveTopology,
};

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_Alloc_Polygon(
	vertexFormat: GGVertexFormat,
	vertexBuffer: *const GGD_BufferInfo,
	indexFormat: GGIndexFormat,
	indexBuffer: *const GGD_BufferInfo,
	_cacheBuffer: *mut GGD_BufferInfo,
) -> *mut GGD_MeshData {
	trace!("MeshData_Alloc_Polygon");

	let vertexBuffer = &*vertexBuffer;
	let indexBuffer = &*indexBuffer;
	let queue = ctx::get().queue();

	let (vertices, vertices_future): (Arc<dyn BufferAccess + Send + Sync>, _) = match vertexFormat {
		VFMT_PNTL_32F => {
			let vertices = (vertexBuffer.read)(vertexBuffer, 0, vertexBuffer.size) as *const Pntl_32F;
			let len = vertexBuffer.size as usize / size_of::<Pntl_32F>();
			let buffer = slice::from_raw_parts(vertices, len).iter().cloned();
			let (vertices, vertices_future) =
				ImmutableBuffer::from_iter(buffer, BufferUsage::vertex_buffer(), queue.clone()).unwrap();
			(vertices, vertices_future)
		},
		VFMT_PNTLB3_32F => unimplemented!(),
		VFMT_PNTLB7_32F => unimplemented!(),
		VFMT_UNDEFINED => unimplemented!(),
	};

	if let Some(status) = vertexBuffer.status {
		status(vertexBuffer, GGD_BufferStatus::GGD_BUFFER_CLOSED as _);
	}

	let topology = match indexFormat {
		IFMT_SOUP_16U | IFMT_SOUP_32U => PrimitiveTopology::TriangleList,
		IFMT_STRIP_16U | IFMT_STRIP_32U => PrimitiveTopology::TriangleStrip,
		IFMT_UNDEFINED => unimplemented!(),
	};

	let (mesh_data, indices_future) = match indexFormat {
		IFMT_SOUP_16U | IFMT_STRIP_16U => {
			let indices = (indexBuffer.read)(indexBuffer, 0, indexBuffer.size) as *const u16;
			let len = indexBuffer.size as usize / size_of::<u16>();
			let buffer = slice::from_raw_parts(indices, len).iter().cloned();
			let (indices, indices_future) =
				ImmutableBuffer::from_iter(buffer, BufferUsage::index_buffer(), queue.clone()).unwrap();
			(MeshData::from_bufs_u16(vertices, indices, topology), indices_future)
		},
		IFMT_SOUP_32U | IFMT_STRIP_32U => {
			let indices = (indexBuffer.read)(indexBuffer, 0, indexBuffer.size) as *const u32;
			let len = indexBuffer.size as usize / size_of::<u32>();
			let buffer = slice::from_raw_parts(indices, len).iter().cloned();
			let (indices, indices_future) =
				ImmutableBuffer::from_iter(buffer, BufferUsage::index_buffer(), queue.clone()).unwrap();
			(MeshData::from_bufs_u32(vertices, indices, topology), indices_future)
		},
		IFMT_UNDEFINED => unimplemented!(),
	};

	vertices_future.join(indices_future).then_signal_fence_and_flush().unwrap().wait(None).unwrap();
	Box::into_raw(Box::new(mesh_data))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_Free(this: *mut GGD_MeshData) {
	trace!("MeshData_Free");

	Box::from_raw(this);
}
