use crate::font::BitmapFont;
use crate::protocol::{HdrWindow, ReadHeaderError};
use crate::DISPLAY;
use log::{debug, error, info, warn};
use servicepoint2::{PixelGrid, PIXEL_WIDTH, TILE_SIZE, DisplayCommandCode};
use std::io::ErrorKind;
use std::net::{ToSocketAddrs, UdpSocket};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct UdpThread {
    thread: JoinHandle<()>,
    stop_tx: Sender<()>,
}

impl UdpThread {
    pub fn start_new(bind: impl ToSocketAddrs) -> Self {
        let (stop_tx, stop_rx) = mpsc::channel();

        let socket = UdpSocket::bind(bind).expect("could not bind socket");
        socket
            .set_nonblocking(true)
            .expect("could not enter non blocking mode");

        let font = BitmapFont::load_file("Web437_IBM_BIOS.woff");

        let thread = thread::spawn(move || {
            let mut buf = [0; 8985];

            while stop_rx.try_recv().is_err() {
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

                Self::handle_package(&mut buf[..amount], &font);
            }
        });

        return Self { stop_tx, thread };
    }

    pub fn stop_and_wait(self) {
        self.stop_tx.send(()).expect("could not send stop packet");
        self.thread.join().expect("could not wait on udp thread");
    }

    fn handle_package(received: &mut [u8], font: &BitmapFont) {
        let header = match HdrWindow::from_buffer(&received[..10]) {
            Err(ReadHeaderError::BufferTooSmall) => {
                error!("received a packet that is too small");
                return;
            }
            Err(ReadHeaderError::InvalidCommand(command_u16)) => {
                error!("received invalid command {}", command_u16);
                return;
            }
            Err(ReadHeaderError::WrongCommandEndianness(command_u16, command_swapped)) => {
                error!(
                "The reversed byte order of {} matches command {:?}, you are probably sending the wrong endianness",
                command_u16, command_swapped
            );
                return;
            }
            Ok(value) => value,
        };

        let payload = &received[10..];

        info!(
            "received from {:?} (and {} bytes of payload)",
            header,
            payload.len()
        );

        match header.command {
            DisplayCommandCode::Clear => {
                info!("clearing display");
                for v in unsafe { DISPLAY.iter_mut() } {
                    *v = false;
                }
            }
            DisplayCommandCode::HardReset => {
                warn!("display shutting down");
                return;
            }
            DisplayCommandCode::BitmapLinearWin => {
                Self::print_bitmap_linear_win(&header, payload);
            }
            DisplayCommandCode::Cp437data => {
                Self::print_cp437_data(&header, payload, font);
            }
            _ => {
                error!("command {:?} not implemented yet", header.command);
            }
        }
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

    fn print_bitmap_linear_win(header: &HdrWindow, payload: &[u8]) {
        if !Self::check_payload_size(payload, header.w as usize * header.h as usize) {
            return;
        }

        let pixel_grid = PixelGrid::load(
            header.w as usize * TILE_SIZE as usize,
            header.h as usize,
            payload,
        );

        Self::print_pixel_grid(
            header.x as usize * TILE_SIZE as usize,
            header.y as usize,
            &pixel_grid,
        );
    }

    fn print_cp437_data(header: &HdrWindow, payload: &[u8], font: &BitmapFont) {
        if !UdpThread::check_payload_size(payload, (header.w * header.h) as usize) {
            return;
        }

        for char_y in 0usize..header.h as usize {
            for char_x in 0usize..header.w as usize {
                let char_code = payload[char_y * header.w as usize + char_x];

                let tile_x = char_x + header.x as usize;
                let tile_y = char_y + header.y as usize;

                let bitmap = font.get_bitmap(char_code);
                Self::print_pixel_grid(
                    tile_x * TILE_SIZE as usize,
                    tile_y * TILE_SIZE as usize,
                    bitmap,
                );
            }
        }
    }

    fn print_pixel_grid(offset_x: usize, offset_y: usize, pixels: &PixelGrid) {
        debug!("printing {}x{} grid at {offset_x} {offset_y}", pixels.width, pixels.height);
        for inner_y in 0..pixels.height {
            for inner_x in 0..pixels.width {
                let is_set = pixels.get(inner_x, inner_y);
                let display_index =
                    (offset_x + inner_x) + ((offset_y + inner_y) * PIXEL_WIDTH as usize);
                unsafe {
                    DISPLAY[display_index] = is_set;
                }
            }
        }
    }
}
