mod commands;

use crate::commands::Args;
use clap::Parser;
use error_iter::ErrorIter as _;
use interpreter::processor::*;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use std::fs;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let data: Vec<u8> = fs::read(args.path.clone()).map_err(|err| {
        format!(
            "Error reading input file at {}: {}",
            args.path.to_str().unwrap_or("<Invalid Path>"),
            err
        )
    })?;

    let mut proc = Processor::new(data).map_err(|err| {
        format!(
            "Error occurred loading program at path {}: {}",
            args.path.display(),
            err
        )
    })?;

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("WHIP-8")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)?
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    event_loop.run(|event, elwt| {
        // Draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            for pixel in pixels.frame_mut().chunks_exact_mut(4) {
                pixel.copy_from_slice(&[0x5e, 0x48, 0xe8, 0xff]);
            }
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                elwt.exit();
                return;
            }

            if proc.step().is_err() {
                elwt.exit();
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }

            // Update internal state and request a redraw
            // world.update();
            window.request_redraw();
        }
    })?;

    Ok(())
}
