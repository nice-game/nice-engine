use std::{
	iter,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};
use vulkano::{
	buffer::{BufferAccess, TypedBufferAccess},
	command_buffer::{
		pool::standard::StandardCommandPoolAlloc,
		sys::{
			Flags, Kind, UnsafeCommandBuffer, UnsafeCommandBufferBuilder, UnsafeCommandBufferBuilderBufferImageCopy,
			UnsafeCommandBufferBuilderImageAspect, UnsafeCommandBufferBuilderImageBlit,
			UnsafeCommandBufferBuilderPipelineBarrier,
		},
		CommandBuffer, CommandBufferExecError,
	},
	device::{Device, DeviceOwned, Queue},
	format::FormatDesc,
	image::{immutable::ImmutableImageInitialization, ImageAccess, ImageLayout},
	instance::QueueFamily,
	sampler::Filter,
	sync::{AccessCheckError, AccessError, AccessFlagBits, GpuFuture, PipelineStages},
};

pub(crate) struct MipmapsCommandBuffer<B, P, F>
where
	F: FormatDesc + Send + Sync + 'static,
	B: BufferAccess + TypedBufferAccess<Content = [P]> + Clone + Send + Sync + 'static,
	P: Send + Sync + Clone + 'static,
{
	device: Arc<Device>,
	inner: UnsafeCommandBuffer<StandardCommandPoolAlloc>,
	already_submitted: AtomicBool,
	buffer: B,
	init: ImmutableImageInitialization<F>,
}
impl<B, P, F> MipmapsCommandBuffer<B, P, F>
where
	F: FormatDesc + Send + Sync + 'static,
	B: BufferAccess + TypedBufferAccess<Content = [P]> + Clone + Send + Sync + 'static,
	P: Send + Sync + Clone + 'static,
{
	pub(crate) fn new(
		device: Arc<Device>,
		queue_family: QueueFamily,
		buffer: B,
		init: ImmutableImageInitialization<F>,
	) -> Self {
		let pool = Device::standard_command_pool(&device, queue_family);
		let mut cmds =
			unsafe { UnsafeCommandBufferBuilder::new(&pool, Kind::primary(), Flags::OneTimeSubmit).unwrap() };

		let mut barrier = UnsafeCommandBufferBuilderPipelineBarrier::new();
		unsafe {
			barrier.add_image_memory_barrier(
				&init,
				0..init.mipmap_levels(),
				0..1,
				PipelineStages { bottom_of_pipe: true, ..PipelineStages::none() },
				AccessFlagBits::none(),
				PipelineStages { transfer: true, ..PipelineStages::none() },
				AccessFlagBits { transfer_read: true, ..AccessFlagBits::none() },
				true,
				None,
				ImageLayout::Undefined,
				ImageLayout::TransferDstOptimal,
			);
			cmds.pipeline_barrier(&barrier);
		}

		let aspect = UnsafeCommandBufferBuilderImageAspect { color: true, depth: false, stencil: false };
		let dimensions = init.dimensions();

		let copy = UnsafeCommandBufferBuilderBufferImageCopy {
			buffer_offset: 0,
			buffer_row_length: 0,
			buffer_image_height: 0,
			image_aspect: aspect,
			image_mip_level: 0,
			image_base_array_layer: 0,
			image_layer_count: 1,
			image_offset: [0; 3],
			image_extent: dimensions.width_height_depth(),
		};
		unsafe { cmds.copy_buffer_to_image(&buffer, &init, ImageLayout::TransferDstOptimal, iter::once(copy)) };

		let mut barrier = UnsafeCommandBufferBuilderPipelineBarrier::new();
		unsafe {
			barrier.add_image_memory_barrier(
				&init,
				0..1,
				0..1,
				PipelineStages { transfer: true, ..PipelineStages::none() },
				AccessFlagBits { transfer_write: true, ..AccessFlagBits::none() },
				PipelineStages { transfer: true, ..PipelineStages::none() },
				AccessFlagBits { transfer_read: true, ..AccessFlagBits::none() },
				true,
				None,
				ImageLayout::TransferDstOptimal,
				ImageLayout::TransferSrcOptimal,
			);
			cmds.pipeline_barrier(&barrier);
		}

		let mut last_mip_dimensions = dimensions;
		for mip in 1..init.mipmap_levels() {
			let mip_dimensions = last_mip_dimensions.mipmap_dimensions(1).unwrap();

			let source_bottom_right = [
				last_mip_dimensions.width() as i32,
				last_mip_dimensions.height() as i32,
				last_mip_dimensions.depth() as i32,
			];
			let destination_bottom_right =
				[mip_dimensions.width() as i32, mip_dimensions.height() as i32, mip_dimensions.depth() as i32];

			let blit = UnsafeCommandBufferBuilderImageBlit {
				aspect,
				source_mip_level: mip - 1,
				destination_mip_level: mip,
				source_base_array_layer: 0,
				destination_base_array_layer: 0,
				layer_count: 1,
				source_top_left: [0; 3],
				source_bottom_right,
				destination_top_left: [0; 3],
				destination_bottom_right,
			};

			unsafe {
				cmds.blit_image(
					&init,
					ImageLayout::TransferSrcOptimal,
					&init,
					ImageLayout::TransferDstOptimal,
					iter::once(blit),
					Filter::Linear,
				)
			};

			let mut barrier = UnsafeCommandBufferBuilderPipelineBarrier::new();
			unsafe {
				barrier.add_image_memory_barrier(
					&init,
					mip..mip + 1,
					0..1,
					PipelineStages { transfer: true, ..PipelineStages::none() },
					AccessFlagBits { transfer_write: true, ..AccessFlagBits::none() },
					PipelineStages { transfer: true, ..PipelineStages::none() },
					AccessFlagBits { transfer_read: true, ..AccessFlagBits::none() },
					true,
					None,
					ImageLayout::TransferDstOptimal,
					ImageLayout::TransferSrcOptimal,
				);
				cmds.pipeline_barrier(&barrier);
			}

			last_mip_dimensions = mip_dimensions;
		}

		let mut barrier = UnsafeCommandBufferBuilderPipelineBarrier::new();
		unsafe {
			barrier.add_image_memory_barrier(
				&init,
				0..init.mipmap_levels(),
				0..1,
				PipelineStages { transfer: true, ..PipelineStages::none() },
				AccessFlagBits { transfer_write: true, ..AccessFlagBits::none() },
				PipelineStages { top_of_pipe: true, ..PipelineStages::none() },
				AccessFlagBits::none(),
				true,
				None,
				ImageLayout::TransferSrcOptimal,
				init.final_layout_requirement(),
			);
			cmds.pipeline_barrier(&barrier);
			init.layout_initialized();
		}

		Self { device, inner: cmds.build().unwrap(), already_submitted: AtomicBool::new(false), buffer, init }
	}

	fn lock_buffer(&self, future: &dyn GpuFuture, queue: &Queue) -> Result<(), CommandBufferExecError> {
		// Because try_gpu_lock needs to be called first,
		// this should never return Ok without first returning Err
		let prev_err = match future.check_buffer_access(&self.buffer, false, queue) {
			Ok(_) => {
				unsafe { self.buffer.increase_gpu_lock() };
				return Ok(());
			},
			Err(err) => err,
		};

		match (self.buffer.try_gpu_lock(false, queue), prev_err) {
			(Ok(_), _) => Ok(()),
			(Err(err), AccessCheckError::Unknown) | (_, AccessCheckError::Denied(err)) => {
				Err(CommandBufferExecError::AccessError {
					error: err,
					command_name: "vkCmdCopyBufferToImage".into(),
					command_param: "source".into(),
					command_offset: 0,
				})
			},
		}
	}

	fn lock_image(&self, future: &dyn GpuFuture, queue: &Queue) -> Result<(), CommandBufferExecError> {
		let prev_err = match future.check_image_access(&self.init, self.init.initial_layout_requirement(), true, queue)
		{
			Ok(_) => {
				unsafe {
					self.init.increase_gpu_lock();
				}
				return Ok(());
			},
			Err(err) => err,
		};

		match (self.init.try_gpu_lock(true, self.init.initial_layout_requirement()), prev_err) {
			(Ok(_), _) => Ok(()),
			(Err(err), AccessCheckError::Unknown) | (_, AccessCheckError::Denied(err)) => {
				Err(CommandBufferExecError::AccessError {
					error: err,
					command_name: "vkCmdBlitImage".into(),
					command_param: "source".into(),
					command_offset: 1,
				})
			},
		}
	}
}
unsafe impl<B, P, F> CommandBuffer for MipmapsCommandBuffer<B, P, F>
where
	F: FormatDesc + Send + Sync + 'static,
	B: BufferAccess + TypedBufferAccess<Content = [P]> + Clone + Send + Sync + 'static,
	P: Send + Sync + Clone + 'static,
{
	type PoolAlloc = StandardCommandPoolAlloc;

	#[inline]
	fn inner(&self) -> &UnsafeCommandBuffer<StandardCommandPoolAlloc> {
		&self.inner
	}

	#[inline]
	fn lock_submit(&self, future: &dyn GpuFuture, queue: &Queue) -> Result<(), CommandBufferExecError> {
		let was_already_submitted = self.already_submitted.swap(true, Ordering::SeqCst);
		if was_already_submitted {
			return Err(CommandBufferExecError::OneTimeSubmitAlreadySubmitted);
		}

		self.lock_buffer(future, queue)?;
		let ret = self.lock_image(future, queue);
		if ret.is_err() {
			unsafe { self.buffer.unlock() };
		}
		ret
	}

	#[inline]
	unsafe fn unlock(&self) {
		self.buffer.unlock();
		self.init.unlock(Some(ImageLayout::ShaderReadOnlyOptimal));
		debug_assert!(self.already_submitted.load(Ordering::SeqCst));
	}

	#[inline]
	fn check_buffer_access(
		&self,
		buffer: &dyn BufferAccess,
		exclusive: bool,
		_queue: &Queue,
	) -> Result<Option<(PipelineStages, AccessFlagBits)>, AccessCheckError> {
		if self.buffer.inner().buffer.key() == buffer.inner().buffer.key() {
			if exclusive {
				return Err(AccessCheckError::Unknown);
			}

			return Ok(Some((PipelineStages { transfer: true, ..PipelineStages::none() }, AccessFlagBits {
				transfer_read: true,
				..AccessFlagBits::none()
			})));
		}

		Err(AccessCheckError::Unknown)
	}

	#[inline]
	fn check_image_access(
		&self,
		image: &dyn ImageAccess,
		layout: ImageLayout,
		_exclusive: bool,
		_queue: &Queue,
	) -> Result<Option<(PipelineStages, AccessFlagBits)>, AccessCheckError> {
		if self.init.inner().image.key() == image.inner().image.key() {
			if layout != ImageLayout::Undefined && self.init.final_layout_requirement() != layout {
				return Err(AccessCheckError::Denied(AccessError::UnexpectedImageLayout {
					allowed: self.init.final_layout_requirement(),
					requested: layout,
				}));
			}

			return Ok(Some((PipelineStages { top_of_pipe: true, ..PipelineStages::none() }, AccessFlagBits::none())));
		}

		Err(AccessCheckError::Unknown)
	}
}
unsafe impl<B, P, F> DeviceOwned for MipmapsCommandBuffer<B, P, F>
where
	F: FormatDesc + Send + Sync + 'static,
	B: BufferAccess + TypedBufferAccess<Content = [P]> + Clone + Send + Sync + 'static,
	P: Send + Sync + Clone + 'static,
{
	fn device(&self) -> &Arc<Device> {
		&self.device
	}
}
