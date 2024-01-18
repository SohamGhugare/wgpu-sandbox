use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run() {
    // defining event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // setting control flow
    event_loop.set_control_flow(ControlFlow::Wait);

    // looping the window
    event_loop.run(move |event, elwt| {
        // handling all events
        match event {
            Event::WindowEvent { 
                event: WindowEvent::CloseRequested,
                ..
             } => {
                println!("Close button was pressed; stopping...");
                elwt.exit();
             },
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

