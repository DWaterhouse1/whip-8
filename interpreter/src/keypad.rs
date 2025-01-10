pub(crate) const NUM_KEYS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    Pressed,
    Released,
}

pub(crate) struct Keys {
    keys_status: [KeyStatus; NUM_KEYS],
}

impl Keys {
    pub(crate) fn new() -> Keys {
        Keys {
            keys_status: [KeyStatus::Released; NUM_KEYS],
        }
    }

    pub(crate) fn input(&mut self, key: usize, status: KeyStatus) {
        if key > NUM_KEYS {
            return;
        }
        self.keys_status[key] = status;
    }

    pub(crate) fn get_status(&self, key: usize) -> Option<KeyStatus> {
        if key > NUM_KEYS {
            None
        } else {
            Some(self.keys_status[key])
        }
    }
}
