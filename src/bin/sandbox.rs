use cgmath::vec3;
use nice_engine::{
	camera::Camera,
	mesh::Mesh,
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

	let mesh = Mesh::new();
	mesh.set_mesh_data(Some(triangle));

	let batch = MeshBatch::new();
	batch.insert_mesh(mesh);

	let mut cam = Camera::new();
	cam.set_mesh_batch(Some(batch.clone()));

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

		cam.transform_mut().pos += vec3(0.001, 0.0, 0.0);

		win.surface().draw(&cam);

		if done {
			break;
		}
	}
}
