use std::{default, sync::Arc};

use wgpu::{Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptionsBase, wgt::DeviceDescriptor};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}};

struct State {
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    size: PhysicalSize<u32>
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let instance = Instance::new(
            &InstanceDescriptor::default(),
        );

        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase::default())
            .await.unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await.unwrap();

        let size = window.inner_size();

        State {
            window,
            device,
            queue,
            size
        }
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
    }

    fn render(&mut self) {

    }
}

#[derive(Default)]
struct App {
    state: Option<State>
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap()
        );

        let state = pollster::block_on(State::new(
            window.clone()
        ));
        self.state = Some(state);
        
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
