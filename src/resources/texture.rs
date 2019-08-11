use crate::texture::ImmutableTexture;
use byteorder::{ReadBytesExt, LE};
use log::debug;
use std::{fs::File, io::prelude::*, path::Path, sync::Arc};
use vulkano::{
	buffer::{BufferUsage, CpuAccessibleBuffer},
	device::Queue,
	format::Format,
	sync::GpuFuture,
};

pub(crate) fn from_nice_texture(
	queue: &Arc<Queue>,
	path: impl AsRef<Path> + Clone + Send,
) -> (ImmutableTexture, impl GpuFuture) {
	let mut fp = File::open(path).unwrap();

	let mut magic_number = [0; 3];
	fp.read_exact(&mut magic_number).unwrap();
	if &magic_number != b"ntx" {
		panic!("Invalid ntx file");
	}

	let format = fp.read_u8().unwrap();
	let width = fp.read_u16::<LE>().unwrap();
	let height = fp.read_u16::<LE>().unwrap();
	debug!(" => resolution: {}x{}", width, height);

	let (bpp, fmt) = match format {
		0 => (32, Format::R8G8B8A8Srgb),
		1 => (32, Format::R8G8B8A8Unorm),
		2 => (32, Format::A2B10G10R10UnormPack32),
		3 => (32, Format::A2B10G10R10UnormPack32),
		4 => (64, Format::R16G16B16A16Sfloat),
		5 => (128, Format::R32G32B32A32Sfloat),
		_ => panic!("Invalid ntx file"),
	};
	let bytes = ((width as u64) * (height as u64) * (bpp as u64) + 7) / 8;

	let pixbuf: Arc<CpuAccessibleBuffer<[u8]>> = unsafe {
		CpuAccessibleBuffer::uninitialized_array(queue.device().clone(), bytes as usize, BufferUsage::transfer_source())
			.unwrap()
	};
	{
		let mut pixels = pixbuf.write().unwrap();
		fp.read_exact(&mut pixels).unwrap();
	};

	let (tex, tex_future) =
		ImmutableTexture::from_buffer(queue.clone(), pixbuf, [width as u32, height as u32], fmt).unwrap();

	(tex, Box::new(tex_future))
}
