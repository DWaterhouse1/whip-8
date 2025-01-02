use crate::types::{Address, GeneralRegister, Nibble};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Sys {
        addr: Address,
    },
    Clear,
    Return,
    Jump {
        addr: Address,
    },
    Call {
        addr: Address,
    },
    SkipIfEqByte {
        reg: GeneralRegister,
        value: u8,
    },
    SkipIfNeqByte {
        reg: GeneralRegister,
        value: u8,
    },
    SkipIfEqReg {
        lhs: GeneralRegister,
        rhs: GeneralRegister,
    },
    LoadValue {
        dest: GeneralRegister,
        value: u8,
    },
    AddValue {
        dest: GeneralRegister,
        value: u8,
    },
    LoadRegister {
        dest: GeneralRegister,
        source: GeneralRegister,
    },
    Or {
        dest: GeneralRegister,
        source: GeneralRegister,
    },
    And {
        dest: GeneralRegister,
        source: GeneralRegister,
    },
    Xor {
        dest: GeneralRegister,
        source: GeneralRegister,
    },
    AddRegister {
        dest: GeneralRegister,
        source: GeneralRegister,
    },
    Subtract {
        dest: GeneralRegister,
        source: GeneralRegister,
    },
    ShiftRight {
        dest: GeneralRegister,
        source: GeneralRegister,
    },
    SubtractNegate {
        dest: GeneralRegister,
        source: GeneralRegister,
    },
    ShiftLeft {
        dest: GeneralRegister,
        source: GeneralRegister,
    },
    SkipIfNeqReg {
        lhs: GeneralRegister,
        rhs: GeneralRegister,
    },
    LoadI {
        addr: Address,
    },
    JumpPlusV0 {
        addr: Address,
    },
    Random {
        dest: GeneralRegister,
        mask: u8,
    },
    Draw {
        x: GeneralRegister,
        y: GeneralRegister,
        num_bytes: Nibble,
    },
    SkipIfKeyDown {
        key_val: GeneralRegister,
    },
    SkipIfKeyUp {
        key_val: GeneralRegister,
    },
    LoadFromDelayTimer {
        dest: GeneralRegister,
    },
    LoadFromKey {
        dest: GeneralRegister,
    },
    SetDelayTimer {
        source: GeneralRegister,
    },
    SetSoundTimer {
        source: GeneralRegister,
    },
    AddI {
        source: GeneralRegister,
    },
    LoadSpriteLocation {
        digit: GeneralRegister,
    },
    LoadBcd {
        source: GeneralRegister,
    },
    StoreRegisterRangeAtI {
        last: GeneralRegister,
    },
    LoadRegisterRangeFromI {
        last: GeneralRegister,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstructionBytePair(pub u16);

impl Display for InstructionBytePair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#06x}", u16::to_be(self.0))
    }
}

impl InstructionBytePair {
    fn get_upper_byte(&self) -> u8 {
        ((self.0 & 0xFF00) >> 8) as u8
    }

    fn get_lower_byte(&self) -> u8 {
        (self.0 & 0x00FF) as u8
    }
}

fn handle_zero(bytes: InstructionBytePair) -> Option<Instruction> {
    match bytes.0 {
        0x00E0 => Some(Instruction::Clear),
        0x00EE => Some(Instruction::Return),
        value => Some(Instruction::Sys {
            addr: Address::from(value),
        }),
    }
}

fn handle_one(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::Jump {
        addr: Address::from(bytes.0 & 0x0FFF),
    })
}

fn handle_two(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::Call {
        addr: Address::from(bytes.0 & 0x0FFF),
    })
}

fn handle_three(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::SkipIfEqByte {
        reg: Nibble::from_lower(bytes.get_upper_byte()).into(),
        value: bytes.get_lower_byte(),
    })
}

fn handle_four(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::SkipIfNeqByte {
        reg: Nibble::from_lower(bytes.get_upper_byte()).into(),
        value: bytes.get_lower_byte(),
    })
}

