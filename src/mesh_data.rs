use crate::Context;
use std::sync::Arc;
use vulkano::{
	buffer::{BufferAccess, BufferUsage, ImmutableBuffer},
	device::Queue,
	memory::DeviceMemoryAllocError,
	sync::GpuFuture,
};

pub struct MeshData {
	vertices: Arc<dyn BufferAccess + Send + Sync>,
	indices: Arc<dyn BufferAccess + Send + Sync>,
	queue: Arc<Queue>,
}
impl MeshData {
	pub fn new<V, I>(
		ctx: &Context,
		vertex_data: V,
		index_data: I,
	) -> Result<(Arc<Self>, impl GpuFuture), DeviceMemoryAllocError>
	where
		V: Send + Sync + 'static,
		I: Send + Sync + 'static,
	{
		let (vertices, vertices_future) =
			ImmutableBuffer::from_data(vertex_data, BufferUsage::vertex_buffer(), ctx.queue().clone())?;
		let (indices, indices_future) =
			ImmutableBuffer::from_data(index_data, BufferUsage::index_buffer(), ctx.queue().clone())?;

		let ret = Arc::new(Self { vertices, indices, queue: ctx.queue().clone() });
		Ok((ret, vertices_future.join(indices_future)))
	}

	pub fn set_vertex_data<T>(&mut self, data: T) -> Result<impl GpuFuture, DeviceMemoryAllocError>
	where
		T: Send + Sync + 'static,
	{
		let (vertices, vertices_future) =
			ImmutableBuffer::from_data(data, BufferUsage::vertex_buffer(), self.queue.clone())?;
		self.vertices = vertices;

		Ok(vertices_future)
	}

	pub fn set_index_data<T>(&mut self, data: T) -> Result<impl GpuFuture, DeviceMemoryAllocError>
	where
		T: Send + Sync + 'static,
	{
		let (indices, indices_future) =
			ImmutableBuffer::from_data(data, BufferUsage::vertex_buffer(), self.queue.clone())?;
		self.indices = indices;

		Ok(indices_future)
	}

	pub fn vertices(&self) -> &Arc<dyn BufferAccess + Send + Sync> {
		&self.vertices
	}

	pub fn indices(&self) -> &Arc<dyn BufferAccess + Send + Sync> {
		&self.indices
	}
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Pntl_32F {
	pub pos: [f32; 3],
	pub nor: [f32; 3],
	pub tex: [f32; 2],
	pub lmap: [f32; 2],
}
vulkano::impl_vertex!(Pntl_32F, pos, nor, tex, lmap);

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Pntlb3_32F {
	pub pos: [f32; 3],
	pub nor: [f32; 3],
	pub tex: [f32; 2],
	pub lmap: [f32; 2],
	pub bone_ids: [f32; 3],
	pub bone_weights: [f32; 3],
}
vulkano::impl_vertex!(Pntlb3_32F, pos, nor, tex, lmap, bone_ids, bone_weights);

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Pntlb7_32F {
	pub pos: [f32; 3],
	pub nor: [f32; 3],
	pub tex: [f32; 2],
	pub lmap: [f32; 2],
	pub bone_ids: [f32; 7],
	pub bone_weights: [f32; 7],
}
vulkano::impl_vertex!(Pntlb7_32F, pos, nor, tex, lmap, bone_ids, bone_weights);
