mod chip_8_interpreter;
mod commands;
mod frontend;
mod timer;
mod utils;

use crate::commands::Args;
use chip_8_interpreter::Chip8Interpreter;
use clap::Parser;
use frontend::{Frontend, FrontendConfig};
use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use timer::Timer;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const OFF_COLOUR: [u8; 4] = [0x10, 0x10, 0x10, 0xFF];
const ON_COLOUR: [u8; 4] = [0x5E, 0x48, 0xE8, 0xFF];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let program_data: Vec<u8> = fs::read(args.path.clone()).map_err(|err| {
        format!(
            "Error reading input file at {}: {}",
            args.path.display(),
            err
        )
    })?;

    // sync structures
    let (frame_tx, frame_rx) = std::sync::mpsc::channel();
    let (key_tx, key_rx) = std::sync::mpsc::channel();
    let (timer_tx, timer_rx) = std::sync::mpsc::channel();
    let exit_requested = Arc::new(AtomicBool::new(false));

    env_logger::init();

    let mut chip8 = Chip8Interpreter::new(
        program_data,
        exit_requested.clone(),
        frame_tx,
        key_rx,
        timer_rx,
    )?;

    let mut timer = Timer::new(timer_tx, exit_requested.clone(), 1.0 / 60.0);

    let frontend = Frontend::new(
        FrontendConfig {
            width: WIDTH as usize,
            height: HEIGHT as usize,
            off_colour: OFF_COLOUR,
            on_colour: ON_COLOUR,
        },
        exit_requested.clone(),
        frame_rx,
        key_tx,
    )?;

    let interpreter_thread = std::thread::spawn(move || {
        chip8.run();
    });

    let timer_thread = std::thread::spawn(move || {
        timer.run();
    });

    frontend.run()?;

    if exit_requested.load(std::sync::atomic::Ordering::SeqCst) {
        interpreter_thread
            .join()
            .expect("Unable to join interpreter thread.");
        timer_thread.join().expect("Unable to join timer thread.");
        return Err("Program exited unsuccessfully".into());
    }

    Ok(())
}
