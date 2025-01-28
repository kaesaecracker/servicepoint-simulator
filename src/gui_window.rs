use softbuffer::Buffer;
use std::{num::NonZero, rc::Rc};
use winit::{dpi::LogicalSize, event_loop::ActiveEventLoop, window::Window};

type Context = softbuffer::Context<Rc<Window>>;
type Surface = softbuffer::Surface<Rc<Window>, Rc<Window>>;

pub struct GuiWindow {
    winit_window: Rc<Window>,
    surface: Surface,
}

impl GuiWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        logical_size: LogicalSize<u16>,
    ) -> GuiWindow {
        let attributes = Window::default_attributes()
            .with_title("servicepoint-simulator")
            .with_min_inner_size(logical_size)
            .with_inner_size(logical_size)
            .with_transparent(false);
        let winit_window =
            Rc::new(event_loop.create_window(attributes).unwrap());
        let context = Context::new(winit_window.clone()).unwrap();
        let mut surface = Surface::new(&context, winit_window.clone()).unwrap();
        surface
            .resize(
                NonZero::new(logical_size.width as u32).unwrap(),
                NonZero::new(logical_size.height as u32).unwrap(),
            )
            .unwrap();

        Self {
            winit_window,
            surface,
        }
    }

    pub fn get_buffer(&mut self) -> Buffer<Rc<Window>, Rc<Window>> {
        self.surface.buffer_mut().unwrap()
    }
    pub(crate) fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }
}
