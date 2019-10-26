#![cfg(feature = "window")]

use crate::{surface::Surface, Context};
use std::sync::Arc;
use vulkano_win::{CreationError, VkSurfaceBuild};
use winit::{
	error::ExternalError,
	event_loop::EventLoop,
	window::{Fullscreen, WindowBuilder},
};

pub struct Window {
	surface: Surface<winit::window::Window>,
}
impl Window {
	pub fn new(ctx: &Arc<Context>, events: &EventLoop<()>) -> Result<Self, CreationError> {
		let vk_surface = WindowBuilder::new()
			.with_fullscreen(Some(Fullscreen::Borderless(events.primary_monitor())))
			// .with_dimensions((1920, 1080).into())
			.with_title("nIce Engine")
			.build_vk_surface(events, ctx.instance.clone())?;
		let surface = Surface::from_vk(ctx, vk_surface);
		Ok(Self { surface })
	}

	pub fn set_cursor_grab(&self, grab: bool) -> Result<(), ExternalError> {
		self.surface.window().set_cursor_grab(grab)
	}

	pub fn set_cursor_visible(&self, visible: bool) {
		self.surface.window().set_cursor_visible(visible);
	}

	pub fn surface(&mut self) -> &mut Surface<winit::window::Window> {
		&mut self.surface
	}
}
