use strum_macros::Display;

use crate::types::{Address, GeneralRegister};

const NUM_GENERAL_REGISTERS: usize = 16;

#[derive(Debug, PartialEq, Eq, Display)]
pub enum Flag {
    Low,
    High,
}

pub struct Registers {
    pub i: Address,
    pub delay: u8,
    pub sound: u8,
    general: [u8; NUM_GENERAL_REGISTERS],
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            i: Address::from(0),
            delay: 0,
            sound: 0,
            general: [0; NUM_GENERAL_REGISTERS],
        }
    }
    pub fn get_general(&self, register: GeneralRegister) -> u8 {
        match register {
            GeneralRegister::V0 => self.general[0x0],
            GeneralRegister::V1 => self.general[0x1],
            GeneralRegister::V2 => self.general[0x2],
            GeneralRegister::V3 => self.general[0x3],
            GeneralRegister::V4 => self.general[0x4],
            GeneralRegister::V5 => self.general[0x5],
            GeneralRegister::V6 => self.general[0x6],
            GeneralRegister::V7 => self.general[0x7],
            GeneralRegister::V8 => self.general[0x8],
            GeneralRegister::V9 => self.general[0x9],
            GeneralRegister::VA => self.general[0xA],
            GeneralRegister::VB => self.general[0xB],
            GeneralRegister::VC => self.general[0xC],
            GeneralRegister::VD => self.general[0xD],
            GeneralRegister::VE => self.general[0xE],
            GeneralRegister::VF => self.general[0xF],
        }
    }

    pub fn set_general(&mut self, register: GeneralRegister, value: u8) {
        match register {
            GeneralRegister::V0 => self.general[0x0] = value,
            GeneralRegister::V1 => self.general[0x1] = value,
            GeneralRegister::V2 => self.general[0x2] = value,
            GeneralRegister::V3 => self.general[0x3] = value,
            GeneralRegister::V4 => self.general[0x4] = value,
            GeneralRegister::V5 => self.general[0x5] = value,
            GeneralRegister::V6 => self.general[0x6] = value,
            GeneralRegister::V7 => self.general[0x7] = value,
            GeneralRegister::V8 => self.general[0x8] = value,
            GeneralRegister::V9 => self.general[0x9] = value,
            GeneralRegister::VA => self.general[0xA] = value,
            GeneralRegister::VB => self.general[0xB] = value,
            GeneralRegister::VC => self.general[0xC] = value,
            GeneralRegister::VD => self.general[0xD] = value,
            GeneralRegister::VE => self.general[0xE] = value,
            GeneralRegister::VF => self.general[0xF] = value,
        }
    }

    #[allow(dead_code)] // TODO
    pub fn decrement_delay(&mut self) {
        if self.delay != 0 {
            self.delay -= 1;
        }
    }

    #[allow(dead_code)] // TODO
    pub fn decrement_sound(&mut self) {
        if self.sound != 0 {
            self.sound -= 1;
        }
    }

    pub fn set_vf_flag(&mut self, flag: Flag) {
        match flag {
            Flag::Low => self.set_general(GeneralRegister::VF, 0x00_u8),
            Flag::High => self.set_general(GeneralRegister::VF, 0x01_u8),
        }
    }

    #[allow(dead_code)] // TODO
    pub fn get_vf_flag(&self) -> Option<Flag> {
        match self.get_general(GeneralRegister::VF) {
            0x00_u8 => Some(Flag::Low),
            0x01_u8 => Some(Flag::High),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_i_zero_initialized() {
        let registers = Registers::new();
        assert_eq!(registers.i, Address::from(0));
    }

    #[test]
    fn test_delay_zero_initialized() {
        let registers = Registers::new();
        assert_eq!(registers.delay, 0_u8);
    }

    #[test]
    fn test_sound_zero_initialized() {
        let registers = Registers::new();
        assert_eq!(registers.sound, 0_u8);
    }

    #[test]
    fn test_general_zero_initialized() {
        let registers = Registers::new();
        for reg in GeneralRegister::iter() {
            assert_eq!(
                registers.get_general(reg),
                0_u8,
                "Register {} failed to be zero initialized.",
                reg
            );
        }
    }

    #[test]
    fn test_general_setters() {
        let mut registers = Registers::new();
        let test_value = 123_u8;
        for reg in GeneralRegister::iter() {
            assert_ne!(test_value, registers.get_general(reg));
            registers.set_general(reg, test_value);
        }

        for reg in GeneralRegister::iter() {
            assert_eq!(registers.get_general(reg), test_value);
        }
    }

    #[test]
    fn test_delay_decrement() {
        let mut registers = Registers::new();
        let test_valule = 123_u8;
        let existing_sound_value = registers.sound;
        registers.delay = test_valule;
        registers.decrement_delay();
        assert_eq!(test_valule - 1, registers.delay);
        assert_eq!(existing_sound_value, registers.sound);
    }

    #[test]
    fn test_sound_decrement() {
        let mut registers = Registers::new();
        let test_valule = 123_u8;
        let existing_delay_value = registers.delay;
        registers.sound = test_valule;
        registers.decrement_sound();
        assert_eq!(test_valule - 1, registers.sound);
        assert_eq!(existing_delay_value, registers.delay);
    }

    #[test]
    fn test_zero_delay_decrement() {
        let mut registers = Registers::new();
        registers.delay = 0_u8;
        registers.decrement_delay();
        assert_eq!(registers.delay, 0_u8);
    }

    #[test]
    fn test_zero_sound_decrement() {
        let mut registers = Registers::new();
        registers.sound = 0_u8;
        registers.decrement_sound();
        assert_eq!(registers.sound, 0_u8);
    }

    #[test]
    fn test_set_flag_high() {
        let mut registers = Registers::new();
        registers.set_vf_flag(Flag::High);
        assert_eq!(registers.get_vf_flag(), Some(Flag::High));
    }

    #[test]
    fn test_set_flag_low() {
        let mut registers = Registers::new();
        registers.set_vf_flag(Flag::Low);
        assert_eq!(registers.get_vf_flag(), Some(Flag::Low));
    }

    #[test]
    fn test_invalid_flag_is_none() {
        let mut registers = Registers::new();
        registers.set_general(GeneralRegister::VF, 123_u8);
        assert_eq!(registers.get_vf_flag(), None);
    }
}
