#![deny(clippy::all)]

mod font;
mod gui;

use crate::font::BitmapFont;
use crate::gui::App;
use clap::Parser;
use log::{debug, error, info, warn};
use servicepoint2::{
    Command, Origin, Packet, PixelGrid, Size, Window, PIXEL_HEIGHT, PIXEL_WIDTH, TILE_SIZE,
};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::{mpsc, RwLock, RwLockWriteGuard};
use std::thread;
use std::time::Duration;
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:2342")]
    bind: String,
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    info!("starting with args: {:?}", &cli);

    let socket = UdpSocket::bind(cli.bind).expect("could not bind socket");
    socket
        .set_nonblocking(true)
        .expect("could not enter non blocking mode");

    let font = BitmapFont::load_file("Web437_IBM_BIOS.woff");

    let display = PixelGrid::new(PIXEL_WIDTH as usize, PIXEL_HEIGHT as usize);
    let display_locked = RwLock::new(display);
    let display_locked_ref = &display_locked;

    std::thread::scope(move |scope| {
        let (stop_udp_tx, stop_udp_rx) = mpsc::channel();
        let (stop_ui_tx, stop_ui_rx) = mpsc::channel();

        let udp_thread = scope.spawn(move || {
            let mut buf = [0; 8985];

            while stop_udp_rx.try_recv().is_err() {
                let (amount, _) = match socket.recv_from(&mut buf) {
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                        continue;
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

                let vec = buf[..amount].to_vec();
                let package = servicepoint2::Packet::from(vec);

                let mut display = display_locked_ref.write().unwrap();
                handle_package(package, &font, &mut display);
            }

            stop_ui_tx.send(()).expect("could not stop ui thread");
        });

        let mut app = App::new(display_locked_ref, stop_ui_rx);

        let event_loop = EventLoop::new().expect("could not create event loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop
            .run_app(&mut app)
            .expect("could not run event loop");

        stop_udp_tx.send(()).expect("could not stop udp thread");

        udp_thread.join().expect("could not join udp thread");
    });

    fn handle_package(
        received: Packet,
        font: &BitmapFont,
        display: &mut RwLockWriteGuard<PixelGrid>,
    ) {
        // TODO handle error case
        let command = match Command::try_from(received) {
            Err(err) => {
                warn!("could not read command for packet: {:?}", err);
                return;
            }
            Ok(val) => val,
        };

        match command {
            Command::Clear => {
                info!("clearing display");
                display.fill(false);
            }
            Command::HardReset => {
                warn!("display shutting down");
                return;
            }
            Command::BitmapLinearWin(Origin(x, y), pixels) => {
                print_pixel_grid(x as usize, y as usize, &pixels, display);
            }
            Command::Cp437Data(window, payload) => {
                print_cp437_data(window, &payload, font, display);
            }
            Command::BitmapLegacy => {
                warn!("ignoring deprecated command {:?}", command);
            }
            Command::BitmapLinear(offset, vec) => {}
            Command::BitmapLinearAnd(_, _) => {}
            Command::BitmapLinearOr(_, _) => {}
            Command::BitmapLinearXor(_, _) => {}

            Command::FadeOut => {}
            Command::CharBrightness(_, _) => {}
            Command::Brightness(_) => {}
        };
    }

    fn check_payload_size(buf: &[u8], expected: usize) -> bool {
        let actual = buf.len();
        if actual == expected {
            return true;
        }

        error!(
            "expected a payload length of {} but got {}",
            expected, actual
        );
        return false;
    }

    fn print_cp437_data(
        window: Window,
        payload: &[u8],
        font: &BitmapFont,
        display: &mut RwLockWriteGuard<PixelGrid>,
    ) {
        let Window(Origin(x, y), Size(w, h)) = window;
        if !check_payload_size(payload, (w * h) as usize) {
            return;
        }

        for char_y in 0usize..h as usize {
            for char_x in 0usize..w as usize {
                let char_code = payload[char_y * w as usize + char_x];

                let tile_x = char_x + x as usize;
                let tile_y = char_y + y as usize;

                let bitmap = font.get_bitmap(char_code);
                print_pixel_grid(
                    tile_x * TILE_SIZE as usize,
                    tile_y * TILE_SIZE as usize,
                    bitmap,
                    display,
                );
            }
        }
    }

    fn print_pixel_grid(
        offset_x: usize,
        offset_y: usize,
        pixels: &PixelGrid,
        display: &mut RwLockWriteGuard<PixelGrid>,
    ) {
        debug!(
            "printing {}x{} grid at {offset_x} {offset_y}",
            pixels.width, pixels.height
        );
        for inner_y in 0..pixels.height {
            for inner_x in 0..pixels.width {
                let is_set = pixels.get(inner_x, inner_y);
                display.set(offset_x + inner_x, offset_y + inner_y, is_set);
            }
        }
    }
}
