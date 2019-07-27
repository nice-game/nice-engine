use nice_engine::{
	mesh_batch::MeshBatch,
	mesh_data::{MeshData, Pntl_32F},
	window::Window,
	Context,
};
use vulkano::sync::GpuFuture;
use winit::{dpi::LogicalSize, Event, EventsLoop, WindowEvent};

pub fn main() {
	let mut ctx = Context::new(Some("nIce Engine"), None).unwrap();
	let mut events = EventsLoop::new();
	let mut win = Window::new(&mut ctx, &events).unwrap();

	let (triangle, triangle_future) = MeshData::new(
		&ctx,
		[
			Pntl_32F { pos: [-0.5, -0.25, 0.0], nor: [0.0; 3], tex: [0.0; 2], lmap: [0.0; 2] },
			Pntl_32F { pos: [0.0, 0.5, 0.0], nor: [0.0; 3], tex: [0.0; 2], lmap: [0.0; 2] },
			Pntl_32F { pos: [0.25, -0.1, 0.0], nor: [0.0; 3], tex: [0.0; 2], lmap: [0.0; 2] },
		],
		[0, 1, 2],
	)
	.unwrap();
	triangle_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

	let _mb = MeshBatch::new(&ctx);

	loop {
		let mut done = false;
		events.poll_events(|event| match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => done = true,
				WindowEvent::Resized(LogicalSize { width, height }) => {
					win.surface().resize(width as u32, height as u32)
				},
				_ => (),
			},
			_ => (),
		});

		win.surface().draw(&triangle);

		if done {
			break;
		}
	}
}
