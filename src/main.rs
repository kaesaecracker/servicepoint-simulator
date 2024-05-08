use std::mem::size_of;
use std::net::{UdpSocket};

use clap::Parser;
use num_derive::FromPrimitive;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long = "bind", default_value = "0.0.0.0:2342")]
    bind: String,
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

#[repr(u16)]
enum DisplaySubcommand {
    SubCmdBitmapNormal = 0x0,
    SubCmdBitmapCompressZ = 0x677a,
    SubCmdBitmapCompressBz = 0x627a,
    SubCmdBitmapCompressLz = 0x6c7a,
    SubCmdBitmapCompressZs = 0x7a73,
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

#[repr(C)]
struct HdrBitmap {
    command: DisplayCommand,
    offset: u16,
    length: u16,
    subcommand: u16,
    reserved: u16,
}

fn main() -> std::io::Result<()> {
    assert_eq!(size_of::<HdrWindow>(), 10, "invalid struct size");

    let cli = Cli::parse();
    println!("running with args: {:?}", &cli);

    loop {
        // to emulate a hard reset, the actual main method gets called until it crashes
        main2(&cli).unwrap();
    }
}

fn main2(cli: &Cli) -> std::io::Result<()> {
    println!("display booting up");

    let socket = UdpSocket::bind(&cli.bind)?;
    let mut buf = [0; 8985];

    loop {
        let (amount, source) = socket.recv_from(&mut buf)?;
        let received = &mut buf[..amount];

        if amount < size_of::<HdrWindow>() {
            println!("received a packet that is too small from {:?}", source);
            continue;
        }

        let command_u16 =
            u16::from_be(unsafe { std::ptr::read(received[0..=1].as_ptr() as *const u16) });
        let maybe_command = num::FromPrimitive::from_u16(command_u16);
        if maybe_command.is_none() {
            println!(
                "command {} received from {:?} is invalid",
                command_u16, source
            );
            continue;
        }

        let header: HdrWindow = HdrWindow {
            command: maybe_command.unwrap(),
            x: u16::from_be(unsafe { std::ptr::read(received[2..=3].as_ptr() as *const u16) }),
            y: u16::from_be(unsafe { std::ptr::read(received[4..=5].as_ptr() as *const u16) }),
            w: u16::from_be(unsafe { std::ptr::read(received[6..=7].as_ptr() as *const u16) }),
            h: u16::from_be(unsafe { std::ptr::read(received[8..=9].as_ptr() as *const u16) }),
        };

        let payload = &received[10..];
        println!(
            "received from {:?}: {:?} (and {} bytes of payload)",
            source,
            header,
            payload.len()
        );

        match header.command {
            DisplayCommand::CmdClear => {
                println!("(imagine an empty screen now)")
            }
            DisplayCommand::CmdHardReset => {
                println!("display shutting down");
                return Ok(());
            }
            DisplayCommand::CmdBitmapLinearWin => {
                print_bitmap_linear_win(&header, payload);
            }
            DisplayCommand::CmdCp437data => {
                print_cp437_data(&header, payload);
            }
            _ => {
                println!(
                    "command {:?} sent by {:?} not implemented yet",
                    header.command, source
                );
            }
        }
    }
}

fn check_payload_size(buf: &[u8], expected: usize) -> bool {
    let actual = buf.len();
    if actual == expected {
        return true;
    }

    println!(
        "expected a payload length of {} but got {}",
        expected, actual
    );
    return false;
}

fn print_bitmap_linear_win(header: &HdrWindow, payload: &[u8]) {
    if !check_payload_size(payload, (header.w * header.h) as usize) {
        return;
    }

    println!("top left is offset by ({} | {})", header.x, header.y);
    for y in 0..header.h {
        for byte_x in 0..header.w {
            let byte_index = (y * header.w + byte_x) as usize;
            let byte = payload[byte_index];

            for bitmask in [1, 2, 4, 8, 16, 32, 64, 128] {
                let char = if byte & bitmask == bitmask {
                    '█'
                } else {
                    ' '
                };
                print!("{}", char);
            }
        }

        println!();
    }
}

fn print_cp437_data(header: &HdrWindow, payload: &[u8]) {
    if !check_payload_size(payload, (header.w * header.h) as usize) {
        return;
    }

    println!("top left is offset by ({} | {})", header.x, header.y);
    for y in 0..header.h {
        for byte_x in 0..header.w {
            let byte_index = (y * header.w + byte_x) as usize;
            print!("{}", payload[byte_index] as char)
        }

        println!();
    }
}
