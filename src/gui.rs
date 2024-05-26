use std::sync::mpsc::Sender;
use std::sync::RwLock;

use log::{info, warn};
use pixels::wgpu::TextureFormat;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use servicepoint::{ByteGrid, PixelGrid, PIXEL_HEIGHT, PIXEL_WIDTH, TILE_SIZE, Grid};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode::KeyC;
use winit::window::{Window, WindowId};

use crate::Cli;

pub struct App<'t> {
    display: &'t RwLock<PixelGrid>,
    luma: &'t RwLock<ByteGrid>,
    window: Option<Window>,
    pixels: Option<Pixels>,
    stop_udp_tx: Sender<()>,
    cli: &'t Cli,
}

const SPACER_HEIGHT: usize = 4;

#[derive(Debug)]
pub enum AppEvents {
    UdpPacketHandled,
    UdpThreadClosed,
}

impl<'t> App<'t> {
    pub fn new(
        display: &'t RwLock<PixelGrid>,
        luma: &'t RwLock<ByteGrid>,
        stop_udp_tx: Sender<()>,
        cli: &'t Cli,
    ) -> Self {
        App {
            display,
            luma,
            stop_udp_tx,
            pixels: None,
            window: None,
            cli,
        }
    }

    fn draw(&mut self) {
        let pixels = self.pixels.as_mut().unwrap();

        let mut frame = pixels.frame_mut().chunks_exact_mut(4);
        let display = self.display.read().unwrap();
        let luma = self.luma.read().unwrap();
        for y in 0..PIXEL_HEIGHT {
            if self.cli.spacers && y != 0 && y % TILE_SIZE == 0 {
                // cannot just frame.skip(PIXEL_WIDTH as usize * SPACER_HEIGHT as usize) because of typing
                for _ in 0..PIXEL_WIDTH * SPACER_HEIGHT {
                    frame.next().unwrap();
                }
            }

            for x in 0..PIXEL_WIDTH {
                let is_set = display.get(x, y);
                let brightness =
                    luma.get(x / TILE_SIZE, y / TILE_SIZE);

                let color = if is_set {
                    [
                        if self.cli.red { brightness } else { 0u8 },
                        if self.cli.green { brightness } else { 0u8 },
                        if self.cli.blue { brightness } else { 0u8 },
                        255,
                    ]
                } else {
                    [0u8, 0, 0, 255]
                };

                let pixel = frame.next().unwrap();
                pixel.copy_from_slice(&color);
            }
        }

        pixels.render().expect("could not render");
    }
}

impl ApplicationHandler<AppEvents> for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let height = if self.cli.spacers {
            let num_spacers = (PIXEL_HEIGHT / TILE_SIZE) - 1;
            PIXEL_HEIGHT + num_spacers * SPACER_HEIGHT
        } else {
            PIXEL_HEIGHT
        };

        let size = LogicalSize::new(PIXEL_WIDTH as u16, height as u16);
        let attributes = Window::default_attributes()
            .with_title("servicepoint-simulator")
            .with_inner_size(size)
            .with_transparent(false);

        let window = event_loop.create_window(attributes).unwrap();
        self.window = Some(window);
        let window = self.window.as_ref().unwrap();

        let window_size = window.inner_size();
        let pixels = PixelsBuilder::new(
            size.width as u32,
            size.height as u32,
            SurfaceTexture::new(window_size.width, window_size.height, &window),
        )
        .render_texture_format(TextureFormat::Bgra8UnormSrgb)
        .build()
        .expect("could not create pixels");

        self.pixels = Some(pixels);
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
                self.luma.write().unwrap().fill(u8::MAX);
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}
