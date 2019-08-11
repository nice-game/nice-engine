#![cfg(feature = "window")]

use crate::{surface::Surface, Context};
use std::sync::Arc;
use vulkano_win::{CreationError, VkSurfaceBuild};
use winit::{EventsLoop, MouseCursor, WindowBuilder};

pub struct Window {
	surface: Surface<winit::Window>,
}
impl Window {
	pub fn new(ctx: &Arc<Context>, events: &EventsLoop) -> Result<Self, CreationError> {
		let vk_surface = WindowBuilder::new()
			.with_fullscreen(Some(events.get_primary_monitor()))
			.with_title("nIce Engine")
			.build_vk_surface(events, ctx.instance.clone())?;
		let surface = Surface::from_vk(ctx, vk_surface);
		Ok(Self { surface })
	}

	pub fn grab_cursor(&self, grab: bool) -> Result<(), String> {
		self.surface.window().grab_cursor(grab)
	}

	pub fn hide_cursor(&self, hide: bool) {
		self.surface.window().hide_cursor(hide);
	}

	pub fn set_cursor(&self, cursor: MouseCursor) {
		self.surface.window().set_cursor(cursor);
	}

	pub fn surface(&mut self) -> &mut Surface<winit::Window> {
		&mut self.surface
	}
}
