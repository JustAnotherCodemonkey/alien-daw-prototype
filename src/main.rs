#![allow(dead_code)]

pub use std::sync::Arc;

mod sound;
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
    tokio::runtime::Runtime::block_on(&rt, build_window()); //put a cfg flag here for wasm. browsers execute futures natively
    println!("Hello, world!");
    Ok(())
}

#[instrument]
async fn logic() {}

async fn build_window() {
    let evntloop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&evntloop).unwrap();
    window.set_title("ALIEN DAW");
    //rerender after every event
    evntloop.set_control_flow(ControlFlow::Wait);
    let mut state = backend::CurrentWindowState::new(window.into()).await;

    match evntloop.run(move |event, elwt| {
        //match on event
        //clippy suggests using if let
        //there might be more patterns to destructure later
        //so I'll leave as is
        match event {
            //match on events that happen in the window
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested {} => elwt.exit(),
                        WindowEvent::Resized(size) => state.resize(*size),
                        WindowEvent::ScaleFactorChanged {
                            scale_factor,
                            inner_size_writer,
                        } => {
                            state.resize(todo!());
                        } //open an issue or make a pull req. the docs have not been updated
                        WindowEvent::RedrawRequested =>{
                            //state.update();
                            match state.render(){
                                Ok(_) => {},
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.get_size()),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{}", e)
                            }
                        },
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }) {
        Ok(()) => {} //event loop runs okay
        Err(err) => {
            eprintln!("Error MSG: {}", err)
        }
    };
}
