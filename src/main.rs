#![deny(clippy::all)]

mod font;
mod gui;
mod protocol;
mod upd_loop;

use std::default::Default;

use crate::gui::App;
use crate::upd_loop::UdpThread;
use clap::Parser;
use log::info;
use servicepoint2::PIXEL_COUNT;
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:2342")]
    bind: String,
}

static mut DISPLAY: [bool; PIXEL_COUNT] = [false; PIXEL_COUNT];

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    info!("starting with args: {:?}", &cli);

    let thread = UdpThread::start_new(cli.bind);

    let event_loop = EventLoop::new().expect("could not create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("could not run event loop");

    thread.stop_and_wait();
}
