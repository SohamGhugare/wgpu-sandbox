pub mod state;

use std::ops::MulAssign;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use crate::state::State;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
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

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;

        window.set_min_inner_size(Some(PhysicalSize::new(450, 400)));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas().unwrap());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(window).await;

    // looping the window
    event_loop
        .run(move |event, elwt| {
            // handling all events
            match event {
                Event::WindowEvent {
                    window_id,
                    ref event,
                } if window_id == state.window().id() => match event {
                    // close event
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => elwt.exit(),
                    // resize event
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    // scale factor changed (related to dpi changes)
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        let current_inner_size = state.window().inner_size();
                        let new_inner_size: winit::dpi::PhysicalSize<u32> = PhysicalSize::new(
                            current_inner_size.width * *scale_factor as u32,
                            current_inner_size.height * *scale_factor as u32,
                        );
                        state.resize(new_inner_size);
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .unwrap();
}
