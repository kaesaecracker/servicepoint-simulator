#![deny(clippy::all)]

mod gui;
mod upd_loop;

use std::default::Default;
use std::sync::mpsc;

use crate::gui::App;
use crate::upd_loop::start_udp_thread;
use clap::Parser;
use log::info;
use pixel_shared_rs::PIXEL_COUNT;
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long = "bind", default_value = "0.0.0.0:2342")]
    bind: String,
}

static mut DISPLAY: [bool; PIXEL_COUNT] = [false; PIXEL_COUNT];

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    info!("starting with args: {:?}", &cli);

    let (stop_udp_tx, stop_udp_rx) = mpsc::channel();
    let thread = start_udp_thread(cli.bind, stop_udp_rx);

    let event_loop = EventLoop::new().expect("could not create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("could not run event loop");

    stop_udp_tx.send(()).expect("could not cancel thread");
    thread.join().expect("could not join threads");
}
