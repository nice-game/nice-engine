use cgmath::vec3;
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
	let (ctx, ctx_future) = Context::new(Some("nIce Engine"), None).unwrap();
	let mut events = EventsLoop::new();
	let mut win = Window::new(&ctx, &events).unwrap();

	let (triangle, triangle_future) = MeshData::new(
		&ctx,
		[
			Pntl_32F { pos: [-1.0, -1.0, 0.0], nor: [0.0; 3], tex: [0.0, 0.0], lmap: [0.0; 2] },
			Pntl_32F { pos: [1.0, -1.0, 0.0], nor: [0.0; 3], tex: [1.0, 0.0], lmap: [0.0; 2] },
			Pntl_32F { pos: [1.0, 1.0, 0.0], nor: [0.0; 3], tex: [1.0, 1.0], lmap: [0.0; 2] },
			Pntl_32F { pos: [-1.0, 1.0, 0.0], nor: [0.0; 3], tex: [0.0, 1.0], lmap: [0.0; 2] },
		],
		vec![0, 1, 2, 2, 3, 0].into_iter(),
	)
	.unwrap();

	let (tex, tex_future) = Texture::from_iter(
		&ctx,
		vec![[255u8, 0, 0, 255], [0, 0, 0, 255], [0, 0, 0, 255], [255, 0, 0, 255]].into_iter(),
		Dimensions::Dim2d { width: 2, height: 2 },
		Format::R8G8B8A8Unorm,
	)
	.unwrap();

	let mut mesh = Mesh::new(ctx.clone());
	mesh.set_mesh_data(Some(triangle));
	mesh.set_texture(&tex);

	let mut cam = Camera::new();
	cam.transform_mut().pos = vec3(0.0, 0.0, 3.0);
	cam.set_perspective(16.0 / 9.0, 90.0, 1.0, 50.0);

	ctx_future.join(triangle_future).join(tex_future).then_signal_fence_and_flush().unwrap().wait(None).unwrap();

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

		// let transform = mesh.transform_mut();
		// transform.rot = transform.rot * Quaternion::from_angle_y(Rad(0.01));

		win.surface().draw(&cam, &[&mesh]);
	}
}
