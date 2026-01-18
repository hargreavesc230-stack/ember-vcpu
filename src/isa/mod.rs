pub mod encoding;
pub mod instr;

pub use encoding::{decode, encode, DecodeError};
pub use instr::{Instr, Opcode, Register};
