use core::fmt;
use grid::Grid;
use strum::IntoEnumIterator;

use crate::display::{Display, Pixel};
use crate::instructions::{self, Instruction};
use crate::registers::{Flag, Registers};
use crate::types::{Address, GeneralRegister};

const MEMORY_SIZE_BYTES: usize = 0xFFF;
const STACK_SIZE: usize = 16;
const PROGRAM_START: usize = 0x200;
const MAX_PROGRAM_BYTES: usize = MEMORY_SIZE_BYTES - PROGRAM_START;
const HEX_SPRITE_STRIDE: usize = 5;
const HEX_SPRITE_DATA: [u8; HEX_SPRITE_STRIDE * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorError {
    ProgramTooLong {
        size: usize,
    },
    StackOverflow {
        address: Address,
    },
    StackUnderflow {
        address: Address,
    },
    MemoryOverrun {
        address: Address,
    },
    DecodeFailure {
        instruction: instructions::InstructionBytePair,
    },
}

impl fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            ProcessorError::ProgramTooLong { size } => format!(
                "Can't load program of size {}, max capacity is {}",
                size, MAX_PROGRAM_BYTES
            ),
            ProcessorError::StackOverflow { address } => format!(
                "Stack overflow occurred while executing instruction at address: {}",
                address
            ),
            ProcessorError::StackUnderflow { address } => format!(
                "Stack underflow occurred while executing instruction at address: {}",
                address
            ),
            ProcessorError::MemoryOverrun { address } => format!(
                "Memory overrun occurred while executing instruction at address: {}",
                address
            ),
            ProcessorError::DecodeFailure { instruction } => {
                format!("Failed to decode instruction: {}", instruction)
            }
        };
        write!(f, "{}", err_msg)
    }
}

impl std::error::Error for ProcessorError {}

pub struct Config {
    display_width: usize,
    display_height: usize,
}

const DEFAULT_CONFIG: Config = Config {
    display_width: 64,
    display_height: 32,
};

pub struct Processor {
    memory: [u8; MEMORY_SIZE_BYTES],
    registers: Registers,
    stack: [Address; STACK_SIZE],
    program_counter: Address,
    stack_pointer: usize,
    display: Display,
}

fn to_bcd(byte: u8) -> [u8; 3] {
    let mut scratch = 0_u32;
    scratch |= byte as u32;
    for _ in 0..8 {
        for nibble_idx in 2..5 {
            if (scratch >> (4 * nibble_idx)) & 0xF_u32 >= 5 {
                scratch += 3_u32 << (4 * nibble_idx);
            }
        }
        scratch <<= 1;
    }

    [
        ((scratch >> 16) & 0xF_u32) as u8,
        ((scratch >> 12) & 0xF_u32) as u8,
        ((scratch >> 8) & 0xF_u32) as u8,
    ]
}

impl Processor {
    pub fn new(program_bytes: Vec<u8>) -> Result<Self, ProcessorError> {
        Self::new_with_config(program_bytes, DEFAULT_CONFIG)
    }
    pub fn new_with_config(program_bytes: Vec<u8>, config: Config) -> Result<Self, ProcessorError> {
        if program_bytes.len() > MAX_PROGRAM_BYTES {
            return Err(ProcessorError::ProgramTooLong {
                size: program_bytes.len(),
            });
        }

        let mut memory = [0_u8; MEMORY_SIZE_BYTES];
        memory[..HEX_SPRITE_DATA.len()].copy_from_slice(&HEX_SPRITE_DATA);
        memory[PROGRAM_START..PROGRAM_START + program_bytes.len()].copy_from_slice(&program_bytes);

        Ok(Processor {
            memory,
            registers: Registers::new(),
            stack: [Address::from(0); STACK_SIZE],
            program_counter: Address::from(PROGRAM_START as u16),
            stack_pointer: 0,
            display: Display::new(config.display_width, config.display_height),
        })
    }

    pub fn step(&mut self) -> Result<(), ProcessorError> {
        let instruction_bytes = self.fetch();

        let instruction =
            instructions::decode(instruction_bytes).ok_or(ProcessorError::DecodeFailure {
                instruction: instruction_bytes,
            })?;

        self.execute(instruction)?;

        Ok(())
    }

    pub fn get_display_buffer(&mut self) -> Option<&Grid<Pixel>> {
        self.display.get_display_buffer()
    }

    fn fetch(&self) -> instructions::InstructionBytePair {
        let instruction_index = u16::from(self.program_counter) as usize;
        let instruction_bytes: [u8; 2] =
            core::array::from_fn(|idx| self.memory[instruction_index + idx]);
        instructions::InstructionBytePair(u16::from_be_bytes(instruction_bytes))
    }

