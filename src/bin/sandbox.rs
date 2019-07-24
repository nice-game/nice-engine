use nice_engine::{window::Window, Context};
use winit::{dpi::LogicalSize, ControlFlow, Event, EventsLoop, WindowEvent};

pub fn main() {
	let mut ctx = Context::new(Some("nIce Engine"), None).unwrap();
	let mut events = EventsLoop::new();
	let mut win = Window::new(&mut ctx, &events).unwrap();
	events.run_forever(|event| match event {
		Event::WindowEvent { event, .. } => match event {
			WindowEvent::CloseRequested => ControlFlow::Break,
			WindowEvent::Resized(LogicalSize { width, height }) => {
				win.resize(width as u32, height as u32);
				ControlFlow::Continue
			},
			_ => ControlFlow::Continue,
		},
		_ => ControlFlow::Continue,
	});
}
