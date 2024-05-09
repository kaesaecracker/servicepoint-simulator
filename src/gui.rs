use crate::{DISPLAY, PIXEL_HEIGHT, PIXEL_WIDTH};
use log::{debug, warn};
use pixels::wgpu::TextureFormat;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[derive(Default)]
pub struct App {
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
                    let is_set = unsafe { DISPLAY[i] };
                    let color = if is_set {
                        [255u8, 255, 255, 255]
                    } else {
                        [0u8, 0, 0, 255]
                    };

                    pixel.copy_from_slice(&color);
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