    fn pc_skip(&mut self) {
        self.program_counter.increment(4);
    }

    fn pc_advance(&mut self) {
        self.program_counter.increment(2);
    }

    fn execute(&mut self, instruction: Instruction) -> Result<(), ProcessorError> {
        match instruction {
            Instruction::Sys { .. } => {
                self.pc_advance();
            }

            Instruction::Clear => {
                self.display.clear();
                self.pc_advance();
            }

            Instruction::Return => {
                if self.stack_pointer == 0 {
                    return Err(ProcessorError::StackUnderflow {
                        address: self.program_counter,
                    });
                }
                self.program_counter = self.stack[self.stack_pointer];
                self.stack_pointer -= 1;
                self.pc_advance();
            }

            Instruction::Jump { addr } => self.program_counter = addr,

            Instruction::Call { addr } => {
                self.stack_pointer += 1;
                if self.stack_pointer >= STACK_SIZE {
                    return Err(ProcessorError::StackOverflow {
                        address: self.program_counter,
                    });
                }

                self.stack[self.stack_pointer] = self.program_counter;
                self.program_counter = addr;
            }

            Instruction::SkipIfEqByte { reg, value } => {
                if self.registers.get_general(reg) == value {
                    self.pc_skip();
                } else {
                    self.pc_advance();
                }
            }

            Instruction::SkipIfNeqByte { reg, value } => {
                if self.registers.get_general(reg) != value {
                    self.pc_skip();
                } else {
                    self.pc_advance();
                }
            }

            Instruction::SkipIfEqReg { lhs, rhs } => {
                if self.registers.get_general(lhs) == self.registers.get_general(rhs) {
                    self.pc_skip();
                } else {
                    self.pc_advance();
                }
            }

            Instruction::LoadValue { dest, value } => {
                self.registers.set_general(dest, value);
                self.pc_advance();
            }

            Instruction::AddValue { dest, value } => {
                let current = self.registers.get_general(dest);
                let (result, _) = current.overflowing_add(value);
                self.registers.set_general(dest, result);
                self.pc_advance();
            }

            Instruction::LoadRegister { dest, source } => {
                let src_value = self.registers.get_general(source);
                self.registers.set_general(dest, src_value);
                self.pc_advance();
            }

            Instruction::Or { dest, source } => {
                let lhs = self.registers.get_general(dest);
                let rhs = self.registers.get_general(source);
                self.registers.set_general(dest, lhs | rhs);
                self.pc_advance();
            }

            Instruction::And { dest, source } => {
                let lhs = self.registers.get_general(dest);
                let rhs = self.registers.get_general(source);
                self.registers.set_general(dest, lhs & rhs);
                self.pc_advance();
            }

            Instruction::Xor { dest, source } => {
                let lhs = self.registers.get_general(dest);
                let rhs = self.registers.get_general(source);
                self.registers.set_general(dest, lhs ^ rhs);
                self.pc_advance();
            }

            Instruction::AddRegister { dest, source } => {
                let lhs = self.registers.get_general(dest);
                let rhs = self.registers.get_general(source);
                let (result, carry) = lhs.overflowing_add(rhs);
                self.registers.set_general(dest, result);
                if carry {
                    self.registers.set_vf_flag(Flag::High);
                } else {
                    self.registers.set_vf_flag(Flag::Low);
                }
                self.pc_advance();
            }

            Instruction::Subtract { dest, source } => {
                let lhs = self.registers.get_general(dest);
                let rhs = self.registers.get_general(source);
                let (result, borrow) = lhs.overflowing_sub(rhs);
                self.registers.set_general(dest, result);
                if !borrow {
                    self.registers.set_vf_flag(Flag::High);
                } else {
                    self.registers.set_vf_flag(Flag::Low);
                }
                self.pc_advance();
            }

            Instruction::ShiftRight { dest, .. } => {
                let value = self.registers.get_general(dest);
                let lsb = value & 0x01_u8;
                self.registers.set_general(dest, value >> 1);

                if lsb == 0x01_u8 {
                    self.registers.set_vf_flag(Flag::High);
                } else {
                    self.registers.set_vf_flag(Flag::Low);
                }

                self.pc_advance();
            }

            Instruction::SubtractNegate { dest, source } => {
                let lhs = self.registers.get_general(dest);
                let rhs = self.registers.get_general(source);
                let (result, borrow) = rhs.overflowing_sub(lhs);
                self.registers.set_general(dest, result);
                if !borrow {
                    self.registers.set_vf_flag(Flag::High);
                } else {
                    self.registers.set_vf_flag(Flag::Low);
                }
                self.pc_advance();
            }

            Instruction::ShiftLeft { dest, .. } => {
                let value = self.registers.get_general(dest);
                let msb = (value & 0b10000000_u8) >> 7;
                self.registers.set_general(dest, value << 1);
                if msb == 0x01_u8 {
                    self.registers.set_vf_flag(Flag::High);
                } else {
                    self.registers.set_vf_flag(Flag::Low);
                }
                self.pc_advance();
            }

            Instruction::SkipIfNeqReg { lhs, rhs } => {
                if self.registers.get_general(lhs) != self.registers.get_general(rhs) {
                    self.pc_skip();
                } else {
                    self.pc_advance();
                }
            }

            Instruction::LoadI { addr } => {
                self.registers.i = addr;
                self.pc_advance();
            }

            Instruction::JumpPlusV0 { addr } => {
                let new_address = Address::from(
                    self.registers.get_general(GeneralRegister::V0) as u16 + u16::from(addr),
                );
                self.program_counter = new_address;
            }

            Instruction::Random { dest, mask } => {
                let random_value: u8 = rand::random();
                self.registers.set_general(dest, random_value & mask);
                self.pc_advance();
            }

            Instruction::Draw { x, y, num_bytes } => {
                let draw_start = u16::from(self.registers.i) as usize;
                let draw_end = draw_start + num_bytes as usize;

                if draw_end > MEMORY_SIZE_BYTES {
                    return Err(ProcessorError::MemoryOverrun {
                        address: self.program_counter,
                    });
                }

                let bytes_to_draw = &self.memory[draw_start..draw_end];
                self.display.draw_sprite(
                    self.registers.get_general(x) as usize,
                    self.registers.get_general(y) as usize,
                    bytes_to_draw,
                );
                self.pc_advance();
            }

            Instruction::SkipIfKeyDown { .. } => {
                unimplemented!()
            }

            Instruction::SkipIfKeyUp { .. } => {
                unimplemented!()
            }

            Instruction::LoadFromDelayTimer { dest } => {
                self.registers.set_general(dest, self.registers.delay);
                self.pc_advance();
            }

            Instruction::LoadFromKey { .. } => {
                unimplemented!()
            }

            Instruction::SetDelayTimer { source } => {
                self.registers.delay = self.registers.get_general(source);
                self.pc_advance();
            }

            Instruction::SetSoundTimer { source } => {
                self.registers.sound = self.registers.get_general(source);
                self.pc_advance();
            }

            Instruction::AddI { source } => {
                let base: u16 = self.registers.i.into();
                let offset: u16 = self.registers.get_general(source) as u16;
                self.registers.i = Address::from(base + offset);
                self.pc_advance();
            }

            Instruction::LoadSpriteLocation { digit } => {
                let hex_digit = self.registers.get_general(digit);
                let hex_sprite_address = (hex_digit & 0x0F) as u16 * HEX_SPRITE_STRIDE as u16;

                self.registers.i = Address::from(hex_sprite_address);

                self.pc_advance();
            }

            Instruction::LoadBcd { source } => {
                let target_address = u16::from(self.registers.i) as usize;
                if target_address + 3 > MEMORY_SIZE_BYTES {
                    return Err(ProcessorError::MemoryOverrun {
                        address: self.program_counter,
                    });
                }

                let binary_value = self.registers.get_general(source);
                let bcd_digits = to_bcd(binary_value);

                self.memory[target_address..target_address + bcd_digits.len()]
                    .copy_from_slice(&bcd_digits);

                self.pc_advance();
            }

            Instruction::StoreRegisterRangeAtI { last } => {
                let mut dest_address = u16::from(self.registers.i) as usize;
                for reg in GeneralRegister::iter().take(last as usize + 1) {
                    if dest_address > MEMORY_SIZE_BYTES {
                        return Err(ProcessorError::MemoryOverrun {
                            address: self.program_counter,
                        });
                    }
                    self.memory[dest_address] = self.registers.get_general(reg);
                    dest_address += 1;
                }
                self.pc_advance();
            }

            Instruction::LoadRegisterRangeFromI { last } => {
                let mut src_address = u16::from(self.registers.i) as usize;
                for reg in GeneralRegister::iter().take(last as usize + 1) {
                    if src_address > MEMORY_SIZE_BYTES {
                        return Err(ProcessorError::MemoryOverrun {
                            address: self.program_counter,
                        });
                    }
                    self.registers.set_general(reg, self.memory[src_address]);
                    src_address += 1;
                }
                self.pc_advance();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common_test_data::{BCD_INPUT_BYTES, BCD_OUTPUT_DIGITS};
    use std::u8;

    #[test]
    fn test_to_bcd() {
        for (test_byte, expected_bytes) in BCD_INPUT_BYTES
            .into_iter()
            .zip(BCD_OUTPUT_DIGITS.into_iter())
        {
            assert_eq!(to_bcd(test_byte), expected_bytes);
        }
    }

    #[test]
    fn test_pc_advances() {
        let mut proc = Processor::new(vec![]).unwrap();
        let initial_pc = proc.program_counter;
        let num_cycles = 3;
        for _ in 0..num_cycles {
            proc.step().unwrap();
        }

        assert_eq!(
            proc.program_counter,
            Address::from((num_cycles * 2) + u16::from(initial_pc))
        );
    }

    #[test]
    fn test_invalid_instruction() {
        let mut proc = Processor::new(vec![0xF0_u8, 0x01_u8]).unwrap();
        assert!(matches!(
            proc.step(),
            Err(ProcessorError::DecodeFailure { .. })
        ));
    }

    #[test]
    fn test_sys() {
        // The SYS instruction is 0x0nnn, and should be ignored
        let mut proc = Processor::new(vec![0x00, 0x00]).unwrap();
        proc.step().unwrap();
    }

    #[test]
    fn test_return() {
        let mut proc = Processor::new(vec![
            0x00, 0x00, // empty      : addr 0x200
            0x22, 0x06, // call 0x206 : addr 0x202
            0x00, 0x00, // empty      : addr 0x204
            0x00, 0xEE, // return     : addr 0x206
        ])
        .unwrap();

        // step once so we get a nonzero pc
        proc.step().unwrap();

        // execute the call
        proc.step().unwrap();

        assert_eq!(proc.program_counter, Address::from(0x206));
        assert_eq!(proc.stack_pointer, 1);

        // execute the return
        proc.step().unwrap();

        assert_eq!(proc.program_counter, Address::from(0x204)); // one past call site
        assert_eq!(proc.stack_pointer, 0);
    }

    #[test]
    fn test_stack_underflow() {
        let mut proc = Processor::new(vec![
            0x00, 0x00, // empty      : addr 0x200
            0x00, 0xEE, // return     : addr 0x202
        ])
        .unwrap();

        // step once so we get a nonzero pc
        proc.step().unwrap();

        // return with empty call stack
        let result = proc.step();

        assert_eq!(
            result,
            Err(ProcessorError::StackUnderflow {
                address: Address::from(0x202)
            })
        );
    }

    #[test]
    fn test_jump() {
        let mut proc = Processor::new(vec![0x1A, 0xAA]).unwrap();
        proc.step().unwrap();
        assert_eq!(proc.program_counter, Address::from(0xAAA));
        assert_eq!(proc.stack_pointer, 0);
    }

    #[test]
    fn test_call() {
        // call 0xAAA
        let mut proc = Processor::new(vec![0x00, 0x00, 0x2A, 0xAA]).unwrap();

        // step once so we get a nonzero pc
        proc.step().unwrap();

        // save off the current pc, which should end up on the top of the stack
        let old_pc = proc.program_counter;

        // execute the call
        proc.step().unwrap();

        assert_eq!(proc.program_counter, Address::from(0xAAA));
        assert_eq!(proc.stack_pointer, 1);
        assert_eq!(proc.stack[proc.stack_pointer], old_pc);
    }

    #[test]
    fn test_stack_overflow() {
        let mut proc = Processor::new(vec![
            0x22, 0x00, // call 0x200 : addr 0x200
        ])
        .unwrap();

        for _ in 0..15 {
            // fill up the call stack
            proc.step().unwrap();
        }

        // call again to overflow
        let result = proc.step();

        assert_eq!(
            result,
            Err(ProcessorError::StackOverflow {
                address: Address::from(0x200)
            })
        );
    }

    #[test]
    fn test_skip_if_eq_byte_false() {
        let mut proc = Processor::new(vec![
            0x32, 0x10, // SE V2, 0x10 : addr 0x200
            0x00, 0x00, // empty       : addr 0x202
            0x00, 0x00, // empty       : addr 0x204
        ])
        .unwrap();
        assert_eq!(proc.registers.get_general(GeneralRegister::V2), 0x00_u8);

        proc.step().unwrap();

        // The register holds 0x00, so we should not have skipped
        assert_eq!(proc.program_counter, Address::from(0x202));
    }

    #[test]
    fn test_skip_if_eq_byte_true() {
        let mut proc = Processor::new(vec![
            0x32, 0x10, // SE V2, 0x10 : addr 0x200
            0x00, 0x00, // empty       : addr 0x202
            0x00, 0x00, // empty       : addr 0x204
        ])
        .unwrap();

        // manually tinker with the register to have the equality high
        proc.registers.set_general(GeneralRegister::V2, 0x10_u8);

        proc.step().unwrap();

        // took the true branch this time, so we should have skipped 0x202
        assert_eq!(proc.program_counter, Address::from(0x204));
    }

    #[test]
    fn test_skip_if_neq_byte_false() {
        let mut proc = Processor::new(vec![
            0x42, 0x10, // SNE V2, 0x10 : addr 0x200
            0x00, 0x00, // empty        : addr 0x202
            0x00, 0x00, // empty        : addr 0x204
        ])
        .unwrap();

        proc.registers.set_general(GeneralRegister::V2, 0x10_u8);
        assert_eq!(proc.registers.get_general(GeneralRegister::V2), 0x10_u8);

        proc.step().unwrap();

        // The register holds 0x10, so we should not have skipped
        assert_eq!(proc.program_counter, Address::from(0x202));
    }

    #[test]
    fn test_skip_if_neq_byte_true() {
        let mut proc = Processor::new(vec![
            0x42, 0x10, // SNE V2, 0x10 : addr 0x200
            0x00, 0x00, // empty        : addr 0x202
            0x00, 0x00, // empty        : addr 0x204
        ])
        .unwrap();

        // manually tinker with the register to have the equality high
        proc.registers.set_general(GeneralRegister::V2, 0x00_u8);
        assert_eq!(proc.registers.get_general(GeneralRegister::V2), 0x00_u8);

        proc.step().unwrap();

        // took the true branch this time, so we should have skipped 0x202
        assert_eq!(proc.program_counter, Address::from(0x204));
    }

    #[test]
    fn test_skip_if_eq_reg_false() {
        let mut proc = Processor::new(vec![
            0x51, 0x20, // SE V1, V2 : addr 0x200
            0x00, 0x00, // empty     : addr 0x202
            0x00, 0x00, // empty     : addr 0x204
        ])
        .unwrap();

        // manually offset the registers V1 and V2
        proc.registers.set_general(GeneralRegister::V1, 102_u8);
        proc.registers.set_general(GeneralRegister::V2, 201_u8);

        proc.step().unwrap();

        // we should not have skipped, and so landed on 0x202
        assert_eq!(proc.program_counter, Address::from(0x202));
    }

    #[test]
    fn test_skip_if_eq_reg_true() {
        let mut proc = Processor::new(vec![
            0x51, 0x20, // SE V1, V2 : addr 0x200
            0x00, 0x00, // empty     : addr 0x202
            0x00, 0x00, // empty     : addr 0x204
        ])
        .unwrap();

        // manually align the registers V1 and V2
        proc.registers.set_general(GeneralRegister::V1, 123_u8);
        proc.registers.set_general(GeneralRegister::V2, 123_u8);

        proc.step().unwrap();

        // we should have skipped, and so landed on 0x204
        assert_eq!(proc.program_counter, Address::from(0x204));
    }

    #[test]
    fn test_load_value() {
        let mut proc = Processor::new(vec![
            0x67, 0x89, // LD V7, 0x89 : addr 0x200
        ])
        .unwrap();

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V7), 0x89_u8);
    }

    #[test]
    fn test_add_value() {
        let init = 0x12_u8;

        let mut proc = Processor::new(vec![
            0x70, init, // ADD V0, 0x34
        ])
        .unwrap();

        let summand = 0x34_u8;
        proc.registers.set_general(GeneralRegister::V0, summand);

        proc.step().unwrap();

        assert_eq!(
            proc.registers.get_general(GeneralRegister::V0),
            init + summand
        );
    }

    #[test]
    fn test_add_value_overflow() {
        let init = 0xEE_u8;

        let mut proc = Processor::new(vec![
            0x70, init, // ADD V0, 0x34
        ])
        .unwrap();

        let summand = 0xCC_u8;
        proc.registers.set_general(GeneralRegister::V0, summand);

        let initial_vf = 0x56_u8;
        proc.registers.set_general(GeneralRegister::VF, initial_vf);

        // should wrap on overflow
        let expected = ((init as u16 + summand as u16) % (u8::MAX as u16 + 1)) as u8;

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V0), expected);

