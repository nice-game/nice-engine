use crate::{ ctx, game_graph::{ GGIndexFormat, GGVertexFormat }, game_graph_driver::GGD_MeshData };
use libc::c_void;
use nice_engine::{ GpuFuture, mesh_data::{ MeshData, Pntl_32F, Pntlb3_32F, Pntlb7_32F } };
use std::slice;

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_Alloc() -> *mut GGD_MeshData {
	let (mesh_data, future) = MeshData::new(ctx::get(), (), ()).unwrap();
	future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
	Box::into_raw(Box::new(mesh_data))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_Free(this: *mut GGD_MeshData) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_SetVertexData(this: *mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGVertexFormat) {
	let mut this_ref = Box::from_raw(this);

	let future: Box<GpuFuture> = match format {
		GGVertexFormat::PNTL_32F => Box::new(
			this_ref.set_vertex_data(slice::from_raw_parts(buffer as *const Pntl_32F, count as usize)).unwrap()
		),
		GGVertexFormat::PNTLB3_32F => Box::new(
			this_ref.set_vertex_data(slice::from_raw_parts(buffer as *const Pntlb3_32F, count as usize)).unwrap()
		),
		GGVertexFormat::PNTLB7_32F => Box::new(
			this_ref.set_vertex_data(slice::from_raw_parts(buffer as *const Pntlb7_32F, count as usize)).unwrap()
		),
		GGVertexFormat::UNDEFINED => Box::new(
			this_ref.set_vertex_data(slice::from_raw_parts(buffer as *const u8, count as usize)).unwrap()
		),
	};

	future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
}

#[allow(non_snake_case)]
pub extern fn MeshData_GetVertexData(_this: *mut GGD_MeshData, _buffer: *mut c_void, _count: *mut u32, _format: *mut GGVertexFormat) {
	panic!("MeshData_GetVertexData not implemented");
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_SetIndexData(this: *mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGIndexFormat) {
	let mut this_ref = Box::from_raw(this);

	let future: Box<GpuFuture> = match format {
		GGIndexFormat::SOUP_16U | GGIndexFormat::STRIP_16U => Box::new(
			this_ref.set_vertex_data(slice::from_raw_parts(buffer as *const u16, count as usize)).unwrap()
		),
		GGIndexFormat::SOUP_32U | GGIndexFormat::STRIP_32U => Box::new(
			this_ref.set_vertex_data(slice::from_raw_parts(buffer as *const u32, count as usize)).unwrap()
		),
		GGIndexFormat::UNDEFINED => Box::new(
			this_ref.set_vertex_data(slice::from_raw_parts(buffer as *const u8, count as usize)).unwrap()
		),
	};

	future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
}

#[allow(non_snake_case)]
pub extern fn MeshData_GetIndexData(_this: *mut GGD_MeshData, _buffer: *mut c_void, _count: *mut u32, _format: *mut GGIndexFormat) {
	panic!("MeshData_GetIndexData not implemented");
}
