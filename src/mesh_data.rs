use crate::Context;
use std::sync::Arc;
use vulkano::{
	buffer::{BufferAccess, BufferUsage, ImmutableBuffer, TypedBufferAccess},
	device::Queue,
	memory::DeviceMemoryAllocError,
	pipeline::input_assembly::PrimitiveTopology,
	sync::GpuFuture,
};

#[derive(Clone)]
pub struct MeshData {
	vertices: Arc<dyn BufferAccess + Send + Sync>,
	indices: IndexBuffer,
	topology: PrimitiveTopology,
}
impl MeshData {
	pub fn new_u16<V, Ib>(
		ctx: &Context,
		vertex_data: V,
		index_data: Ib,
		topology: PrimitiveTopology,
	) -> Result<(Arc<Self>, impl GpuFuture), DeviceMemoryAllocError>
	where
		V: Send + Sync + 'static,
		Ib: ExactSizeIterator<Item = u16>,
	{
		let queue = ctx.queue();

		let (vertices, vertices_future) = make_vertex_buffer(queue, vertex_data)?;
		let (indices, indices_future) =
			ImmutableBuffer::from_iter(index_data, BufferUsage::index_buffer(), queue.clone())?;
		let indices = IndexBuffer::U16(indices);

		let ret = Arc::new(Self { vertices, indices, topology });
		Ok((ret, vertices_future.join(indices_future)))
	}

	pub fn from_bufs_u16(
		vertices: Arc<dyn BufferAccess + Send + Sync>,
		indices: Arc<dyn TypedBufferAccess<Content = [u16]> + Send + Sync>,
		topology: PrimitiveTopology,
	) -> Arc<Self> {
		let indices = IndexBuffer::U16(indices);
		Arc::new(Self { vertices, indices, topology })
	}

	pub fn new_u32<V, Ib>(
		ctx: &Context,
		vertex_data: V,
		index_data: Ib,
		topology: PrimitiveTopology,
	) -> Result<(Arc<Self>, impl GpuFuture), DeviceMemoryAllocError>
	where
		V: Send + Sync + 'static,
		Ib: ExactSizeIterator<Item = u32>,
	{
		let queue = ctx.queue();

		let (vertices, vertices_future) = make_vertex_buffer(queue, vertex_data)?;
		let (indices, indices_future) =
			ImmutableBuffer::from_iter(index_data, BufferUsage::index_buffer(), queue.clone())?;
		let indices = IndexBuffer::U32(indices);

		let ret = Arc::new(Self { vertices, indices, topology });
		Ok((ret, vertices_future.join(indices_future)))
	}

	pub fn from_bufs_u32(
		vertices: Arc<dyn BufferAccess + Send + Sync>,
		indices: Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>,
		topology: PrimitiveTopology,
	) -> Arc<Self> {
		let indices = IndexBuffer::U32(indices);
		Arc::new(Self { vertices, indices, topology })
	}

	pub fn vertices(&self) -> &Arc<dyn BufferAccess + Send + Sync> {
		&self.vertices
	}

	pub fn indices(&self) -> &IndexBuffer {
		&self.indices
	}

	pub fn topology(&self) -> PrimitiveTopology {
		self.topology
	}
}

#[derive(Clone)]
pub enum IndexBuffer {
	U16(Arc<dyn TypedBufferAccess<Content = [u16]> + Send + Sync>),
	U32(Arc<dyn TypedBufferAccess<Content = [u32]> + Send + Sync>),
}
impl IndexBuffer {
	pub fn len(&self) -> usize {
		match self {
			Self::U16(buf) => buf.len(),
			Self::U32(buf) => buf.len(),
		}
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

fn make_vertex_buffer<V: Send + Sync + 'static>(
	queue: &Arc<Queue>,
	vertex_data: V,
) -> Result<(Arc<ImmutableBuffer<V>>, impl GpuFuture), DeviceMemoryAllocError> {
	ImmutableBuffer::from_data(vertex_data, BufferUsage::vertex_buffer(), queue.clone())
}
