#![deny(clippy::all)]

use crate::execute_command::CommandExecutor;
use crate::gui::{App, AppEvents};
use clap::Parser;
use log::{info, warn, LevelFilter};
use servicepoint::*;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::{mpsc, RwLock};
use std::time::Duration;
use winit::event_loop::{ControlFlow, EventLoop};

mod execute_command;
mod font;
mod font_renderer;
mod gui;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:2342")]
    bind: String,
    #[arg(short, long, default_value_t = false)]
    spacers: bool,
    #[arg(short, long, default_value_t = false)]
    red: bool,
    #[arg(short, long, default_value_t = false)]
    green: bool,
    #[arg(short, long, default_value_t = false)]
    blue: bool,
}

const BUF_SIZE: usize = 8985;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let mut cli = Cli::parse();
    if !(cli.red || cli.blue || cli.green) {
        cli.green = true;
    }

    info!("starting with args: {:?}", &cli);
    let socket = UdpSocket::bind(&cli.bind).expect("could not bind socket");
    socket
        .set_nonblocking(true)
        .expect("could not enter non blocking mode");

    let display = RwLock::new(Bitmap::new(PIXEL_WIDTH, PIXEL_HEIGHT));

    let mut luma = BrightnessGrid::new(TILE_WIDTH, TILE_HEIGHT);
    luma.fill(Brightness::MAX);
    let luma = RwLock::new(luma);

    let (stop_udp_tx, stop_udp_rx) = mpsc::channel();
    let mut app = App::new(&display, &luma, stop_udp_tx, &cli);

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("could not create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let event_proxy = event_loop.create_proxy();
    let command_executor = CommandExecutor::new(&display, &luma);

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

                if !command_executor.execute(command) {
                    // hard reset
                    event_proxy
                        .send_event(AppEvents::UdpThreadClosed)
                        .expect("could not send close event");
                    break;
                }

                event_proxy
                    .send_event(AppEvents::UdpPacketHandled)
                    .expect("could not send packet handled event");
            }
        });
        event_loop
            .run_app(&mut app)
            .expect("could not run event loop");
    });
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
