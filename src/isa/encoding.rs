use super::instr::{Instr, Opcode, Register};

// Opcode table (bits [31:26]):
// 0x00 R-type, 0x01 ADDI, 0x02 LD, 0x03 ST, 0x04 BEQ, 0x05 BNE,
// 0x06 BLT, 0x07 BGE, 0x08 JAL, 0x09 JMP, 0x0A ECALL
// R-type funct14 table (bits [13:0]):
// 0x0001 ADD, 0x0002 SUB, 0x0003 AND, 0x0004 OR, 0x0005 XOR,
// 0x0006 SHL, 0x0007 SHR

const OPCODE_SHIFT: u32 = 26;
const OPCODE_MASK: u32 = 0x3F;
const RD_SHIFT: u32 = 22;
const RS1_SHIFT: u32 = 18;
const RS2_SHIFT: u32 = 14;
const REG_MASK: u32 = 0x0F;
const FUNCT_MASK: u32 = 0x3FFF;
const IMM16_MASK: u32 = 0xFFFF;
const IMM26_MASK: u32 = 0x03FF_FFFF;

const FUNCT_ADD: u16 = 0x0001;
const FUNCT_SUB: u16 = 0x0002;
const FUNCT_AND: u16 = 0x0003;
const FUNCT_OR: u16 = 0x0004;
const FUNCT_XOR: u16 = 0x0005;
const FUNCT_SHL: u16 = 0x0006;
const FUNCT_SHR: u16 = 0x0007;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    ReservedOpcode(u8),
    ReservedFunct(u16),
    ReservedField { field: &'static str, value: u32 },
    InvalidRegister { field: &'static str, value: u8 },
}

pub fn encode(instr: &Instr) -> u32 {
    match *instr {
        Instr::Add { rd, rs1, rs2 } => encode_r(rd, rs1, rs2, FUNCT_ADD),
        Instr::Sub { rd, rs1, rs2 } => encode_r(rd, rs1, rs2, FUNCT_SUB),
        Instr::And { rd, rs1, rs2 } => encode_r(rd, rs1, rs2, FUNCT_AND),
        Instr::Or { rd, rs1, rs2 } => encode_r(rd, rs1, rs2, FUNCT_OR),
        Instr::Xor { rd, rs1, rs2 } => encode_r(rd, rs1, rs2, FUNCT_XOR),
        Instr::Shl { rd, rs1, rs2 } => encode_r(rd, rs1, rs2, FUNCT_SHL),
        Instr::Shr { rd, rs1, rs2 } => encode_r(rd, rs1, rs2, FUNCT_SHR),
        Instr::Addi { rd, rs1, imm } => encode_i(Opcode::Addi, rd, rs1, Register::R0, imm),
        Instr::Ld { rd, base, offset } => encode_i(Opcode::Ld, rd, base, Register::R0, offset),
        Instr::St { rs, base, offset } => encode_i(Opcode::St, rs, base, Register::R0, offset),
        Instr::Beq { rs1, rs2, offset } => encode_i(Opcode::Beq, Register::R0, rs1, rs2, offset),
        Instr::Bne { rs1, rs2, offset } => encode_i(Opcode::Bne, Register::R0, rs1, rs2, offset),
        Instr::Blt { rs1, rs2, offset } => encode_i(Opcode::Blt, Register::R0, rs1, rs2, offset),
        Instr::Bge { rs1, rs2, offset } => encode_i(Opcode::Bge, Register::R0, rs1, rs2, offset),
        Instr::Jal { offset } => encode_j(Opcode::Jal, offset),
        Instr::Jmp { offset } => encode_j(Opcode::Jmp, offset),
        Instr::Ecall { imm } => encode_i(Opcode::Ecall, Register::R0, Register::R0, Register::R0, imm),
    }
}

pub fn decode(word: u32) -> Result<Instr, DecodeError> {
    let opcode_raw = ((word >> OPCODE_SHIFT) & OPCODE_MASK) as u8;
    let opcode = Opcode::from_u8(opcode_raw).ok_or(DecodeError::ReservedOpcode(opcode_raw))?;

    match opcode {
        Opcode::RType => decode_r(word),
        Opcode::Addi => decode_i_addi(word),
        Opcode::Ld => decode_i_ld(word),
        Opcode::St => decode_i_st(word),
        Opcode::Beq => decode_i_branch(word, Opcode::Beq),
        Opcode::Bne => decode_i_branch(word, Opcode::Bne),
        Opcode::Blt => decode_i_branch(word, Opcode::Blt),
        Opcode::Bge => decode_i_branch(word, Opcode::Bge),
        Opcode::Jal => decode_j(word, Opcode::Jal),
        Opcode::Jmp => decode_j(word, Opcode::Jmp),
        Opcode::Ecall => decode_i_ecall(word),
    }
}

fn encode_r(rd: Register, rs1: Register, rs2: Register, funct: u16) -> u32 {
    let opcode = Opcode::RType as u32;
    (opcode << OPCODE_SHIFT)
        | ((rd.index() as u32) << RD_SHIFT)
        | ((rs1.index() as u32) << RS1_SHIFT)
        | ((rs2.index() as u32) << RS2_SHIFT)
        | (funct as u32 & FUNCT_MASK)
}

fn encode_i(opcode: Opcode, rd: Register, rs1: Register, rs2: Register, imm: i16) -> u32 {
    let opcode = opcode as u32;
    let imm_bits = (imm as u16) as u32;
    (opcode << OPCODE_SHIFT)
        | ((rd.index() as u32) << RD_SHIFT)
        | ((rs1.index() as u32) << RS1_SHIFT)
        | ((rs2.index() as u32) << RS2_SHIFT)
        | (imm_bits & IMM16_MASK)
}

fn encode_j(opcode: Opcode, imm: i32) -> u32 {
    debug_assert!(fits_i26(imm));
    let opcode = opcode as u32;
    let imm_bits = (imm as u32) & IMM26_MASK;
    (opcode << OPCODE_SHIFT) | imm_bits
}

fn decode_r(word: u32) -> Result<Instr, DecodeError> {
    let rd = decode_reg(word, RD_SHIFT, "rd")?;
    let rs1 = decode_reg(word, RS1_SHIFT, "rs1")?;
    let rs2 = decode_reg(word, RS2_SHIFT, "rs2")?;
    let funct = (word & FUNCT_MASK) as u16;

    match funct {
        FUNCT_ADD => Ok(Instr::Add { rd, rs1, rs2 }),
        FUNCT_SUB => Ok(Instr::Sub { rd, rs1, rs2 }),
        FUNCT_AND => Ok(Instr::And { rd, rs1, rs2 }),
        FUNCT_OR => Ok(Instr::Or { rd, rs1, rs2 }),
        FUNCT_XOR => Ok(Instr::Xor { rd, rs1, rs2 }),
        FUNCT_SHL => Ok(Instr::Shl { rd, rs1, rs2 }),
        FUNCT_SHR => Ok(Instr::Shr { rd, rs1, rs2 }),
        _ => Err(DecodeError::ReservedFunct(funct)),
    }
}

fn decode_i_addi(word: u32) -> Result<Instr, DecodeError> {
    let rd = decode_reg(word, RD_SHIFT, "rd")?;
    let rs1 = decode_reg(word, RS1_SHIFT, "rs1")?;
    let rs2 = decode_reg(word, RS2_SHIFT, "rs2")?;
    require_zero("rs2", rs2.index() as u32)?;
    let imm = decode_imm16(word);
    Ok(Instr::Addi { rd, rs1, imm })
}

fn decode_i_ld(word: u32) -> Result<Instr, DecodeError> {
    let rd = decode_reg(word, RD_SHIFT, "rd")?;
    let base = decode_reg(word, RS1_SHIFT, "rs1")?;
    let rs2 = decode_reg(word, RS2_SHIFT, "rs2")?;
    require_zero("rs2", rs2.index() as u32)?;
    let offset = decode_imm16(word);
    Ok(Instr::Ld { rd, base, offset })
}

fn decode_i_st(word: u32) -> Result<Instr, DecodeError> {
    let rs = decode_reg(word, RD_SHIFT, "rd")?;
    let base = decode_reg(word, RS1_SHIFT, "rs1")?;
    let rs2 = decode_reg(word, RS2_SHIFT, "rs2")?;
    require_zero("rs2", rs2.index() as u32)?;
    let offset = decode_imm16(word);
    Ok(Instr::St { rs, base, offset })
}

fn decode_i_branch(word: u32, opcode: Opcode) -> Result<Instr, DecodeError> {
    let rd = decode_reg(word, RD_SHIFT, "rd")?;
    require_zero("rd", rd.index() as u32)?;
    let rs1 = decode_reg(word, RS1_SHIFT, "rs1")?;
    let rs2 = decode_reg(word, RS2_SHIFT, "rs2")?;
    let offset = decode_imm16(word);

    match opcode {
        Opcode::Beq => Ok(Instr::Beq { rs1, rs2, offset }),
        Opcode::Bne => Ok(Instr::Bne { rs1, rs2, offset }),
        Opcode::Blt => Ok(Instr::Blt { rs1, rs2, offset }),
        Opcode::Bge => Ok(Instr::Bge { rs1, rs2, offset }),
        _ => Err(DecodeError::ReservedOpcode(opcode as u8)),
    }
}

fn decode_j(word: u32, opcode: Opcode) -> Result<Instr, DecodeError> {
    let imm = decode_imm26(word);
    match opcode {
        Opcode::Jal => Ok(Instr::Jal { offset: imm }),
        Opcode::Jmp => Ok(Instr::Jmp { offset: imm }),
        _ => Err(DecodeError::ReservedOpcode(opcode as u8)),
    }
}

fn decode_i_ecall(word: u32) -> Result<Instr, DecodeError> {
    let rd = decode_reg(word, RD_SHIFT, "rd")?;
    let rs1 = decode_reg(word, RS1_SHIFT, "rs1")?;
    let rs2 = decode_reg(word, RS2_SHIFT, "rs2")?;
    require_zero("rd", rd.index() as u32)?;
    require_zero("rs1", rs1.index() as u32)?;
    require_zero("rs2", rs2.index() as u32)?;
    let imm = decode_imm16(word);
    Ok(Instr::Ecall { imm })
}

fn decode_reg(word: u32, shift: u32, field: &'static str) -> Result<Register, DecodeError> {
    let raw = ((word >> shift) & REG_MASK) as u8;
    Register::from_u8(raw).ok_or(DecodeError::InvalidRegister { field, value: raw })
}

fn decode_imm16(word: u32) -> i16 {
    let raw = (word & IMM16_MASK) as u16;
    raw as i16
}

fn decode_imm26(word: u32) -> i32 {
    let raw = word & IMM26_MASK;
    ((raw << 6) as i32) >> 6
}

fn require_zero(field: &'static str, value: u32) -> Result<(), DecodeError> {
    if value == 0 {
        Ok(())
    } else {
        Err(DecodeError::ReservedField { field, value })
    }
}

fn fits_i26(value: i32) -> bool {
    let limit = 1 << 25;
    value >= -limit && value < limit
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};
    use crate::isa::instr::{Instr, Register};

    fn round_trip(instr: Instr) {
        let word = encode(&instr);
        let decoded = decode(word).expect("decode failed");
        assert_eq!(decoded, instr);
    }

    fn pc_relative_target(pc: u64, offset: i64) -> u64 {
        (pc as i64 + 4 + (offset << 2)) as u64
    }

    #[test]
    fn round_trip_all_instructions() {
        let instrs = vec![
            Instr::Add { rd: Register::R1, rs1: Register::R2, rs2: Register::R3 },
            Instr::Sub { rd: Register::R4, rs1: Register::R5, rs2: Register::R6 },
            Instr::And { rd: Register::R7, rs1: Register::R8, rs2: Register::R9 },
            Instr::Or { rd: Register::R10, rs1: Register::R11, rs2: Register::R12 },
            Instr::Xor { rd: Register::R13, rs1: Register::R14, rs2: Register::R15 },
            Instr::Shl { rd: Register::R2, rs1: Register::R3, rs2: Register::R4 },
            Instr::Shr { rd: Register::R5, rs1: Register::R6, rs2: Register::R7 },
            Instr::Addi { rd: Register::R8, rs1: Register::R9, imm: 123 },
            Instr::Ld { rd: Register::R10, base: Register::R11, offset: -16 },
            Instr::St { rs: Register::R12, base: Register::R13, offset: 32 },
            Instr::Beq { rs1: Register::R1, rs2: Register::R2, offset: 4 },
            Instr::Bne { rs1: Register::R3, rs2: Register::R4, offset: -8 },
            Instr::Blt { rs1: Register::R5, rs2: Register::R6, offset: 7 },
            Instr::Bge { rs1: Register::R7, rs2: Register::R8, offset: -3 },
            Instr::Jal { offset: 12 },
            Instr::Jmp { offset: -5 },
            Instr::Ecall { imm: 2 },
        ];

        for instr in instrs {
            round_trip(instr);
        }
    }

    #[test]
    fn sign_extension_imm16() {
        let instr = Instr::Addi { rd: Register::R1, rs1: Register::R2, imm: -1 };
        round_trip(instr);

        let instr = Instr::Ld { rd: Register::R3, base: Register::R4, offset: i16::MIN };
        round_trip(instr);
    }

    #[test]
    fn sign_extension_imm26() {
        let instr = Instr::Jmp { offset: -1 };
        round_trip(instr);

        let instr = Instr::Jal { offset: (1 << 25) - 1 };
        round_trip(instr);
    }

    #[test]
    fn branch_and_jump_offset_scaling() {
        let pc = 0x1000_u64;
        let offset = 3_i16;
        let instr = Instr::Beq { rs1: Register::R1, rs2: Register::R2, offset };
        let decoded = decode(encode(&instr)).expect("decode failed");
        if let Instr::Beq { offset: dec_off, .. } = decoded {
            let target = pc_relative_target(pc, dec_off as i64);
            assert_eq!(target, 0x1000 + 4 + 12);
        } else {
            panic!("unexpected decoded instruction");
        }

        let j_offset = -2_i32;
        let instr = Instr::Jal { offset: j_offset };
        let decoded = decode(encode(&instr)).expect("decode failed");
        if let Instr::Jal { offset: dec_off } = decoded {
            let target = pc_relative_target(pc, dec_off as i64);
            assert_eq!(target, 0x1000 + 4 - 8);
        } else {
            panic!("unexpected decoded instruction");
        }
    }

    #[test]
    fn r0_write_encodable() {
        let instr = Instr::Add { rd: Register::R0, rs1: Register::R1, rs2: Register::R2 };
        round_trip(instr);
    }
}
