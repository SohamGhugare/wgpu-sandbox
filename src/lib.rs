#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run() {

    // toggling logger for wasm32
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            tracing_subscriber::fmt::init();
        }
    }

    // defining event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // setting control flow
    event_loop.set_control_flow(ControlFlow::Wait);

    // looping the window
    event_loop.run(move |event, elwt| {
        // handling all events
        match event {
            // close window
            Event::WindowEvent { 
                event: WindowEvent::CloseRequested,
                ..
             } => {
                println!("Close button was pressed; stopping...");
                elwt.exit();
             },
             // waiting for new events
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent { 
                event: WindowEvent::RedrawRequested, 
                .. 
            } => {

            },
            _ => ()
        }
    }).unwrap();
}

