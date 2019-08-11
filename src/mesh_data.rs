use std::sync::Arc;
use vulkano::{
	buffer::{BufferAccess, TypedBufferAccess},
	device::Queue,
};

#[derive(Clone)]
pub(crate) struct MeshData {
	vertices: Arc<dyn BufferAccess + Send + Sync>,
	indices: Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>,
	queue: Arc<Queue>,
}
impl MeshData {
	pub(crate) fn from_bufs(
		vertices: Arc<dyn BufferAccess + Send + Sync>,
		indices: Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>,
		queue: Arc<Queue>,
	) -> Arc<Self> {
		Arc::new(Self { vertices, indices, queue })
	}

	pub(crate) fn vertices(&self) -> &Arc<dyn BufferAccess + Send + Sync> {
		&self.vertices
	}

	pub(crate) fn indices(&self) -> &Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync> {
		&self.indices
	}
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct Pntl_32F {
	pub(crate) pos: [f32; 3],
	pub(crate) nor: [f32; 3],
	pub(crate) texc: [f32; 2],
	pub(crate) lmap: [f32; 2],
}
vulkano::impl_vertex!(Pntl_32F, pos, nor, texc, lmap);

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct Pntlb3_32F {
	pub(crate) pos: [f32; 3],
	pub(crate) nor: [f32; 3],
	pub(crate) texc: [f32; 2],
	pub(crate) lmap: [f32; 2],
	pub(crate) bone_ids: [f32; 3],
	pub(crate) bone_weights: [f32; 3],
}
vulkano::impl_vertex!(Pntlb3_32F, pos, nor, texc, lmap, bone_ids, bone_weights);

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct Pntlb7_32F {
	pub(crate) pos: [f32; 3],
	pub(crate) nor: [f32; 3],
	pub(crate) texc: [f32; 2],
	pub(crate) lmap: [f32; 2],
	pub(crate) bone_ids: [f32; 7],
	pub(crate) bone_weights: [f32; 7],
}
vulkano::impl_vertex!(Pntlb7_32F, pos, nor, texc, lmap, bone_ids, bone_weights);
