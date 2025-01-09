use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
    Arc,
};

use grid::Grid;
use interpreter::{
    display::Pixel,
    processor::{Processor, ProcessorError},
};

use crate::utils::log_error;

pub struct Chip8Interpreter {
    processor: Processor,
    exit_requested: Arc<AtomicBool>,
    frame_channel: Sender<Grid<Pixel>>,
}

impl Chip8Interpreter {
    pub fn new(
        program_data: Vec<u8>,
        exit_flag: Arc<AtomicBool>,
        frame_sender: Sender<Grid<Pixel>>,
    ) -> Result<Chip8Interpreter, ProcessorError> {
        Ok(Self {
            processor: Processor::new(program_data)?,
            exit_requested: exit_flag,
            frame_channel: frame_sender,
        })
    }

    pub fn run(&mut self) {
        while !self.exit_requested.load(Ordering::SeqCst) {
            if let Err(err) = self.processor.step() {
                self.encountered_error(err);
                return;
            }

            if let Some(fresh_frame) = self.processor.get_display_buffer() {
                if let Err(err) = self.frame_channel.send(fresh_frame.clone()) {
                    self.encountered_error(err);
                    return;
                }
            }
        }
    }

    fn encountered_error<E: std::error::Error + 'static>(&mut self, err: E) {
        log_error(err);
        self.exit_requested.store(true, Ordering::SeqCst);
    }
}