        // this instruction does not affect the overflow flag
        assert_eq!(proc.registers.get_general(GeneralRegister::VF), initial_vf);
    }

    #[test]
    fn test_load_register() {
        let mut proc = Processor::new(vec![
            0x81, 0x20, // LD V1, V2
        ])
        .unwrap();

        proc.registers.set_general(GeneralRegister::V1, 0x01_u8);
        proc.registers.set_general(GeneralRegister::V2, 0x02_u8);

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), 0x02_u8);
    }

    #[test]
    fn test_or() {
        let mut proc = Processor::new(vec![
            0x81, 0x21, // OR V1, V2
        ])
        .unwrap();

        let lhs = 0xF0_u8;
        let rhs = 0xAA_u8;
        let expected = lhs | rhs;

        proc.registers.set_general(GeneralRegister::V1, lhs);
        proc.registers.set_general(GeneralRegister::V2, rhs);

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), expected);
    }

    #[test]
    fn test_and() {
        let mut proc = Processor::new(vec![
            0x81, 0x22, // AND V1, V2
        ])
        .unwrap();

        let lhs = 0xF0_u8;
        let rhs = 0xAA_u8;
        let expected = lhs & rhs;

        proc.registers.set_general(GeneralRegister::V1, lhs);
        proc.registers.set_general(GeneralRegister::V2, rhs);

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), expected);
    }

    #[test]
    fn test_xor() {
        let mut proc = Processor::new(vec![
            0x81, 0x23, // AND V1, V2
        ])
        .unwrap();

        let lhs = 0xF0_u8;
        let rhs = 0xAA_u8;
        let expected = lhs ^ rhs;

        proc.registers.set_general(GeneralRegister::V1, lhs);
        proc.registers.set_general(GeneralRegister::V2, rhs);

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), expected);
    }

    #[test]
    fn test_add_register() {
        let mut proc = Processor::new(vec![
            0x81, 0x24, // ADD V1, V2
        ])
        .unwrap();

        // set vf to some value so we can check this instruction has affected the overflow flag
        let initial_vf = 0x56_u8;
        proc.registers.set_general(GeneralRegister::VF, initial_vf);
        assert_eq!(proc.registers.get_vf_flag(), None);

        let lhs = 0x12_u8;
        let rhs = 0x34_u8;
        proc.registers.set_general(GeneralRegister::V1, lhs);
        proc.registers.set_general(GeneralRegister::V2, rhs);

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), lhs + rhs);

        // should not have overflowed
        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::Low));
    }

    #[test]
    fn test_add_register_overflow() {
        let mut proc = Processor::new(vec![
            0x81, 0x24, // ADD V1, V2
        ])
        .unwrap();

        // set vf to some value so we can check this instruction has affected the overflow flag
        let initial_vf = 0x56_u8;
        proc.registers.set_general(GeneralRegister::VF, initial_vf);
        assert_eq!(proc.registers.get_vf_flag(), None);

        let lhs = 0xEE_u8;
        let rhs = 0xCC_u8;
        proc.registers.set_general(GeneralRegister::V1, lhs);
        proc.registers.set_general(GeneralRegister::V2, rhs);

        // should wrap on overflow
        let expected = ((lhs as u16 + rhs as u16) % (u8::MAX as u16 + 1)) as u8;

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), expected);

        // should not have overflowed
        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::High));
    }

    #[test]
    fn test_subtract() {
        let mut proc = Processor::new(vec![
            0x81, 0x25, // SUB V1, V2
        ])
        .unwrap();

        // set vf to some value so we can check this instruction has affected the overflow flag
        let initial_vf = 0x56_u8;
        proc.registers.set_general(GeneralRegister::VF, initial_vf);
        assert_eq!(proc.registers.get_vf_flag(), None);

        let lhs = 0x43_u8;
        let rhs = 0x21_u8;
        proc.registers.set_general(GeneralRegister::V1, lhs);
        proc.registers.set_general(GeneralRegister::V2, rhs);

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), lhs - rhs);

        // should not have underflowed
        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::High));
    }

    #[test]
    fn test_subtract_underflow() {
        let mut proc = Processor::new(vec![
            0x81, 0x25, // SUB V1, V2
        ])
        .unwrap();

        // set vf to some value so we can check this instruction has affected the overflow flag
        let initial_vf = 0x56_u8;
        proc.registers.set_general(GeneralRegister::VF, initial_vf);
        assert_eq!(proc.registers.get_vf_flag(), None);

        let lhs = 0x12_u8;
        let rhs = 0x34_u8;
        proc.registers.set_general(GeneralRegister::V1, lhs);
        proc.registers.set_general(GeneralRegister::V2, rhs);

        // should wrap on overflow
        let expected = (lhs as i16 - rhs as i16) as u8;

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), expected);

        // should have underflowed
        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::Low));
    }

    #[test]
    fn test_shift_right_lsb_high() {
        let mut proc = Processor::new(vec![
            0x81, 0x26, // SHR V1 {, V2}
        ])
        .unwrap();

        let initial_value = 0b01010101_u8;
        proc.registers
            .set_general(GeneralRegister::V1, initial_value);

        proc.step().unwrap();

        assert_eq!(
            proc.registers.get_general(GeneralRegister::V1),
            initial_value >> 1
        );

        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::High));
    }

    #[test]
    fn test_shift_right_lsb_low() {
        let mut proc = Processor::new(vec![
            0x81, 0x26, // SHR V1 {, V2}
        ])
        .unwrap();

        let initial_value = 0b10101010_u8;
        proc.registers
            .set_general(GeneralRegister::V1, initial_value);

        proc.step().unwrap();

        assert_eq!(
            proc.registers.get_general(GeneralRegister::V1),
            initial_value >> 1
        );

        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::Low));
    }

    #[test]
    fn test_subtract_negate() {
        let mut proc = Processor::new(vec![
            0x81, 0x27, // SUBN V1, V2
        ])
        .unwrap();

        // set vf to some value so we can check this instruction has affected the overflow flag
        let initial_vf = 0x56_u8;
        proc.registers.set_general(GeneralRegister::VF, initial_vf);
        assert_eq!(proc.registers.get_vf_flag(), None);

        let rhs = 0x43_u8;
        let lhs = 0x21_u8;
        proc.registers.set_general(GeneralRegister::V1, lhs);
        proc.registers.set_general(GeneralRegister::V2, rhs);

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), rhs - lhs);

        // should not have underflowed
        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::High));
    }

    #[test]
    fn test_subtract_negate_underflow() {
        let mut proc = Processor::new(vec![
            0x81, 0x27, // SUBN V1, V2
        ])
        .unwrap();

        // set vf to some value so we can check this instruction has affected the overflow flag
        let initial_vf = 0x56_u8;
        proc.registers.set_general(GeneralRegister::VF, initial_vf);
        assert_eq!(proc.registers.get_vf_flag(), None);

        let rhs = 0x12_u8;
        let lhs = 0x34_u8;
        proc.registers.set_general(GeneralRegister::V1, lhs);
        proc.registers.set_general(GeneralRegister::V2, rhs);

        // should wrap on overflow
        let expected = (rhs as i16 - lhs as i16) as u8;

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::V1), expected);

        // should have underflowed
        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::Low));
    }

    #[test]
    fn test_shift_left_msb_high() {
        let mut proc = Processor::new(vec![
            0x81, 0x2E, // SHL V1 {, V2}
        ])
        .unwrap();

        let initial_value = 0b10101010_u8;
        proc.registers
            .set_general(GeneralRegister::V1, initial_value);

        proc.step().unwrap();

        assert_eq!(
            proc.registers.get_general(GeneralRegister::V1),
            initial_value << 1
        );

        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::High));
    }

    #[test]
    fn test_shift_left_msb_low() {
        let mut proc = Processor::new(vec![
            0x81, 0x2E, // SHL V1 {, V2}
        ])
        .unwrap();

        let initial_value = 0b01010101_u8;
        proc.registers
            .set_general(GeneralRegister::V1, initial_value);

        proc.step().unwrap();

        assert_eq!(
            proc.registers.get_general(GeneralRegister::V1),
            initial_value << 1
        );

        assert_eq!(proc.registers.get_vf_flag(), Some(Flag::Low));
    }

    #[test]
    fn test_skip_if_neq_reg_false() {
        let mut proc = Processor::new(vec![
            0x91, 0x20, // SNE V1, V2 : addr 0x200
            0x00, 0x00, // empty      : addr 0x202
            0x00, 0x00, // empty      : addr 0x204
        ])
        .unwrap();

        // manually align the registers V1 and V2
        proc.registers.set_general(GeneralRegister::V1, 123_u8);
        proc.registers.set_general(GeneralRegister::V2, 123_u8);

        proc.step().unwrap();

        // we should not have skipped, and so landed on 0x202
        assert_eq!(proc.program_counter, Address::from(0x202));
    }

    #[test]
    fn test_skip_if_neq_reg_true() {
        let mut proc = Processor::new(vec![
            0x91, 0x20, // SE V1, V2 : addr 0x200
            0x00, 0x00, // empty     : addr 0x202
            0x00, 0x00, // empty     : addr 0x204
        ])
        .unwrap();

        // manually offset the registers V1 and V2
        proc.registers.set_general(GeneralRegister::V1, 102_u8);
        proc.registers.set_general(GeneralRegister::V2, 201_u8);

        proc.step().unwrap();

        // we should have skipped, and so landed on 0x204
        assert_eq!(proc.program_counter, Address::from(0x204));
    }

    #[test]
    fn test_load_i() {
        let mut proc = Processor::new(vec![
            0xA1, 0x23, // LD I, 0x123
        ])
        .unwrap();

        proc.step().unwrap();

        assert_eq!(proc.registers.i, Address::from(0x123));
    }

    #[test]
    fn test_jump_plus_v0() {
        let mut proc = Processor::new(vec![
            0xB3, 0x01, // JP V0, 0x301 : addr 0x200
        ])
        .unwrap();

        proc.registers.set_general(GeneralRegister::V0, 0x20_u8);

        proc.step().unwrap();

        assert_eq!(proc.program_counter, Address::from(0x321));
    }

    #[test]
    fn test_load_from_delay_timer() {
        let mut proc = Processor::new(vec![
            0xFA, 0x07, // LD VA, DT
        ])
        .unwrap();

        proc.registers.delay = 0xBC;

        proc.step().unwrap();

        assert_eq!(proc.registers.get_general(GeneralRegister::VA), 0xBC);
    }

    #[test]
    fn test_set_delay_timer() {
        let mut proc = Processor::new(vec![
            0xFB, 0x15, // LD DT, VB
        ])
        .unwrap();

        proc.registers.set_general(GeneralRegister::VB, 0xBC);

        proc.step().unwrap();

        assert_eq!(proc.registers.delay, 0xBC);
    }

    #[test]
    fn test_set_sound_timer() {
        let mut proc = Processor::new(vec![
            0xFB, 0x18, // LD ST, VB
        ])
        .unwrap();

        proc.registers.set_general(GeneralRegister::VB, 0xBC);

        proc.step().unwrap();

        assert_eq!(proc.registers.sound, 0xBC);
    }

    #[test]
    fn test_add_i() {
        let mut proc = Processor::new(vec![
            0xF4, 0x1E, // ADD I, V4
        ])
        .unwrap();

        let initial = Address::from(0x300);
        let offset = 0x21_u8;

        proc.registers.i = initial;
        proc.registers.set_general(GeneralRegister::V4, offset);

        proc.step().unwrap();

        assert_eq!(
            proc.registers.i,
            Address::from(u16::from(initial) + offset as u16)
        );
    }

    #[test]
    fn test_load_sprite_location() {
        for sprite_idx in 0..16_u8 {
            let mut proc = Processor::new(vec![
                0xF0, 0x29, // LD F, V0
            ])
            .unwrap();

            proc.registers.set_general(GeneralRegister::V0, sprite_idx);

            proc.step().unwrap();

            assert_eq!(
                proc.registers.i,
                Address::from(sprite_idx as u16 * HEX_SPRITE_STRIDE as u16)
            );
        }
    }

    #[test]
    fn test_load_bcd() {
        for (test_byte, expected_digits) in BCD_INPUT_BYTES
            .into_iter()
            .zip(BCD_OUTPUT_DIGITS.into_iter())
        {
            let mut proc = Processor::new(vec![
                0xF8, 0x33, // LD B, V8
            ])
            .unwrap();

            proc.registers.set_general(GeneralRegister::V8, test_byte);
            proc.registers.i = Address::from(0x400);

            proc.step().unwrap();

            let target_idx = u16::from(proc.registers.i) as usize;

            assert_eq!(expected_digits, proc.memory[target_idx..target_idx + 3]);
        }
    }

    #[test]
    fn test_store_register_range_at_i() {
        for reg_end in 0..16_u8 {
            let ld_i_vx = 0xF0_u8 | reg_end;
            let mut proc = Processor::new(vec![
                ld_i_vx, 0x55, // LD [I], VX
            ])
            .unwrap();

            for (idx, reg) in GeneralRegister::iter().enumerate() {
                proc.registers.set_general(reg, idx as u8);
            }

            let target_addr = Address::from(0x400);

            proc.registers.i = target_addr;

            proc.step().unwrap();

            // V0 to VX inclusive have been written to the target address
            for idx in 0..=reg_end as usize {
                assert_eq!(
                    proc.memory[u16::from(target_addr) as usize + idx],
                    idx as u8
                );
            }

            // the remaining have not
            for idx in reg_end as usize + 1..16 {
                assert_eq!(proc.memory[u16::from(target_addr) as usize + idx], 0x00_u8);
            }
        }
    }

    #[test]
    fn test_load_register_range_from_i() {
        for reg_end in 0..16_u8 {
            let ld_i_vx = 0xF0_u8 | reg_end;
            let mut proc = Processor::new(vec![
                ld_i_vx, 0x65, // LD VX, [I]
            ])
            .unwrap();

            let target_addr = Address::from(0x400);
            proc.registers.i = target_addr;
            for idx in 0..16 {
                proc.memory[idx + u16::from(target_addr) as usize] = idx as u8;
            }

            proc.step().unwrap();

            // V0 to VX inclusive have been set
            for (val, reg) in GeneralRegister::iter()
                .enumerate()
                .take(reg_end as usize + 1)
            {
                assert_eq!(proc.registers.get_general(reg), val as u8);
            }

            // the remaining have not
            for reg in GeneralRegister::iter().skip(reg_end as usize + 1) {
                assert_eq!(proc.registers.get_general(reg), 0x00_u8);
            }
        }
    }
}
