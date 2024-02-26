#![allow(dead_code)]

mod sound;
mod synth;

use tokio::runtime::Builder;
use tracing::instrument;
pub use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub mod backend;

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let rt = Builder::new_multi_thread().build()?;
    let _logic_task = rt.spawn(logic());
    build_window();
    println!("Hello, world!");
    Ok(())
}

#[instrument]
async fn logic() {}

fn build_window() {
    let evntloop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&evntloop).unwrap();
    window.set_title("ALIEN DAW");
    //rerender after every event
    evntloop.set_control_flow(ControlFlow::Wait);

    match evntloop.run(move |event, elwt| {
        //match on event
        //clippy suggests using if let
        //there might be more patterns to destructure later
        //so I'll leave as is
        match event {
            //match on events that happen in the window
            Event::WindowEvent {
                ref event,
                window_id: _,
            } => match event {
                WindowEvent::CloseRequested {} => elwt.exit(),
                WindowEvent::Resized(PhysicalSize {
                    width: _,
                    height: _,
                }) => {} //cant find logic to resize the window
                _ => {}
            },
            _ => {}
        }
    }) {
        Ok(()) => {} //event loop runs okay
        Err(err) => {
            eprintln!("Error MSG: {}", err)
        }
    };
}
