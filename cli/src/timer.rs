use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
    Arc,
};

pub struct Timer {
    timer_channel: Sender<usize>,
    exit_requested: Arc<AtomicBool>,
    period: f64,
}

impl Timer {
    pub fn new(timer_sender: Sender<usize>, exit_flag: Arc<AtomicBool>, period: f64) -> Self {
        Timer {
            timer_channel: timer_sender,
            exit_requested: exit_flag,
            period,
        }
    }

    pub fn run(&mut self) {
        let timer_duration = std::time::Duration::from_secs_f64(self.period);
        let mut timer = std::time::Instant::now() + timer_duration;
        while !self.exit_requested.load(Ordering::SeqCst) {
            let now = std::time::Instant::now();
            let mut ticks = 0;
            while now > timer {
                ticks += 1;
                timer += timer_duration;
            }

            if ticks != 0 {
                let _ = self.timer_channel.send(ticks);
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
