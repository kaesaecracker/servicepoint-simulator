use log::{trace, warn};
use pixels::wgpu::TextureFormat;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use servicepoint2::{PixelGrid, PIXEL_HEIGHT, PIXEL_WIDTH};
use std::sync::mpsc::Receiver;
use std::sync::RwLock;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct App<'t> {
    display: &'t RwLock<PixelGrid>,
    window: Option<Window>,
    pixels: Option<Pixels>,
    stop_ui_rx: Receiver<()>,
}

impl<'t> App<'t> {
    pub fn new(display: &RwLock<PixelGrid>, stop_ui_rx: Receiver<()>) -> App {
        App {
            display,
            stop_ui_rx,
            pixels: None,
            window: None,
        }
    }
}

impl ApplicationHandler for App<'_> {
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
        trace!("event {:?}", event);
        match event {
            WindowEvent::CloseRequested => {
                warn!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if self.stop_ui_rx.try_recv().is_ok() {
                    warn!("ui thread stopping");
                    event_loop.exit();
                }

                let window = self.window.as_ref().unwrap();
                let pixels = self.pixels.as_mut().unwrap();
                let mut frame = pixels.frame_mut().chunks_exact_mut(4);

                let display = self.display.read().unwrap();

                let size = window.inner_size();
                for y in 0..size.height {
                    for x in 0..size.width {
                        let is_set = display.get(x as usize, y as usize);
                        let color = if is_set {
                            [255u8, 255, 255, 255]
                        } else {
                            [0u8, 0, 0, 255]
                        };

                        let pixel = frame.next().unwrap();
                        pixel.copy_from_slice(&color);
                    }
                }

                pixels.render().expect("could not render");
                window.request_redraw();
            }
            _ => (),
        }
    }
}
