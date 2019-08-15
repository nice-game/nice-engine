use crate::{
	ctx,
	game_graph::{
		GGIndexFormat::{self, *},
		GGVertexFormat::{self, *},
	},
	game_graph_driver::GGD_MeshData,
};
use libc::c_void;
use nice_engine::{
	mesh_data::{MeshData, Pntl_32F},
	GpuFuture,
};
use std::{slice, sync::Arc};
use vulkano::buffer::{BufferAccess, BufferUsage, ImmutableBuffer, TypedBufferAccess};

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_Alloc_Polygon(
	vertexBuffer: *const c_void,
	vertexCount: u32,
	vertexFormat: GGVertexFormat,
	indexBuffer: *const c_void,
	indexCount: u32,
	indexFormat: GGIndexFormat,
) -> *mut GGD_MeshData {
	let queue = ctx::get().queue();

	let (vertices, vertices_future): (Arc<dyn BufferAccess + Send + Sync>, _) = match vertexFormat {
		VFMT_PNTL_32F => {
			let buffer = slice::from_raw_parts(vertexBuffer as *const Pntl_32F, vertexCount as usize).iter().cloned();
			let (vertices, vertices_future) =
				ImmutableBuffer::from_iter(buffer, BufferUsage::vertex_buffer(), queue.clone()).unwrap();
			(vertices, vertices_future)
		},
		VFMT_PNTLB3_32F => unimplemented!(),
		VFMT_PNTLB7_32F => unimplemented!(),
		VFMT_UNDEFINED => unimplemented!(),
	};

	let (indices, indices_future): (Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>, _) = match indexFormat {
		IFMT_SOUP_16U => unimplemented!(),
		IFMT_SOUP_32U => unimplemented!(),
		IFMT_STRIP_16U => unimplemented!(),
		IFMT_STRIP_32U => {
			let buffer = slice::from_raw_parts(indexBuffer as *const u32, indexCount as usize).iter().cloned();
			let (indices, indices_future) =
				ImmutableBuffer::from_iter(buffer, BufferUsage::vertex_buffer(), queue.clone()).unwrap();
			(indices, indices_future)
		},
		IFMT_UNDEFINED => unimplemented!(),
	};

	let mesh_data = MeshData::from_bufs(vertices, indices);
	vertices_future.join(indices_future).then_signal_fence_and_flush().unwrap().wait(None).unwrap();
	Box::into_raw(Box::new(mesh_data))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_Free(this: *mut GGD_MeshData) {
	Box::from_raw(this);
}
