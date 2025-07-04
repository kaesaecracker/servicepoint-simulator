#![deny(clippy::all)]

use crate::font_renderer::FontRenderer8x8;
use crate::udp_server::UdpServer;
use crate::{command_executor::CommandExecutionContext, gui::Gui};
use clap::Parser;
use cli::Cli;
use log::{info, LevelFilter};
use servicepoint::*;
use std::sync::{mpsc, RwLock};
use winit::event_loop::{ControlFlow, EventLoop};

mod cli;
mod command_executor;
mod cp437_font;
mod font_renderer;
mod gui;
mod gui_window;
mod udp_server;

fn main() {
    let mut cli = Cli::parse();
    if !(cli.gui.red || cli.gui.blue || cli.gui.green) {
        cli.gui.green = true;
    }

    init_logging(cli.verbose);
    info!("starting with args: {:?}", &cli);

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("could not create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let display = RwLock::new(Bitmap::max_sized());
    let luma = RwLock::new(BrightnessGrid::new(TILE_WIDTH, TILE_HEIGHT));
    let (stop_udp_tx, stop_udp_rx) = mpsc::channel();
    let font_renderer = cli
        .font
        .map(FontRenderer8x8::from_name)
        .unwrap_or_else(FontRenderer8x8::default);
    let context = CommandExecutionContext::new(&display, &luma, font_renderer);
    let mut udp_server = UdpServer::new(
        cli.bind,
        stop_udp_rx,
        context,
        event_loop.create_proxy(),
    );
    let mut gui = Gui::new(&display, &luma, stop_udp_tx, cli.gui);

    std::thread::scope(move |scope| {
        scope.spawn(move || udp_server.run());
        event_loop
            .run_app(&mut gui)
            .expect("could not run event loop");
    });
}

fn init_logging(debug: bool) {
    let filter = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    env_logger::builder()
        .filter_level(filter)
        .parse_default_env()
        .init();
}
