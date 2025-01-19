use std::slice::ChunksExactMut;
use std::sync::mpsc::Sender;
use std::sync::RwLock;

use log::{info, warn};
use pixels::{Pixels, SurfaceTexture};
use servicepoint::{
    Bitmap, Brightness, BrightnessGrid, Grid, PIXEL_HEIGHT, PIXEL_WIDTH,
    TILE_SIZE,
};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode::KeyC;
use winit::window::{Window, WindowId};

use crate::Cli;

pub struct App<'t> {
    display: &'t RwLock<Bitmap>,
    luma: &'t RwLock<BrightnessGrid>,
    window: Option<Window>,
    stop_udp_tx: Sender<()>,
    cli: &'t Cli,
    logical_size: LogicalSize<u16>,
}

const SPACER_HEIGHT: usize = 4;

#[derive(Debug)]
pub enum AppEvents {
    UdpPacketHandled,
    UdpThreadClosed,
}

impl<'t> App<'t> {
    pub fn new(
        display: &'t RwLock<Bitmap>,
        luma: &'t RwLock<BrightnessGrid>,
        stop_udp_tx: Sender<()>,
        cli: &'t Cli,
    ) -> Self {
        let logical_size = {
            let height = if cli.spacers {
                let num_spacers = (PIXEL_HEIGHT / TILE_SIZE) - 1;
                PIXEL_HEIGHT + num_spacers * SPACER_HEIGHT
            } else {
                PIXEL_HEIGHT
            };
            LogicalSize::new(PIXEL_WIDTH as u16, height as u16)
        };

        App {
            display,
            luma,
            stop_udp_tx,
            window: None,
            cli,
            logical_size,
        }
    }

    fn draw(&mut self) {
        let window = self.window.as_ref().unwrap();
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, &window);
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
                    u8::from(luma.get(x / TILE_SIZE, y / TILE_SIZE));
                let scale =
                    (u8::MAX as f32) / (u8::from(Brightness::MAX) as f32);
                let brightness = (scale * brightness as f32) as u8;
                let color = self.get_color(is_set, brightness);
                let pixel = frame.next().unwrap();
                pixel.copy_from_slice(&color);
            }
        }
    }

    fn get_color(&self, is_set: bool, brightness: u8) -> [u8; 4] {
        if is_set {
            [
                if self.cli.red { brightness } else { 0u8 },
                if self.cli.green { brightness } else { 0u8 },
                if self.cli.blue { brightness } else { 0u8 },
                255,
            ]
        } else {
            [0u8, 0, 0, 255]
        }
    }
}

impl ApplicationHandler<AppEvents> for App<'_> {
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
