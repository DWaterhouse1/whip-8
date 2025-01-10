use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, Sender},
    Arc,
};

use grid::Grid;
use interpreter::{
    display::Pixel,
    keypad::KeyStatus,
    processor::{Processor, ProcessorError},
};

use crate::utils::log_error;

pub struct KeyUpdate {
    pub key: usize,
    pub status: KeyStatus,
}

pub struct Chip8Interpreter {
    processor: Processor,
    exit_requested: Arc<AtomicBool>,
    frame_channel: Sender<Grid<Pixel>>,
    keys_channel: Receiver<KeyUpdate>,
}

impl Chip8Interpreter {
    pub fn new(
        program_data: Vec<u8>,
        exit_flag: Arc<AtomicBool>,
        frame_sender: Sender<Grid<Pixel>>,
        key_receiver: Receiver<KeyUpdate>,
    ) -> Result<Chip8Interpreter, ProcessorError> {
        Ok(Self {
            processor: Processor::new(program_data)?,
            exit_requested: exit_flag,
            frame_channel: frame_sender,
            keys_channel: key_receiver,
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

            while let Ok(key_event) = self.keys_channel.try_recv() {
                self.processor
                    .add_key_event(key_event.key, key_event.status);
            }
        }
    }

    fn encountered_error<E: std::error::Error + 'static>(&mut self, err: E) {
        log_error(err);
        self.exit_requested.store(true, Ordering::SeqCst);
    }
}
