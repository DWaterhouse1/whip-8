use strum_macros::{Display, EnumIter};

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Address(u16);

impl Address {
    pub fn increment(&mut self, value: usize) {
        *self = Address(self.0 + value as u16);
    }
}

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Address(value & 0x0FFF)
    }
}

impl From<Address> for u16 {
    fn from(value: Address) -> Self {
        value.0
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter, Clone, Copy)]
pub enum Nibble {
    Zero = 0x0_u8,
    One = 0x1_u8,
    Two = 0x2_u8,
    Three = 0x3_u8,
    Four = 0x4_u8,
    Five = 0x5_u8,
    Six = 0x6_u8,
    Seven = 0x7_u8,
    Eight = 0x8_u8,
    Nine = 0x9_u8,
    Ten = 0xA_u8,
    Eleven = 0xB_u8,
    Twelve = 0xC_u8,
    Thirteen = 0xD_u8,
    Fourteen = 0xE_u8,
    Fifteen = 0xF_u8,
}

impl Nibble {
    pub fn from_upper(byte: u8) -> Nibble {
        match (byte & 0xF0_u8) >> 4 {
            0x00 => Nibble::Zero,
            0x01 => Nibble::One,
            0x02 => Nibble::Two,
            0x03 => Nibble::Three,
            0x04 => Nibble::Four,
            0x05 => Nibble::Five,
            0x06 => Nibble::Six,
            0x07 => Nibble::Seven,
            0x08 => Nibble::Eight,
            0x09 => Nibble::Nine,
            0x0A => Nibble::Ten,
            0x0B => Nibble::Eleven,
            0x0C => Nibble::Twelve,
            0x0D => Nibble::Thirteen,
            0x0E => Nibble::Fourteen,
            0x0F => Nibble::Fifteen,
            _ => unreachable!(),
        }
    }

    pub fn from_lower(byte: u8) -> Nibble {
        match byte & 0x0F_u8 {
            0x00 => Nibble::Zero,
            0x01 => Nibble::One,
            0x02 => Nibble::Two,
            0x03 => Nibble::Three,
            0x04 => Nibble::Four,
            0x05 => Nibble::Five,
            0x06 => Nibble::Six,
            0x07 => Nibble::Seven,
            0x08 => Nibble::Eight,
            0x09 => Nibble::Nine,
            0x0A => Nibble::Ten,
            0x0B => Nibble::Eleven,
            0x0C => Nibble::Twelve,
            0x0D => Nibble::Thirteen,
            0x0E => Nibble::Fourteen,
            0x0F => Nibble::Fifteen,
            _ => unreachable!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, EnumIter, Clone, Copy, Display)]
pub enum GeneralRegister {
    V0 = 0x0_u8,
    V1 = 0x1_u8,
    V2 = 0x2_u8,
    V3 = 0x3_u8,
    V4 = 0x4_u8,
    V5 = 0x5_u8,
    V6 = 0x6_u8,
    V7 = 0x7_u8,
    V8 = 0x8_u8,
    V9 = 0x9_u8,
    VA = 0xA_u8,
    VB = 0xB_u8,
    VC = 0xC_u8,
    VD = 0xD_u8,
    VE = 0xE_u8,
    VF = 0xF_u8,
}

impl From<Nibble> for GeneralRegister {
    fn from(value: Nibble) -> Self {
        match value {
            Nibble::Zero => GeneralRegister::V0,
            Nibble::One => GeneralRegister::V1,
            Nibble::Two => GeneralRegister::V2,
            Nibble::Three => GeneralRegister::V3,
            Nibble::Four => GeneralRegister::V4,
            Nibble::Five => GeneralRegister::V5,
            Nibble::Six => GeneralRegister::V6,
            Nibble::Seven => GeneralRegister::V7,
            Nibble::Eight => GeneralRegister::V8,
            Nibble::Nine => GeneralRegister::V9,
            Nibble::Ten => GeneralRegister::VA,
            Nibble::Eleven => GeneralRegister::VB,
            Nibble::Twelve => GeneralRegister::VC,
            Nibble::Thirteen => GeneralRegister::VD,
            Nibble::Fourteen => GeneralRegister::VE,
            Nibble::Fifteen => GeneralRegister::VF,
        }
    }
}
