#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Register {
    pub const fn index(self) -> u8 {
        match self {
            Register::R0 => 0,
            Register::R1 => 1,
            Register::R2 => 2,
            Register::R3 => 3,
            Register::R4 => 4,
            Register::R5 => 5,
            Register::R6 => 6,
            Register::R7 => 7,
            Register::R8 => 8,
            Register::R9 => 9,
            Register::R10 => 10,
            Register::R11 => 11,
            Register::R12 => 12,
            Register::R13 => 13,
            Register::R14 => 14,
            Register::R15 => 15,
        }
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Register::R0),
            1 => Some(Register::R1),
            2 => Some(Register::R2),
            3 => Some(Register::R3),
            4 => Some(Register::R4),
            5 => Some(Register::R5),
            6 => Some(Register::R6),
            7 => Some(Register::R7),
            8 => Some(Register::R8),
            9 => Some(Register::R9),
            10 => Some(Register::R10),
            11 => Some(Register::R11),
            12 => Some(Register::R12),
            13 => Some(Register::R13),
            14 => Some(Register::R14),
            15 => Some(Register::R15),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opcode {
    RType = 0x00,
    Addi = 0x01,
    Ld = 0x02,
    St = 0x03,
    Beq = 0x04,
    Bne = 0x05,
    Blt = 0x06,
    Bge = 0x07,
    Jal = 0x08,
    Jmp = 0x09,
    Ecall = 0x0A,
}

impl Opcode {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Opcode::RType),
            0x01 => Some(Opcode::Addi),
            0x02 => Some(Opcode::Ld),
            0x03 => Some(Opcode::St),
            0x04 => Some(Opcode::Beq),
            0x05 => Some(Opcode::Bne),
            0x06 => Some(Opcode::Blt),
            0x07 => Some(Opcode::Bge),
            0x08 => Some(Opcode::Jal),
            0x09 => Some(Opcode::Jmp),
            0x0A => Some(Opcode::Ecall),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instr {
    Add { rd: Register, rs1: Register, rs2: Register },
    Sub { rd: Register, rs1: Register, rs2: Register },
    And { rd: Register, rs1: Register, rs2: Register },
    Or { rd: Register, rs1: Register, rs2: Register },
    Xor { rd: Register, rs1: Register, rs2: Register },
    Shl { rd: Register, rs1: Register, rs2: Register },
    Shr { rd: Register, rs1: Register, rs2: Register },
    Addi { rd: Register, rs1: Register, imm: i16 },
    Ld { rd: Register, base: Register, offset: i16 },
    St { rs: Register, base: Register, offset: i16 },
    Beq { rs1: Register, rs2: Register, offset: i16 },
    Bne { rs1: Register, rs2: Register, offset: i16 },
    Blt { rs1: Register, rs2: Register, offset: i16 },
    Bge { rs1: Register, rs2: Register, offset: i16 },
    Jal { offset: i32 },
    Jmp { offset: i32 },
    Ecall { imm: i16 },
}
