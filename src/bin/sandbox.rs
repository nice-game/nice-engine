use cgmath::{prelude::*, vec3, Quaternion, Rad};
use nice_engine::{
	camera::Camera,
	mesh::Mesh,
	mesh_data::{MeshData, Pntl_32F},
	texture::Texture,
	window::Window,
	Context,
};
use vulkano::{format::Format, image::Dimensions, sync::GpuFuture};
use winit::{dpi::LogicalSize, Event, EventsLoop, WindowEvent};

pub fn main() {
	let ctx = Context::new(Some("nIce Engine"), None).unwrap();
	let mut events = EventsLoop::new();
	let mut win = Window::new(&ctx, &events).unwrap();

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

	let (tex, tex_future) = Texture::from_iter(
		&ctx,
		vec![[0u8, 0, 255, 255], [0, 0, 0, 255], [0, 0, 0, 255], [0, 0, 255, 255]].into_iter(),
		Dimensions::Dim2d { width: 2, height: 2 },
		Format::R8G8B8A8Unorm,
	)
	.unwrap();

	triangle_future.join(tex_future).then_signal_fence_and_flush().unwrap().wait(None).unwrap();

	let mut mesh = Mesh::new();
	mesh.set_mesh_data(Some(triangle));

	let mut cam = Camera::new();
	cam.transform_mut().pos = vec3(0.0, 0.0, 3.0);
	cam.set_perspective(16.0 / 9.0, 90.0, 1.0, 1000.0);

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

		if done {
			break;
		}

		let transform = mesh.transform_mut();
		transform.rot = transform.rot * Quaternion::from_angle_y(Rad(0.01));

		win.surface().draw(&cam, &[&mesh]);
	}
}
