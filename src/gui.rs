use std::sync::mpsc::Sender;
use std::sync::RwLock;

use log::{info, warn};
use pixels::wgpu::TextureFormat;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use servicepoint2::{
    ByteGrid, PixelGrid, PIXEL_HEIGHT, PIXEL_WIDTH, TILE_SIZE,
};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct App<'t> {
    display: &'t RwLock<PixelGrid>,
    luma: &'t RwLock<ByteGrid>,
    window: Option<Window>,
    pixels: Option<Pixels>,
    stop_udp_tx: Sender<()>,
}

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
    ) -> Self {
        App {
            display,
            luma,
            stop_udp_tx,
            pixels: None,
            window: None,
        }
    }
}

impl ApplicationHandler<AppEvents> for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = Size::from(LogicalSize::new(
            PIXEL_WIDTH as f64,
            PIXEL_HEIGHT as f64,
        ));
        let attributes = Window::default_attributes()
            .with_title("pixel-receiver-rs")
            .with_inner_size(size);

        let window = event_loop.create_window(attributes).unwrap();
        self.window = Some(window);
        let window = self.window.as_ref().unwrap();

        self.pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(
                window_size.width,
                window_size.height,
                &window,
            );
            Some(
                PixelsBuilder::new(
                    PIXEL_WIDTH as u32,
                    PIXEL_HEIGHT as u32,
                    surface_texture,
                )
                .render_texture_format(TextureFormat::Bgra8UnormSrgb)
                .build()
                .expect("could not create pixels"),
            )
        };
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
        if event == WindowEvent::CloseRequested {
            warn!("window event cloe requested");
            self.window = None;
            let _ = self.stop_udp_tx.send(()); // try to stop udp thread
            event_loop.exit();
        }

        if event != WindowEvent::RedrawRequested {
            return;
        }

        let pixels = self.pixels.as_mut().unwrap();

        let mut frame = pixels.frame_mut().chunks_exact_mut(4);

        let display = self.display.read().unwrap();
        let luma = self.luma.read().unwrap();

        for y in 0..PIXEL_HEIGHT as usize {
            for x in 0..PIXEL_WIDTH as usize {
                let is_set = display.get(x, y);
                let brightness =
                    luma.get(x / TILE_SIZE as usize, y / TILE_SIZE as usize);

                let color = if is_set {
                    [0u8, brightness, 0, 255]
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
