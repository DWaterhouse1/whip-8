use strum_macros::EnumIter;

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(u16);

impl From<u16> for Address {
    fn from(value: u16) -> Self {
        Address(value & 0x0FFF)
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter, Clone, Copy)]
pub enum Nibble {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
    Fifteen,
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
#[derive(Debug, Eq, PartialEq, EnumIter, Clone, Copy)]
pub enum Register {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
}

impl From<Nibble> for Register {
    fn from(value: Nibble) -> Self {
        match value {
            Nibble::Zero => Register::V0,
            Nibble::One => Register::V1,
            Nibble::Two => Register::V2,
            Nibble::Three => Register::V3,
            Nibble::Four => Register::V4,
            Nibble::Five => Register::V5,
            Nibble::Six => Register::V6,
            Nibble::Seven => Register::V7,
            Nibble::Eight => Register::V8,
            Nibble::Nine => Register::V9,
            Nibble::Ten => Register::VA,
            Nibble::Eleven => Register::VB,
            Nibble::Twelve => Register::VC,
            Nibble::Thirteen => Register::VD,
            Nibble::Fourteen => Register::VE,
            Nibble::Fifteen => Register::VF,
        }
    }
}
