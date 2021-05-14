use winit::{
    event::{Event, ModifiersState, WindowEvent, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub(crate) mod app;
pub(crate) mod model;
pub(crate) mod backdrop;

use crate::app::App;

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();
    let (surface, adapter) = {
        let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        (surface, adapter)
    };

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let mut app = App::new(size, adapter, surface, device);
    let mut modifiers = ModifiersState::empty();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    app.resize(size);
                    app.redraw(&queue);
                },
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ModifiersChanged(m) => {
                    modifiers = m;
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    if modifiers.logo() && input.virtual_keycode == Some(VirtualKeyCode::Q) {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(_) => app.redraw(&queue),
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    env_logger::init();
    pollster::block_on(run(event_loop, window));
}
