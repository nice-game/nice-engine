use crate::Context;
use std::sync::Arc;
use vulkano::{
	buffer::{BufferAccess, BufferUsage, ImmutableBuffer, TypedBufferAccess},
	device::Queue,
	memory::DeviceMemoryAllocError,
	sync::GpuFuture,
};

#[derive(Clone)]
pub struct MeshData {
	vertices: Arc<dyn BufferAccess + Send + Sync>,
	indices: Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>,
}
impl MeshData {
	pub fn new<V, I>(
		ctx: &Context,
		vertex_data: V,
		index_data: I,
	) -> Result<(Arc<Self>, impl GpuFuture), DeviceMemoryAllocError>
	where
		V: Send + Sync + 'static,
		I: ExactSizeIterator<Item = u32>,
	{
		let (vertices, vertices_future) =
			ImmutableBuffer::from_data(vertex_data, BufferUsage::vertex_buffer(), ctx.queue().clone())?;
		let (indices, indices_future) =
			ImmutableBuffer::from_iter(index_data, BufferUsage::index_buffer(), ctx.queue().clone())?;

		let ret = Arc::new(Self { vertices, indices });
		Ok((ret, vertices_future.join(indices_future)))
	}

	pub fn from_bufs(
		vertices: Arc<dyn BufferAccess + Send + Sync>,
		indices: Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>,
	) -> Arc<Self> {
		Arc::new(Self { vertices, indices })
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
pub struct Pntl_32F {
	pub pos: [f32; 3],
	pub nor: [f32; 3],
	pub texc: [f32; 2],
	pub lmap: [f32; 2],
}
vulkano::impl_vertex!(Pntl_32F, pos, nor, texc, lmap);

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Pntlb3_32F {
	pub pos: [f32; 3],
	pub nor: [f32; 3],
	pub texc: [f32; 2],
	pub lmap: [f32; 2],
	pub bone_ids: [f32; 3],
	pub bone_weights: [f32; 3],
}
vulkano::impl_vertex!(Pntlb3_32F, pos, nor, texc, lmap, bone_ids, bone_weights);

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Pntlb7_32F {
	pub pos: [f32; 3],
	pub nor: [f32; 3],
	pub texc: [f32; 2],
	pub lmap: [f32; 2],
	pub bone_ids: [f32; 7],
	pub bone_weights: [f32; 7],
}
vulkano::impl_vertex!(Pntlb7_32F, pos, nor, texc, lmap, bone_ids, bone_weights);
