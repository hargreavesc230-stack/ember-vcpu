# Ember vCPU ISA (Phase 1)

## Overview
Ember-vCPU is a simple 64-bit, fixed-width (32-bit) load/store ISA. All general-purpose
registers (GPRs) and the program counter (PC) are 64-bit. Instructions are 32-bit words
aligned to 4 bytes. Memory is byte-addressed and little-endian.

Key properties:
- 16 GPRs: r0..r15 (u64 each)
- r0 is hardwired to 0; writes are ignored
- PC is a separate u64 register; it normally advances by 4 each instruction
- Load/store only; arithmetic and logic operate on registers
- No global flags register; branches compare registers directly

## Registers and PC
- `r0` always reads as 0. Any write to `r0` is ignored.
- `r1`..`r15` are general-purpose registers.
- `PC` holds the address of the current instruction. Unless control flow changes,
  `PC = PC + 4` after each instruction.

## Memory model
- Byte-addressed memory with 64-bit addresses.
- Implementations may bound memory to a finite array, but addresses are specified
  as 64-bit values.
- Loads and stores operate on 64-bit values.
- Memory is little-endian (least significant byte at lowest address).
- Loads and stores require 8-byte alignment. Unaligned access is an
  implementation-defined trap.

## Immediates and addressing
- I-type immediates are signed 16-bit values (`imm16`), sign-extended to 64-bit.
- J-type immediates are signed 26-bit values (`imm26`), sign-extended and scaled by 4.
- Branch and jump targets are PC-relative:
  - `target = PC + 4 + (signext(imm) << 2)`

## Instruction semantics
Notation:
- `R[x]` is the 64-bit value of register `x`.
- `sext16` sign-extends a 16-bit immediate to 64-bit.
- `sext26` sign-extends a 26-bit immediate to 64-bit.
- `addr = R[base] + sext16(offset)` uses 64-bit signed arithmetic.

### Arithmetic and logic (R-type)
- `ADD rd, rs1, rs2`: `R[rd] = R[rs1] + R[rs2]`
- `SUB rd, rs1, rs2`: `R[rd] = R[rs1] - R[rs2]`
- `AND rd, rs1, rs2`: `R[rd] = R[rs1] & R[rs2]`
- `OR  rd, rs1, rs2`: `R[rd] = R[rs1] | R[rs2]`
- `XOR rd, rs1, rs2`: `R[rd] = R[rs1] ^ R[rs2]`
- `SHL rd, rs1, rs2`: `R[rd] = R[rs1] << (R[rs2] & 0x3f)`
- `SHR rd, rs1, rs2`: `R[rd] = R[rs1] >> (R[rs2] & 0x3f)` (logical)

### Immediate arithmetic (I-type)
- `ADDI rd, rs1, imm16`: `R[rd] = R[rs1] + sext16(imm16)`

### Memory (I-type)
- `LD rd, offset(rs1)`:
  - `addr = R[rs1] + sext16(offset)`
  - `R[rd] = load64_le(addr)`
- `ST rs, offset(rs1)`:
  - `addr = R[rs1] + sext16(offset)`
  - `store64_le(addr, R[rs])`

### Branches (I-type)
- `BEQ rs1, rs2, off`: if `R[rs1] == R[rs2]` then
  `PC = PC + 4 + (sext16(off) << 2)`
- `BNE rs1, rs2, off`: if `R[rs1] != R[rs2]` then
  `PC = PC + 4 + (sext16(off) << 2)`
- `BLT rs1, rs2, off`: if `(R[rs1] as i64) < (R[rs2] as i64)` then
  `PC = PC + 4 + (sext16(off) << 2)`
- `BGE rs1, rs2, off`: if `(R[rs1] as i64) >= (R[rs2] as i64)` then
  `PC = PC + 4 + (sext16(off) << 2)`

### Jumps (J-type)
- `JAL off`:
  - `R[r15] = PC + 4`
  - `PC = PC + 4 + (sext26(off) << 2)`
- `JMP off`:
  - `PC = PC + 4 + (sext26(off) << 2)`

### System
- `ECALL imm16`: invoke host syscall `imm16` (values 0..2 are defined).

Syscall ABI (minimal):
- `ECALL 0` (exit): `r1` = exit code
- `ECALL 1` (putc): `r1` low 8 bits = character
- `ECALL 2` (puts): `r1` = pointer, `r2` = length in bytes

Unless specified, registers are preserved across syscalls. `ECALL 0` terminates
execution.
