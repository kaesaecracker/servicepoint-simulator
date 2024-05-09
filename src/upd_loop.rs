use crate::{DISPLAY, PIXEL_WIDTH, TILE_SIZE};
use log::{error, info, warn};
use num_derive::FromPrimitive;
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

            handle_package(&mut buf[..amount]);
        }
    });
}

#[derive(Debug, FromPrimitive)]
enum DisplayCommand {
    CmdClear = 0x0002,
    CmdCp437data = 0x0003,
    CmdCharBrightness = 0x0005,
    CmdBrightness = 0x0007,
    CmdHardReset = 0x000b,
    CmdFadeOut = 0x000d,
    CmdBitmapLegacy = 0x0010,
    CmdBitmapLinear = 0x0012,
    CmdBitmapLinearWin = 0x0013,
    CmdBitmapLinearAnd = 0x0014,
    CmdBitmapLinearOr = 0x0015,
    CmdBitmapLinearXor = 0x0016,
}


#[repr(C)]
#[derive(Debug)]
struct HdrWindow {
    command: DisplayCommand,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

/* needed for commands that are not implemented yet
#[repr(C)]
struct HdrBitmap {
    command: DisplayCommand,
    offset: u16,
    length: u16,
    subcommand: DisplaySubcommand,
    reserved: u16,
}

#[repr(u16)]
enum DisplaySubcommand {
    SubCmdBitmapNormal = 0x0,
    SubCmdBitmapCompressZ = 0x677a,
    SubCmdBitmapCompressBz = 0x627a,
    SubCmdBitmapCompressLz = 0x6c7a,
    SubCmdBitmapCompressZs = 0x7a73,
}
*/

fn handle_package(received: &mut [u8]) {
    let header = read_hdr_window(&received[..10]);
    if let Err(err) = header {
        warn!("could not read header: {}", err);
        return;
    }

    let header = header.unwrap();
    let payload = &received[10..];

    info!(
        "received from {:?} (and {} bytes of payload)",
        header,
        payload.len()
    );

    match header.command {
        DisplayCommand::CmdClear => {
            info!("(imagine an empty screen now)")
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

fn read_hdr_window(buffer: &[u8]) -> Result<HdrWindow, String> {
    if buffer.len() < size_of::<HdrWindow>() {
        return Err("received a packet that is too small".into());
    }

    let command_u16 = u16::from_be(unsafe { std::ptr::read(buffer[0..=1].as_ptr() as *const u16) });
    let maybe_command = num::FromPrimitive::from_u16(command_u16);
    if maybe_command.is_none() {
        return Err(format!("received invalid command {}", command_u16));
    }

    return Ok(HdrWindow {
        command: maybe_command.unwrap(),
        x: u16::from_be(unsafe { std::ptr::read(buffer[2..=3].as_ptr() as *const u16) }),
        y: u16::from_be(unsafe { std::ptr::read(buffer[4..=5].as_ptr() as *const u16) }),
        w: u16::from_be(unsafe { std::ptr::read(buffer[6..=7].as_ptr() as *const u16) }),
        h: u16::from_be(unsafe { std::ptr::read(buffer[8..=9].as_ptr() as *const u16) }),
    });
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

                unsafe {
                    DISPLAY[translated_y * PIXEL_WIDTH as usize + translated_x] = is_set;
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
