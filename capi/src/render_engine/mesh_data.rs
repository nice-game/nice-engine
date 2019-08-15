use crate::{
	ctx,
	game_graph::{GGIndexFormat, GGVertexFormat},
	game_graph_driver::GGD_MeshData,
};
use libc::c_void;
use nice_engine::{
	mesh_data::{MeshData, Pntl_32F, Pntlb3_32F, Pntlb7_32F},
	GpuFuture,
};
use std::{slice, sync::Arc};
use vulkano::buffer::{BufferAccess, BufferUsage, ImmutableBuffer, TypedBufferAccess};

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_Alloc(
	vertexBuffer: *const c_void,
	vertexCount: u32,
	vertexFormat: GGVertexFormat,
	indexBuffer: *const c_void,
	indexCount: u32,
	indexFormat: GGIndexFormat,
) -> *mut GGD_MeshData {
	let queue = ctx::get().queue();

	let (vertices, vertices_future): (Arc<dyn BufferAccess + Send + Sync>, _) = match vertexFormat {
		GGVertexFormat::PNTL_32F => {
			let buffer = slice::from_raw_parts(vertexBuffer as *const Pntl_32F, vertexCount as usize).iter().cloned();
			let (vertices, vertices_future) =
				ImmutableBuffer::from_iter(buffer, BufferUsage::vertex_buffer(), queue.clone()).unwrap();
			(vertices, vertices_future)
		},
		GGVertexFormat::PNTLB3_32F => unimplemented!(),
		GGVertexFormat::PNTLB7_32F => unimplemented!(),
		GGVertexFormat::UNDEFINED => unimplemented!(),
	};

	let (indices, indices_future): (Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>, _) = match indexFormat {
		GGIndexFormat::SOUP_16U => unimplemented!(),
		GGIndexFormat::SOUP_32U => unimplemented!(),
		GGIndexFormat::STRIP_16U => unimplemented!(),
		GGIndexFormat::STRIP_32U => {
			let buffer = slice::from_raw_parts(indexBuffer as *const u32, indexCount as usize).iter().cloned();
			let (indices, indices_future) =
				ImmutableBuffer::from_iter(buffer, BufferUsage::vertex_buffer(), queue.clone()).unwrap();
			(indices, indices_future)
		},
		GGIndexFormat::UNDEFINED => unimplemented!(),
	};

	let mesh_data = MeshData::from_bufs(vertices, indices);
	vertices_future.join(indices_future).then_signal_fence_and_flush().unwrap().wait(None).unwrap();
	Box::into_raw(Box::new(mesh_data))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_Free(this: *mut GGD_MeshData) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn MeshData_GetVertexData(
	_this: *mut GGD_MeshData,
	_buffer: *mut c_void,
	_count: *mut u32,
	_format: *mut GGVertexFormat,
) {
	panic!("MeshData_GetVertexData not implemented");
}

#[allow(non_snake_case)]
pub extern fn MeshData_GetIndexData(
	_this: *mut GGD_MeshData,
	_buffer: *mut c_void,
	_count: *mut u32,
	_format: *mut GGIndexFormat,
) {
	panic!("MeshData_GetIndexData not implemented");
}
