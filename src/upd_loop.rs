use crate::DISPLAY;
use log::{error, info, warn};
use pixel_shared_rs::{
    read_header, DisplayCommand, HdrWindow, ReadHeaderError, PIXEL_WIDTH, TILE_SIZE,
};
use std::io::ErrorKind;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub fn start_udp_thread(bind: String, stop_receiver: Receiver<()>) -> JoinHandle<()> {
    assert_eq!(size_of::<HdrWindow>(), 10, "invalid struct size");

    return thread::spawn(move || {
        let socket = UdpSocket::bind(bind).expect("could not bind socket");
        socket
            .set_nonblocking(true)
            .expect("could not enter non blocking mode");

        let mut buf = [0; 8985];

        while stop_receiver.try_recv().is_err() {
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

            handle_package(&mut buf[..amount]);
        }
    });
}

fn handle_package(received: &mut [u8]) {
    let header = match read_header(&received[..10]) {
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
        DisplayCommand::CmdClear => {
            info!("clearing display");
            for v in unsafe { DISPLAY.iter_mut() } {
                *v = false;
            }
        }
        DisplayCommand::CmdHardReset => {
            warn!("display shutting down");
            return;
        }
        DisplayCommand::CmdBitmapLinearWin => {
            print_bitmap_linear_win(&header, payload);
        }
        DisplayCommand::CmdCp437data => {
            print_cp437_data(&header, payload);
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
    if !check_payload_size(payload, (header.w * header.h) as usize) {
        return;
    }

    info!(
        "top left is offset {} tiles in x-direction and {} pixels in y-direction",
        header.x, header.y
    );

    let mut text_repr = String::new();

    for y in 0..header.h {
        for byte_x in 0..header.w {
            let byte_index = (y * header.w + byte_x) as usize;
            let byte = payload[byte_index];

            for pixel_x in 1u8..=8u8 {
                let bit_index = 8 - pixel_x;
                let bitmask = 1 << bit_index;
                let is_set = byte & bitmask != 0;
                let char = if is_set { '█' } else { ' ' };
                text_repr.push(char);

                let x = byte_x * TILE_SIZE + pixel_x as u16;

                let translated_x = (x + header.x) as usize;
                let translated_y = (y + header.y) as usize;
                let index = translated_y * PIXEL_WIDTH as usize + translated_x;

                unsafe {
                    DISPLAY[index] = is_set;
                }
            }
        }

        text_repr.push('\n');
    }
    info!("{}", text_repr);
}

// TODO: actually convert from CP437
fn print_cp437_data(header: &HdrWindow, payload: &[u8]) {
    if !check_payload_size(payload, (header.w * header.h) as usize) {
        return;
    }

    info!("top left is offset by ({} | {}) tiles", header.x, header.y);

    let mut str = String::new();
    for y in 0..header.h {
        for x in 0..header.w {
            let byte_index = (y * header.w + x) as usize;
            str.push(payload[byte_index] as char);
        }

        str.push('\n');
    }

    info!("{}", str);
}
