#![deny(clippy::all)]

use std::default::Default;
use std::io::ErrorKind;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use clap::Parser;
use image::GenericImage;
use log::{debug, error, info, warn};
use num_derive::FromPrimitive;
use pixels::wgpu::TextureFormat;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::platform::x11::WindowAttributesExtX11;
use winit::window::{Window, WindowId};

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

const TILE_SIZE: u16 = 8;
const TILE_WIDTH: u16 = 65;
const TILE_HEIGHT: u16 = 20;
const TILE_COUNT: u16 = TILE_WIDTH * TILE_HEIGHT;
const PIXEL_WIDTH: u16 = TILE_WIDTH * TILE_SIZE;
const PIXEL_HEIGHT: u16 = TILE_HEIGHT * TILE_SIZE;
const PIXEL_COUNT: usize = PIXEL_WIDTH as usize * PIXEL_HEIGHT as usize;

static mut DISPLAY: [bool; PIXEL_COUNT] = [false; PIXEL_COUNT];

fn main() {
    assert_eq!(size_of::<HdrWindow>(), 10, "invalid struct size");

    env_logger::init();

    let cli = Cli::parse();
    info!("running with args: {:?}", &cli);

    info!("display booting up");

    let bind = cli.bind;
    let (tx, rx) = mpsc::channel();
    let thread = thread::spawn(move || {
        let socket = UdpSocket::bind(bind).expect("could not bind socket");
        socket
            .set_nonblocking(true)
            .expect("could not enter non blocking mode");

        let mut buf = [0; 8985];

        while rx.try_recv().is_err() {
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

    let event_loop = EventLoop::new().expect("could not create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("could not run event loop");

    tx.send(()).expect("could not cancel thread");
    thread.join().expect("could not join threads");
}

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

#[derive(Default)]
struct App {
    window: Option<Window>,
    pixels: Option<Pixels>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = Size::from(LogicalSize::new(PIXEL_WIDTH as f64, PIXEL_HEIGHT as f64));
        let attributes = Window::default_attributes()
            .with_title("pixel-receiver-rs")
            .with_inner_size(size);

        let window = event_loop.create_window(attributes).unwrap();
        self.window = Some(window);
        let window = self.window.as_ref().unwrap();

        self.pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Some(
                PixelsBuilder::new(PIXEL_WIDTH as u32, PIXEL_HEIGHT as u32, surface_texture)
                    .render_texture_format(TextureFormat::Bgra8UnormSrgb)
                    .build()
                    .expect("could not create pixels"),
            )
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        debug!("event {:?}", event);
        match event {
            WindowEvent::CloseRequested => {
                warn!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();
                let pixels = self.pixels.as_mut().unwrap();
                let frame = pixels.frame_mut().chunks_exact_mut(4);

                let mut i = 0;
                for pixel in frame {
                    unsafe {
                        if i >= DISPLAY.len() {
                            break;
                        }

                        let color = if DISPLAY[i] {
                            [255u8, 255, 255, 255]
                        } else {
                            [0u8, 0, 0, 255]
                        };
                        pixel.copy_from_slice(&color);
                    }
                    i += 1;
                }

                debug!("drawn {} pixels", i);

                pixels.render().expect("could not render");
                window.request_redraw();
            }
            _ => (),
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
