#![deny(clippy::all)]

mod font;
mod gui;

use crate::font::BitmapFont;
use crate::gui::App;
use clap::Parser;
use log::{debug, error, info, warn};
use servicepoint2::{
    ByteGrid, Command, Origin, Packet, PixelGrid, PIXEL_HEIGHT, PIXEL_WIDTH, TILE_HEIGHT,
    TILE_SIZE, TILE_WIDTH,
};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::{mpsc, RwLock, RwLockWriteGuard};
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

    let display = RwLock::new(PixelGrid::new(PIXEL_WIDTH as usize, PIXEL_HEIGHT as usize));
    let display_ref = &display;

    let mut luma = ByteGrid::new(TILE_WIDTH as usize, TILE_HEIGHT as usize);
    luma.fill(u8::MAX);
    let luma = RwLock::new(luma);
    let luma_ref = &luma;

    let (stop_udp_tx, stop_udp_rx) = mpsc::channel();
    let (stop_ui_tx, stop_ui_rx) = mpsc::channel();

    let mut app = App::new(display_ref, luma_ref, stop_ui_rx, stop_udp_tx);

    let event_loop = EventLoop::new().expect("could not create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    std::thread::scope(move |scope| {
        let udp_thread = scope.spawn(move || {
            let mut buf = [0; 8985];

            while stop_udp_rx.try_recv().is_err() {
                let (amount, _) = match socket.recv_from(&mut buf) {
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(1));
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

                if !handle_package(package, &font, display_ref, luma_ref) {
                    break; // hard reset
                }
            }

            stop_ui_tx.send(()).expect("could not stop ui thread");
        });

        event_loop
            .run_app(&mut app)
            .expect("could not run event loop");

        udp_thread.join().expect("could not join udp thread");
    });
}

fn handle_package(
    received: Packet,
    font: &BitmapFont,
    display_ref: &RwLock<PixelGrid>,
    luma_ref: &RwLock<ByteGrid>,
) -> bool {
    let command = match Command::try_from(received) {
        Err(err) => {
            warn!("could not read command for packet: {:?}", err);
            return true;
        }
        Ok(val) => val,
    };

    debug!("received {command:?}");
    match command {
        Command::Clear => {
            info!("clearing display");
            display_ref.write().unwrap().fill(false);
        }
        Command::HardReset => {
            warn!("display shutting down");
            return false;
        }
        Command::BitmapLinearWin(Origin(x, y), pixels) => {
            let mut display = display_ref.write().unwrap();
            print_pixel_grid(x as usize, y as usize, &pixels, &mut display);
        }
        Command::Cp437Data(origin, grid) => {
            let mut display = display_ref.write().unwrap();
            print_cp437_data(origin, &grid, font, &mut display);
        }
        #[allow(deprecated)]
        Command::BitmapLegacy => {
            warn!("ignoring deprecated command {:?}", command);
        }
        // TODO: how to deduplicate this code in a rusty way?
        Command::BitmapLinear(offset, vec) => {
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let pixel_index = offset as usize + bitmap_index;
                let y = pixel_index / PIXEL_WIDTH as usize;
                let x = pixel_index % PIXEL_WIDTH as usize;
                display.set(x, y, vec.get(bitmap_index));
            }
        }
        Command::BitmapLinearAnd(offset, vec) => {
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let pixel_index = offset as usize + bitmap_index;
                let y = pixel_index / PIXEL_WIDTH as usize;
                let x = pixel_index % PIXEL_WIDTH as usize;
                let old_value = display.get(x, y);
                display.set(x, y, old_value && vec.get(bitmap_index));
            }
        }
        Command::BitmapLinearOr(offset, vec) => {
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let pixel_index = offset as usize + bitmap_index;
                let y = pixel_index / PIXEL_WIDTH as usize;
                let x = pixel_index % PIXEL_WIDTH as usize;
                let old_value = display.get(x, y);
                display.set(x, y, old_value || vec.get(bitmap_index));
            }
        }
        Command::BitmapLinearXor(offset, vec) => {
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let pixel_index = offset as usize + bitmap_index;
                let y = pixel_index / PIXEL_WIDTH as usize;
                let x = pixel_index % PIXEL_WIDTH as usize;
                let old_value = display.get(x, y);
                display.set(x, y, old_value ^ vec.get(bitmap_index));
            }
        }
        Command::CharBrightness(origin, grid) => {
            let Origin(offset_x, offset_y) = origin;
            let offset_x = offset_x as usize;
            let offset_y = offset_y as usize;

            let mut luma = luma_ref.write().unwrap();
            for inner_y in 0..grid.height {
                for inner_x in 0..grid.width {
                    let brightness = grid.get(inner_x, inner_y);
                    luma.set(offset_x + inner_x, offset_y + inner_y, brightness);
                }
            }
        }
        Command::Brightness(brightness) => {
            luma_ref.write().unwrap().fill(brightness);
        }
        Command::FadeOut => {
            error!("command not implemented: {command:?}")
        }
    };

    true
}

fn print_cp437_data(
    origin: Origin,
    grid: &ByteGrid,
    font: &BitmapFont,
    display: &mut RwLockWriteGuard<PixelGrid>,
) {
    let Origin(x, y) = origin;
    for char_y in 0usize..grid.height {
        for char_x in 0usize..grid.width {
            let char_code = grid.get(char_x, char_y);

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
