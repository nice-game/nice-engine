use crate::Context;
use std::sync::Arc;
use vulkano::{
	buffer::{ BufferAccess, BufferUsage, ImmutableBuffer },
	device::Queue,
	memory::DeviceMemoryAllocError,
	sync::GpuFuture,
}

pub struct MeshData {
	vertices: Arc<BufferAccess>,
	indices: Arc<BufferAccess>,
	queue: Arc<Queue>,
}
impl MeshData {
	pub fn new<V, I>(
		ctx: &Context,
		vertex_data: V,
		index_data: I,
	) -> Result<(Self, impl GpuFuture), DeviceMemoryAllocError>
	where V: Send + Sync + 'static, I: Send + Sync + 'static {
		let (vertices, vertices_future) =
			ImmutableBuffer::from_data(vertex_data, BufferUsage::vertex_buffer(), ctx.queue().clone())?;
		let (indices, indices_future) =
			ImmutableBuffer::from_data(index_data, BufferUsage::index_buffer(), ctx.queue().clone())?;

		Ok((
			Self { vertices: vertices, indices: indices, queue: ctx.queue().clone() },
			vertices_future.join(indices_future),
		))
	}

	pub fn set_vertex_data<T>(&mut self, data: T) -> Result<impl GpuFuture, DeviceMemoryAllocError>
	where T: Send + Sync + 'static {
		let (vertices, vertices_future) =
			ImmutableBuffer::from_data(data, BufferUsage::vertex_buffer(), self.queue.clone())?;
		self.vertices = vertices;

		Ok(vertices_future)
	}

	pub fn set_index_data<T>(&mut self, data: T) -> Result<impl GpuFuture, DeviceMemoryAllocError>
	where T: Send + Sync + 'static {
		let (indices, indices_future) =
			ImmutableBuffer::from_data(data, BufferUsage::vertex_buffer(), self.queue.clone())?;
		self.indices = indices;

		Ok(indices_future)
	}
}

#[repr(C)]
pub struct Pntl_32F {
	pos: [f32; 3],
	nor: [f32; 3],
	tex: [f32; 2],
	lmap: [f32; 2],
}

#[repr(C)]
pub struct Pntlb3_32F {
	pos: [f32; 3],
	nor: [f32; 3],
	tex: [f32; 2],
	lmap: [f32; 2],
	bone_ids: [f32; 3],
	bone_weights: [f32; 3],
}

#[repr(C)]
pub struct Pntlb7_32F {
	pos: [f32; 3],
	nor: [f32; 3],
	tex: [f32; 2],
	lmap: [f32; 2],
	bone_ids: [f32; 7],
	bone_weights: [f32; 7],
}
