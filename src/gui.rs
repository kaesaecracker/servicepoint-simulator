use std::{sync::mpsc::Sender, sync::RwLock};

use log::{info, warn};
use servicepoint::*;
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent,
    event_loop::ActiveEventLoop, keyboard::KeyCode::KeyC, window::WindowId,
};

use crate::cli::GuiOptions;
use crate::gui_window::GuiWindow;

pub struct Gui<'t> {
    display: &'t RwLock<Bitmap>,
    luma: &'t RwLock<BrightnessGrid>,
    stop_udp_tx: Sender<()>,
    options: GuiOptions,
    logical_size: LogicalSize<u16>,
    window: Option<GuiWindow>,
}

const SPACER_HEIGHT: usize = 4;
const NUM_SPACERS: usize = (PIXEL_HEIGHT / TILE_SIZE) - 1;
const PIXEL_HEIGHT_WITH_SPACERS: usize =
    PIXEL_HEIGHT + NUM_SPACERS * SPACER_HEIGHT;

const OFF_COLOR: u32 = u32::from_ne_bytes([0u8, 0, 0, 0]);

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
        options: GuiOptions,
    ) -> Self {
        Self {
            window: None,
            logical_size: Self::get_logical_size(options.spacers),
            display,
            luma,
            stop_udp_tx,
            options,
        }
    }

    fn draw(&mut self) {
        let display = self.display.read().unwrap();
        let luma = self.luma.read().unwrap();
        let brightness_scale =
            (u8::MAX as f32) / (u8::from(Brightness::MAX) as f32);

        let mut buffer = self.window.as_mut().unwrap().get_buffer();
        let mut frame = buffer.iter_mut();

        for tile_y in 0..TILE_HEIGHT {
            if self.options.spacers && tile_y != 0 {
                // cannot just frame.skip(PIXEL_WIDTH as usize * SPACER_HEIGHT as usize) because of typing
                for _ in 0..PIXEL_WIDTH * SPACER_HEIGHT {
                    frame.next().unwrap();
                }
            }

            let start_y = tile_y * TILE_SIZE;
            for y in start_y..start_y + TILE_SIZE {
                for tile_x in 0..TILE_WIDTH {
                    let brightness = u8::from(luma.get(tile_x, tile_y));
                    let brightness =
                        (brightness_scale * brightness as f32) as u8;
                    let on_color =
                        Self::get_on_color(&self.options, brightness);
                    let start_x = tile_x * TILE_SIZE;
                    for x in start_x..start_x + TILE_SIZE {
                        let color = if display.get(x, y) {
                            on_color
                        } else {
                            OFF_COLOR
                        };
                        *frame.next().unwrap() = color;
                    }
                }
            }
        }

        buffer.present().unwrap();
    }

    fn get_on_color(options: &GuiOptions, brightness: u8) -> u32 {
        u32::from_ne_bytes([
            if options.blue { brightness } else { 0u8 },
            if options.green { brightness } else { 0u8 },
            if options.red { brightness } else { 0u8 },
            0,
        ])
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

impl<'t> ApplicationHandler<AppEvents> for Gui<'t> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(GuiWindow::new(event_loop, self.logical_size));
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

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.window = None;
    }
}
