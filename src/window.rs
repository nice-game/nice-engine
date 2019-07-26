#![cfg(feature = "window")]

use crate::{surface::Surface, Context};
use vulkano_win::{CreationError, VkSurfaceBuild};
use winit::{EventsLoop, WindowBuilder};

pub struct Window {
	surface: Surface<winit::Window>,
}
impl Window {
	pub fn new(ctx: &mut Context, events: &EventsLoop) -> Result<Self, CreationError> {
		let vk_surface = WindowBuilder::new()
			.with_dimensions((1440, 810).into())
			.with_title("nIce Engine")
			.build_vk_surface(events, ctx.instance.clone())?;
		let surface = Surface::from_vk(ctx, vk_surface);
		Ok(Self { surface })
	}

	pub fn surface(&mut self) -> &mut Surface<winit::Window> {
		&mut self.surface
	}
}
