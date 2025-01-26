#![deny(clippy::all)]

use crate::{
    execute_command::{CommandExecutor, ExecutionResult},
    gui::{AppEvents, Gui},
};
use clap::Parser;
use log::{error, info, warn, LevelFilter};
use servicepoint::*;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::{mpsc, RwLock};
use std::time::Duration;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use cli::Cli;
use crate::font_renderer::FontRenderer8x8;

mod execute_command;
mod cp437_font;
mod font_renderer;
mod gui;
mod cli;

const BUF_SIZE: usize = 8985;

fn main() {
    let mut cli = Cli::parse();
    if !(cli.gui.red || cli.gui.blue || cli.gui.green) {
        cli.gui.green = true;
    }

    init_logging(cli.debug);
    info!("starting with args: {:?}", &cli);

    let socket = UdpSocket::bind(&cli.bind).expect("could not bind socket");
    socket
        .set_nonblocking(true)
        .expect("could not enter non blocking mode");

    let display = RwLock::new(Bitmap::new(PIXEL_WIDTH, PIXEL_HEIGHT));
    let luma = RwLock::new(BrightnessGrid::new(TILE_WIDTH, TILE_HEIGHT));

    let (stop_udp_tx, stop_udp_rx) = mpsc::channel();
    let mut gui = Gui::new(&display, &luma, stop_udp_tx, cli.gui);

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("could not create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let event_proxy = event_loop.create_proxy();
    let font_renderer = cli.font.map(move |font| FontRenderer8x8::from_name(font))
        .unwrap_or_else(move || FontRenderer8x8::default());
    let command_executor = CommandExecutor::new(&display, &luma, font_renderer);

    std::thread::scope(move |scope| {
        scope.spawn(move || {
            let mut buf = [0; BUF_SIZE];
            while stop_udp_rx.try_recv().is_err() {
                let amount = match receive_into_buf(&socket, &mut buf) {
                    Some(value) => value,
                    None => continue,
                };

                let command = match command_from_slice(&buf[..amount]) {
                    Some(value) => value,
                    None => continue,
                };

                handle_command(&event_proxy, &command_executor, command);
            }
        });
        event_loop
            .run_app(&mut gui)
            .expect("could not run event loop");
    });
}

fn handle_command(event_proxy: &EventLoopProxy<AppEvents>, command_executor: &CommandExecutor, command: Command) {
    match command_executor.execute(command) {
        ExecutionResult::Success => {
            event_proxy
                .send_event(AppEvents::UdpPacketHandled)
                .expect("could not send packet handled event");
        }
        ExecutionResult::Failure => {
            error!("failed to execute command");
        }
        ExecutionResult::Shutdown => {
            event_proxy
                .send_event(AppEvents::UdpThreadClosed)
                .expect("could not send close event");
        }
    }
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

fn command_from_slice(slice: &[u8]) -> Option<Command> {
    let package = match servicepoint::Packet::try_from(slice) {
        Err(_) => {
            warn!("could not load packet with length {}", slice.len());
            return None;
        }
        Ok(package) => package,
    };

    let command = match Command::try_from(package) {
        Err(err) => {
            warn!("could not read command for packet: {:?}", err);
            return None;
        }
        Ok(val) => val,
    };
    Some(command)
}

fn receive_into_buf(
    socket: &UdpSocket,
    buf: &mut [u8; BUF_SIZE],
) -> Option<usize> {
    let (amount, _) = match socket.recv_from(buf) {
        Err(err) if err.kind() == ErrorKind::WouldBlock => {
            std::thread::sleep(Duration::from_millis(1));
            return None;
        }
        Ok(result) => result,
        other => other.unwrap(),
    };

    if amount == buf.len() {
        warn!(
            "the received package may have been truncated to a length of {}",
            amount
        );
    }
    Some(amount)
}
