use crate::font::BitmapFont;
use crate::DISPLAY;
use log::{debug, error, info, warn};
use servicepoint2::{Command, Origin, Packet, PixelGrid, PIXEL_WIDTH, TILE_SIZE, Window, Size};
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

                let vec = buf[..amount].to_vec();
                let package = servicepoint2::Packet::from(vec);

                Self::handle_package(package, &font);
            }
        });

        return Self { stop_tx, thread };
    }

    pub fn stop_and_wait(self) {
        self.stop_tx.send(()).expect("could not send stop packet");
        self.thread.join().expect("could not wait on udp thread");
    }

    fn handle_package(received: Packet, font: &BitmapFont) {
        // TODO handle error case
        let command = Command::try_from(received).unwrap();

        match command {
            Command::Clear => {
                info!("clearing display");
                for v in unsafe { DISPLAY.iter_mut() } {
                    *v = false;
                }
            }
            Command::HardReset => {
                warn!("display shutting down");
                return;
            }
            Command::BitmapLinearWin(Origin(x, y), pixels) => {
                Self::print_pixel_grid(x as usize, y as usize, &pixels);
            }
            Command::Cp437Data(window, payload) => {
                Self::print_cp437_data(window, &payload, font);
            }
            _ => {
                error!("command {:?} not implemented yet", command);
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

    fn print_cp437_data(window: Window, payload: &[u8], font: &BitmapFont) {
        let Window(Origin(x,y), Size(w, h)) = window;
        if !UdpThread::check_payload_size(payload, (w * h) as usize) {
            return;
        }

        for char_y in 0usize..h as usize {
            for char_x in 0usize..w as usize {
                let char_code = payload[char_y * w as usize + char_x];

                let tile_x = char_x + x as usize;
                let tile_y = char_y + y as usize;

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
        debug!(
            "printing {}x{} grid at {offset_x} {offset_y}",
            pixels.width, pixels.height
        );
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
