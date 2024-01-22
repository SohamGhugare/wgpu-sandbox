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

