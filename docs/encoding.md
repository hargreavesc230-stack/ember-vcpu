# Ember vCPU Instruction Encoding

All instructions are 32-bit words and aligned to 4 bytes. Fields are encoded in
little-endian memory, but bit numbering in this document uses the conventional
most-significant-bit (MSB) numbering for a 32-bit word.

## Bit layouts

### R-type
```
31           26 25 22 21 18 17 14 13                       0
+--------------+-----+-----+-----+-------------------------+
|   opcode     | rd  | rs1 | rs2 |        funct14          |
+--------------+-----+-----+-----+-------------------------+
```

### I-type
```
31           26 25 22 21 18 17 14 15                       0
+--------------+-----+-----+-----+-------------------------+
|   opcode     | rd  | rs1 | rs2 |          imm16          |
+--------------+-----+-----+-----+-------------------------+
```

Field usage:
- `rd` is the destination for ADDI/LD, and the source register for ST.
- `rs1` is the base register for ADDI/LD/ST, or the first compare register for branches.
- `rs2` is the second compare register for branches.
- For ADDI/LD/ST/ECALL, `rs2` must be 0 (reserved for future sub-ops).
- For ECALL, `rd` and `rs1` must be 0.
- For branches, `rd` must be 0 (reserved).

### J-type
```
31           26 25                                           0
+--------------+---------------------------------------------+
|   opcode     |                  imm26                      |
+--------------+---------------------------------------------+
```

## Opcode table
Opcodes are 6-bit values in bits [31:26]. Any opcode not listed below is reserved.

```
0x00  R-type (funct14 selects operation)
0x01  ADDI
0x02  LD
0x03  ST
0x04  BEQ
0x05  BNE
0x06  BLT
0x07  BGE
0x08  JAL
0x09  JMP
0x0A  ECALL
```

## R-type funct14 table
`funct14` selects the R-type operation. Any value not listed below is reserved.

```
0x0001  ADD
0x0002  SUB
0x0003  AND
0x0004  OR
0x0005  XOR
0x0006  SHL
0x0007  SHR
```

## Immediate interpretation
- `imm16` is a signed 16-bit value, sign-extended to 64-bit.
- `imm26` is a signed 26-bit value, sign-extended to 64-bit.
- Branch and jump targets use a scaled PC-relative offset:
  - `target = PC + 4 + (signext(imm) << 2)`

## Reserved encodings
- Any opcode or R-type `funct14` not listed is reserved.
- For ADDI/LD/ST/ECALL, `rs2` must be 0. Non-zero values are reserved.
- For ECALL, `rd` and `rs1` must be 0. Non-zero values are reserved.
- For branch opcodes, `rd` must be 0. Non-zero values are reserved.

## Examples
- Branch with `imm16 = 0xFFFE (-2)`:
  - `target = PC + 4 + ((-2) << 2) = PC - 4`
- Jump with `imm26 = 0x000001 (1)`:
  - `target = PC + 4 + (1 << 2) = PC + 8`