fn handle_five(bytes: InstructionBytePair) -> Option<Instruction> {
    if Nibble::from_lower(bytes.get_lower_byte()) != Nibble::Zero {
        return None;
    }

    Some(Instruction::SkipIfEqReg {
        lhs: Nibble::from_lower(bytes.get_upper_byte()).into(),
        rhs: Nibble::from_upper(bytes.get_lower_byte()).into(),
    })
}

fn handle_six(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::LoadValue {
        dest: Nibble::from_lower(bytes.get_upper_byte()).into(),
        value: bytes.get_lower_byte(),
    })
}

fn handle_seven(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::AddValue {
        dest: Nibble::from_lower(bytes.get_upper_byte()).into(),
        value: bytes.get_lower_byte(),
    })
}

fn handle_eight(bytes: InstructionBytePair) -> Option<Instruction> {
    let x: GeneralRegister = Nibble::from_lower(bytes.get_upper_byte()).into();
    let y: GeneralRegister = Nibble::from_upper(bytes.get_lower_byte()).into();
    match Nibble::from_lower(bytes.get_lower_byte()) {
        Nibble::Zero => Some(Instruction::LoadRegister { dest: x, source: y }),
        Nibble::One => Some(Instruction::Or { dest: x, source: y }),
        Nibble::Two => Some(Instruction::And { dest: x, source: y }),
        Nibble::Three => Some(Instruction::Xor { dest: x, source: y }),
        Nibble::Four => Some(Instruction::AddRegister { dest: x, source: y }),
        Nibble::Five => Some(Instruction::Subtract { dest: x, source: y }),
        Nibble::Six => Some(Instruction::ShiftRight { dest: x, source: y }),
        Nibble::Seven => Some(Instruction::SubtractNegate { dest: x, source: y }),
        Nibble::Fourteen => Some(Instruction::ShiftLeft { dest: x, source: y }),
        _ => None,
    }
}

fn handle_nine(bytes: InstructionBytePair) -> Option<Instruction> {
    if Nibble::from_lower(bytes.get_lower_byte()) != Nibble::Zero {
        return None;
    }

    Some(Instruction::SkipIfNeqReg {
        lhs: Nibble::from_lower(bytes.get_upper_byte()).into(),
        rhs: Nibble::from_upper(bytes.get_lower_byte()).into(),
    })
}

fn handle_ten(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::LoadI {
        addr: Address::from(bytes.0 & 0x0FFF),
    })
}

fn handle_eleven(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::JumpPlusV0 {
        addr: Address::from(bytes.0 & 0x0FFF),
    })
}

fn handle_twelve(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::Random {
        dest: Nibble::from_lower(bytes.get_upper_byte()).into(),
        mask: bytes.get_lower_byte(),
    })
}

fn handle_thirteen(bytes: InstructionBytePair) -> Option<Instruction> {
    Some(Instruction::Draw {
        x: Nibble::from_lower(bytes.get_upper_byte()).into(),
        y: Nibble::from_upper(bytes.get_lower_byte()).into(),
        num_bytes: Nibble::from_lower(bytes.get_lower_byte()),
    })
}

fn handle_fourteen(bytes: InstructionBytePair) -> Option<Instruction> {
    let key_val: GeneralRegister = Nibble::from_lower(bytes.get_upper_byte()).into();
    match bytes.get_lower_byte() {
        0x9E => Some(Instruction::SkipIfKeyDown { key_val }),
        0xA1 => Some(Instruction::SkipIfKeyUp { key_val }),
        _ => None,
    }
}

fn handle_fifteen(bytes: InstructionBytePair) -> Option<Instruction> {
    let x: GeneralRegister = Nibble::from_lower(bytes.get_upper_byte()).into();
    match bytes.get_lower_byte() {
        0x07 => Some(Instruction::LoadFromDelayTimer { dest: x }),
        0x0A => Some(Instruction::LoadFromKey { dest: x }),
        0x15 => Some(Instruction::SetDelayTimer { source: x }),
        0x18 => Some(Instruction::SetSoundTimer { source: x }),
        0x1E => Some(Instruction::AddI { source: x }),
        0x29 => Some(Instruction::LoadSpriteLocation { digit: x }),
        0x33 => Some(Instruction::LoadBcd { source: x }),
        0x55 => Some(Instruction::StoreRegisterRangeAtI { last: x }),
        0x65 => Some(Instruction::LoadRegisterRangeFromI { last: x }),
        _ => None,
    }
}

