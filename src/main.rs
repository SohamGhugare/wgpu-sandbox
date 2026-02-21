mod shape;
mod state;

use clap::Parser;
use shape::{Shape, ShapeConfig};
use state::State;
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[derive(Parser)]
#[command(name = "wgpu-sandbox")]
struct Cli {
    #[arg(long, value_enum, default_value = "triangle")]
    shape: Shape,

    #[arg(long, default_value = "red")]
    color: String,

    #[arg(long, default_value_t = 0.5)]
    size: f32,
}

fn parse_color(s: &str) -> [f32; 4] {
    match s.to_lowercase().as_str() {
        "red"     => [1.0, 0.0, 0.0, 1.0],
        "green"   => [0.0, 1.0, 0.0, 1.0],
        "blue"    => [0.0, 0.0, 1.0, 1.0],
        "white"   => [1.0, 1.0, 1.0, 1.0],
        "black"   => [0.0, 0.0, 0.0, 1.0],
        "yellow"  => [1.0, 1.0, 0.0, 1.0],
        "cyan"    => [0.0, 1.0, 1.0, 1.0],
        "magenta" => [1.0, 0.0, 1.0, 1.0],
        "orange"  => [1.0, 0.5, 0.0, 1.0],
        "purple"  => [0.5, 0.0, 0.5, 1.0],
        _         => [1.0, 0.0, 0.0, 1.0],
    }
}

struct App {
    state: Option<State>,
    config: Option<ShapeConfig>,
}

impl App {
    fn new(config: ShapeConfig) -> Self {
        Self { state: None, config: Some(config) }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Cheesecake"))
                .unwrap(),
        );

        let config = self.config.take().unwrap();
        let state = pollster::block_on(State::new(window.clone(), config));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
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
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    let config = ShapeConfig {
        shape: cli.shape,
        color: parse_color(&cli.color),
        size: cli.size,
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(config);
    event_loop.run_app(&mut app).unwrap();
}
