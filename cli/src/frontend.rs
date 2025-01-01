// this file modifies example code from the Pixels crate,
// specifically https://github.com/parasyte/pixels/tree/main/examples/minimal-winit
// See PIXELS_LICENSE.md for the license

use crate::utils::log_error;
use grid::Grid;
use interpreter::display::Pixel;
use pixels::{Pixels, SurfaceTexture};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Receiver,
    Arc,
};
use winit::keyboard::KeyCode;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

const INITIAL_DISPLAY_SCALING: usize = 10;

pub struct FrontendConfig {
    pub width: usize,
    pub height: usize,
    pub off_colour: [u8; 4],
    pub on_colour: [u8; 4],
}

pub struct Frontend {
    pixels: Pixels,
    event_loop: EventLoop<()>,
    input: WinitInputHelper,
    window: Window,
    exit_requested: Arc<AtomicBool>,
    frame_channel: Receiver<Grid<Pixel>>,
    image_buffer: Grid<Pixel>,
    off_colour: [u8; 4],
    on_colour: [u8; 4],
}

impl Frontend {
    pub fn new(
        config: FrontendConfig,
        exit_flag: Arc<AtomicBool>,
        frame_receiver: Receiver<Grid<Pixel>>,
    ) -> Result<Frontend, Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let input = WinitInputHelper::new();
        let window = {
            let size = LogicalSize::new(
                (INITIAL_DISPLAY_SCALING * config.width) as f64,
                (INITIAL_DISPLAY_SCALING * config.height) as f64,
            );
            WindowBuilder::new()
                .with_title("WHIP-8")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)?
        };
        let pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(config.width as u32, config.height as u32, surface_texture)?
        };

        Ok(Frontend {
            pixels,
            event_loop,
            input,
            window,
            exit_requested: exit_flag,
            frame_channel: frame_receiver,
            image_buffer: Grid::<Pixel>::init(config.height, config.width, Pixel::Off),
            off_colour: config.off_colour,
            on_colour: config.on_colour,
        })
    }

    pub fn run(mut self) -> Result<(), winit::error::EventLoopError> {
        self.event_loop.run(|event, elwt| {
            if self.exit_requested.load(Ordering::SeqCst) {
                elwt.exit();
                return;
            }

            if let Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } = event
            {
                if let Ok(recv_frame) = self.frame_channel.try_recv() {
                    self.image_buffer = recv_frame
                }

                for (dest, src) in self
                    .pixels
                    .frame_mut()
                    .chunks_exact_mut(4)
                    .zip(self.image_buffer.iter())
                {
                    dest.copy_from_slice(match src {
                        Pixel::Off => &self.off_colour,
                        Pixel::On => &self.on_colour,
                    });
                }

                if let Err(err) = self.pixels.render() {
                    log_error(err);
                    self.exit_requested.store(true, Ordering::SeqCst);
                    elwt.exit();
                    return;
                }
            }

            if self.input.update(&event)
                && (self.input.key_pressed(KeyCode::Escape) || self.input.close_requested())
            {
                elwt.exit();
                return;
            }

            if let Some(size) = self.input.window_resized() {
                if let Err(err) = self.pixels.resize_surface(size.width, size.height) {
                    log_error(err);
                    self.exit_requested.store(true, Ordering::SeqCst);
                    elwt.exit();
                    return;
                }
            }

            self.window.request_redraw();
        })
    }
}
