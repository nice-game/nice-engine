use cgmath::{prelude::*, vec3, Quaternion, Rad};
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
	batch.insert_mesh(mesh.clone());

	let mut cam = Camera::new();
	cam.transform_mut().pos = vec3(0.0, 0.0, 3.0);
	cam.set_perspective(16.0 / 9.0, 90.0, 1.0, 1000.0);
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

		let mut transform = mesh.transform();
		transform.rot = transform.rot * Quaternion::from_angle_y(Rad(0.01));
		mesh.set_transform(transform);

		win.surface().draw(&cam);

		if done {
			break;
		}
	}
}
