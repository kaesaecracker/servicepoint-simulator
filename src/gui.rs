use std::slice::ChunksExactMut;
use std::sync::mpsc::Sender;
use std::sync::RwLock;

use log::{info, warn};
use pixels::{Pixels, SurfaceTexture};
use servicepoint::*;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode::KeyC;
use winit::window::{Window, WindowId};

use crate::Cli;

pub struct Gui<'t> {
    display: &'t RwLock<Bitmap>,
    luma: &'t RwLock<BrightnessGrid>,
    window: Option<Window>,
    stop_udp_tx: Sender<()>,
    cli: &'t Cli,
    logical_size: LogicalSize<u16>,
}

const SPACER_HEIGHT: usize = 4;
const NUM_SPACERS: usize = (PIXEL_HEIGHT / TILE_SIZE) - 1;
const PIXEL_HEIGHT_WITH_SPACERS: usize =
    PIXEL_HEIGHT + NUM_SPACERS * SPACER_HEIGHT;

const OFF_COLOR: [u8; 4] = [0u8, 0, 0, 255];

#[derive(Debug)]
pub enum AppEvents {
    UdpPacketHandled,
    UdpThreadClosed,
}

impl<'t> Gui<'t> {
    pub fn new(
        display: &'t RwLock<Bitmap>,
        luma: &'t RwLock<BrightnessGrid>,
        stop_udp_tx: Sender<()>,
        cli: &'t Cli,
    ) -> Self {
        Gui {
            display,
            luma,
            stop_udp_tx,
            cli,
            window: None,
            logical_size: Self::get_logical_size(cli.spacers),
        }
    }

    fn draw(&mut self) {
        let window = self.window.as_ref().unwrap();
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, &window);

        // TODO: fix pixels: creating a new instance per draw crashes after some time on macOS,
        // but keeping one instance for the lifetime of the Gui SIGSEGVs on Wayland when entering a background state.
        let mut pixels = Pixels::new(
            self.logical_size.width as u32,
            self.logical_size.height as u32,
            surface_texture,
        )
        .unwrap();

        let mut frame = pixels.frame_mut().chunks_exact_mut(4);
        self.draw_frame(&mut frame);
        pixels.render().expect("could not render");
    }

    fn draw_frame(&self, frame: &mut ChunksExactMut<u8>) {
        let display = self.display.read().unwrap();
        let luma = self.luma.read().unwrap();
        let brightness_scale = (u8::MAX as f32) / (u8::from(Brightness::MAX) as f32);

        for tile_y in 0..TILE_HEIGHT {
            if self.cli.spacers && tile_y != 0 {
                // cannot just frame.skip(PIXEL_WIDTH as usize * SPACER_HEIGHT as usize) because of typing
                for _ in 0..PIXEL_WIDTH * SPACER_HEIGHT {
                    frame.next().unwrap();
                }
            }

            let start_y = tile_y * TILE_SIZE;
            for y in start_y..start_y + TILE_SIZE {
                for tile_x in 0..TILE_WIDTH {
                    let brightness = u8::from(luma.get(tile_x, tile_y));
                    let brightness = (brightness_scale * brightness as f32) as u8;
                    let on_color = self.get_on_color(brightness);
                    let start_x = tile_x * TILE_SIZE;
                    for x in start_x..start_x + TILE_SIZE {
                        let color = if display.get(x, y) { on_color } else { OFF_COLOR };
                        let pixel = frame.next().unwrap();
                        pixel.copy_from_slice(&color);
                    }
                }
            }
        }
    }

    fn get_on_color(&self, brightness: u8) -> [u8; 4] {
        [
            if self.cli.red { brightness } else { 0u8 },
            if self.cli.green { brightness } else { 0u8 },
            if self.cli.blue { brightness } else { 0u8 },
            255,
        ]
    }

    fn get_logical_size(spacers: bool) -> LogicalSize<u16> {
        let height = if spacers {
            PIXEL_HEIGHT_WITH_SPACERS
        } else {
            PIXEL_HEIGHT
        };
        LogicalSize::new(PIXEL_WIDTH as u16, height as u16)
    }
}

impl ApplicationHandler<AppEvents> for Gui<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title("servicepoint-simulator")
            .with_inner_size(self.logical_size)
            .with_transparent(false);

        let window = event_loop.create_window(attributes).unwrap();
        self.window = Some(window);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvents) {
        match event {
            AppEvents::UdpPacketHandled => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            AppEvents::UdpThreadClosed => {
                info!("stopping ui thread after udp thread stopped");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                warn!("window event close requested");
                self.window = None;
                let _ = self.stop_udp_tx.send(()); // try to stop udp thread
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.physical_key == KeyC && !event.repeat =>
            {
                self.display.write().unwrap().fill(false);
                self.luma.write().unwrap().fill(Brightness::MAX);
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}