pub fn decode(bytes: InstructionBytePair) -> Option<Instruction> {
    match Nibble::from_upper(bytes.get_upper_byte()) {
        Nibble::Zero => handle_zero(bytes),
        Nibble::One => handle_one(bytes),
        Nibble::Two => handle_two(bytes),
        Nibble::Three => handle_three(bytes),
        Nibble::Four => handle_four(bytes),
        Nibble::Five => handle_five(bytes),
        Nibble::Six => handle_six(bytes),
        Nibble::Seven => handle_seven(bytes),
        Nibble::Eight => handle_eight(bytes),
        Nibble::Nine => handle_nine(bytes),
        Nibble::Ten => handle_ten(bytes),
        Nibble::Eleven => handle_eleven(bytes),
        Nibble::Twelve => handle_twelve(bytes),
        Nibble::Thirteen => handle_thirteen(bytes),
        Nibble::Fourteen => handle_fourteen(bytes),
        Nibble::Fifteen => handle_fifteen(bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    fn all_addresses() -> impl Iterator<Item = u16> {
        0x0000..0x1000
    }

    fn all_bytes() -> impl Iterator<Item = u8> {
        0x00..=0xFF
    }

    #[test]
    fn test_cls() {
        let clear_bytes = InstructionBytePair(0x00E0);
        let decoded = decode(clear_bytes).unwrap();
        assert_eq!(decoded, Instruction::Clear);
    }

    #[test]
    fn test_ret() {
        let clear_bytes = InstructionBytePair(0x00EE);
        let decoded = decode(clear_bytes).unwrap();
        assert_eq!(decoded, Instruction::Return);
    }

    #[test]
    fn test_sys() {
        let non_sys_addresses = [0x00E0, 0x00EE];
        for value in all_addresses().filter(|x| !non_sys_addresses.contains(x)) {
            let sys_bytes = InstructionBytePair(value);
            let decoded = decode(sys_bytes).unwrap();
            assert_eq!(decoded, Instruction::Sys { addr: value.into() });
        }

        for value in non_sys_addresses {
            let non_sys_bytes = InstructionBytePair(value);
            let decoded = decode(non_sys_bytes).unwrap();
            assert!(!matches!(decoded, Instruction::Sys { addr: _ }));
        }
    }

    #[test]
    fn test_jp() {
        for value in all_addresses() {
            let jump_bytes = InstructionBytePair(0x1000 | value);
            let decoded = decode(jump_bytes).unwrap();
            assert_eq!(decoded, Instruction::Jump { addr: value.into() });
        }
    }

    #[test]
    fn test_call() {
        for value in all_addresses() {
            let jump_bytes = InstructionBytePair(0x2000 | value);
            let decoded = decode(jump_bytes).unwrap();
            assert_eq!(decoded, Instruction::Call { addr: value.into() });
        }
    }

    #[test]
    fn test_se_vx_byte() {
        for reg in GeneralRegister::iter() {
            for value in all_bytes() {
                let skip_eq_bytes =
                    InstructionBytePair(0x3000 | ((reg as u16) << 8) | value as u16);
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::SkipIfEqByte { reg, value });
            }
        }
    }

    #[test]
    fn test_sne_vx_byte() {
        for reg in GeneralRegister::iter() {
            for value in all_bytes() {
                let skip_eq_bytes =
                    InstructionBytePair(0x4000 | ((reg as u16) << 8) | value as u16);
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::SkipIfNeqByte { reg, value });
            }
        }
    }

    #[test]
    fn test_se_vx_vy() {
        for lhs in GeneralRegister::iter() {
            for rhs in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x5000 | ((lhs as u16) << 8) | ((rhs as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::SkipIfEqReg { lhs, rhs });
            }
        }
    }

    #[test]
    fn test_invalid_fives() {
        for bytes in (0x0000..0x1000).filter(|x| (x % 0x0010) != 0) {
            let invalid_bytes = InstructionBytePair(0x5000 | bytes);
            let decoded = decode(invalid_bytes);
            assert!(decoded.is_none());
        }
    }

    #[test]
    fn test_ld_vx_byte() {
        for dest in GeneralRegister::iter() {
            for value in all_bytes() {
                let skip_eq_bytes =
                    InstructionBytePair(0x6000 | ((dest as u16) << 8) | value as u16);
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::LoadValue { dest, value });
            }
        }
    }

    #[test]
    fn test_add_vx_byte() {
        for dest in GeneralRegister::iter() {
            for value in all_bytes() {
                let skip_eq_bytes =
                    InstructionBytePair(0x7000 | ((dest as u16) << 8) | value as u16);
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::AddValue { dest, value });
            }
        }
    }

    #[test]
    fn test_ld_vx_vy() {
        for dest in GeneralRegister::iter() {
            for source in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x8000 | ((dest as u16) << 8) | ((source as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::LoadRegister { dest, source });
            }
        }
    }

    #[test]
    fn test_or_vx_vy() {
        for dest in GeneralRegister::iter() {
            for source in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x8001 | ((dest as u16) << 8) | ((source as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::Or { dest, source });
            }
        }
    }

    #[test]
    fn test_and_vx_vy() {
        for dest in GeneralRegister::iter() {
            for source in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x8002 | ((dest as u16) << 8) | ((source as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::And { dest, source });
            }
        }
    }

    #[test]
    fn test_xor_vx_vy() {
        for dest in GeneralRegister::iter() {
            for source in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x8003 | ((dest as u16) << 8) | ((source as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::Xor { dest, source });
            }
        }
    }

    #[test]
    fn test_add_vx_vy() {
        for dest in GeneralRegister::iter() {
            for source in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x8004 | ((dest as u16) << 8) | ((source as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::AddRegister { dest, source });
            }
        }
    }

    #[test]
    fn test_sub_vx_vy() {
        for dest in GeneralRegister::iter() {
            for source in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x8005 | ((dest as u16) << 8) | ((source as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::Subtract { dest, source });
            }
        }
    }

    #[test]
    fn test_shr_vx_vy() {
        for dest in GeneralRegister::iter() {
            for source in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x8006 | ((dest as u16) << 8) | ((source as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::ShiftRight { dest, source });
            }
        }
    }

    #[test]
    fn test_subn_vx_vy() {
        for dest in GeneralRegister::iter() {
            for source in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x8007 | ((dest as u16) << 8) | ((source as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::SubtractNegate { dest, source });
            }
        }
    }

    #[test]
    fn test_shl_vx_vy() {
        for dest in GeneralRegister::iter() {
            for source in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x800E | ((dest as u16) << 8) | ((source as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::ShiftLeft { dest, source });
            }
        }
    }

    #[test]
    fn test_invalid_eights() {
        for bytes in (0x0000..0x1000).filter(|x| (x % 0x0010) > 0x7 && (x % 0x0010) != 0xE) {
            let invalid_bytes = InstructionBytePair(0x8000 | bytes);
            let decoded = decode(invalid_bytes);
            assert!(decoded.is_none());
        }
    }

    #[test]
    fn test_sne_vx_vy() {
        for lhs in GeneralRegister::iter() {
            for rhs in GeneralRegister::iter() {
                let skip_eq_bytes =
                    InstructionBytePair(0x9000 | ((lhs as u16) << 8) | ((rhs as u16) << 4));
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::SkipIfNeqReg { lhs, rhs });
            }
        }
    }

    #[test]
    fn test_invalid_nines() {
        for bytes in (0x0000..0x1000).filter(|x| (x % 0x0010) != 0) {
            let invalid_bytes = InstructionBytePair(0x9000 | bytes);
            let decoded = decode(invalid_bytes);
            assert!(decoded.is_none());
        }
    }

    #[test]
    fn test_ld_i() {
        for value in all_addresses() {
            let jump_bytes = InstructionBytePair(0xA000 | value);
            let decoded = decode(jump_bytes).unwrap();
            assert_eq!(decoded, Instruction::LoadI { addr: value.into() });
        }
    }

    #[test]
    fn test_jp_v0() {
        for value in all_addresses() {
            let jump_bytes = InstructionBytePair(0xB000 | value);
            let decoded = decode(jump_bytes).unwrap();
            assert_eq!(decoded, Instruction::JumpPlusV0 { addr: value.into() });
        }
    }

    #[test]
    fn test_rnd_vx() {
        for dest in GeneralRegister::iter() {
            for mask in all_bytes() {
                let skip_eq_bytes =
                    InstructionBytePair(0xC000 | ((dest as u16) << 8) | mask as u16);
                let decoded = decode(skip_eq_bytes).unwrap();
                assert_eq!(decoded, Instruction::Random { dest, mask });
            }
        }
    }

    #[test]
    fn test_drw_vx_vy() {
        for x in GeneralRegister::iter() {
            for y in GeneralRegister::iter() {
                for num_bytes in Nibble::iter() {
                    let draw_bytes = InstructionBytePair(
                        0xD000 | ((x as u16) << 8) | ((y as u16) << 4) | num_bytes as u16,
                    );
                    let decoded = decode(draw_bytes).unwrap();
                    assert_eq!(decoded, Instruction::Draw { x, y, num_bytes });
                }
            }
        }
    }

    #[test]
    fn test_skp_vx() {
        for key_val in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xE09E | ((key_val as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::SkipIfKeyDown { key_val });
        }
    }

    #[test]
    fn test_sknp_vx() {
        for key_val in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xE0A1 | ((key_val as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::SkipIfKeyUp { key_val });
        }
    }

    #[test]
    fn test_ld_vx_dt() {
        for dest in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xF007 | ((dest as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::LoadFromDelayTimer { dest });
        }
    }

    #[test]
    fn test_ld_vx_k() {
        for dest in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xF00A | ((dest as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::LoadFromKey { dest });
        }
    }

    #[test]
    fn test_ld_dt_vx() {
        for source in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xF015 | ((source as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::SetDelayTimer { source });
        }
    }

    #[test]
    fn test_ld_st_vx() {
        for source in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xF018 | ((source as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::SetSoundTimer { source });
        }
    }

    #[test]
    fn test_add_i_vx() {
        for source in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xF01E | ((source as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::AddI { source });
        }
    }

    #[test]
    fn test_ld_f_vx() {
        for digit in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xF029 | ((digit as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::LoadSpriteLocation { digit });
        }
    }

    #[test]
    fn test_ld_b_vx() {
        for source in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xF033 | ((source as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::LoadBcd { source });
        }
    }

    #[test]
    fn test_ld_iarray_vx() {
        for last in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xF055 | ((last as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::StoreRegisterRangeAtI { last });
        }
    }

    #[test]
    fn test_ld_vx_iarray() {
        for last in GeneralRegister::iter() {
            let skip_key_bytes = InstructionBytePair(0xF065 | ((last as u16) << 8));
            let decoded = decode(skip_key_bytes).unwrap();
            assert_eq!(decoded, Instruction::LoadRegisterRangeFromI { last });
        }
    }

    #[test]
    fn test_invalid_fifteens() {
        let valid_tails = [0x07, 0x0A, 0x15, 0x18, 0x1E, 0x29, 0x33, 0x55, 0x65];
        for x in GeneralRegister::iter() {
            for invalid_tail in (0x00..=0xFF).filter(|x| !valid_tails.contains(x)) {
                let invalid_bytes = InstructionBytePair(0xF000 | ((x as u16) << 8) | invalid_tail);
                let decoded = decode(invalid_bytes);
                assert!(decoded.is_none());
            }
        }
    }
}
