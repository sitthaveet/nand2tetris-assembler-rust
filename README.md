# Hack Assembler (Rust Implementation)

A complete assembler for the Hack computer platform from the [Nand2Tetris](https://www.nand2tetris.org/) course (Project 6). This implementation translates Hack assembly language (.asm) into binary machine code (.hack).

## Overview

The Hack Assembler is a two-pass assembler that converts symbolic Hack assembly code into 16-bit binary instructions. It handles three types of instructions:

- **A-instructions**: `@value` or `@symbol` - Address/constant loading
- **C-instructions**: `dest=comp;jump` - Computation and control flow
- **L-instructions**: `(LABEL)` - Symbolic labels for jump destinations

## Features

- **Two-pass assembly algorithm**
  - First pass: Discovers and records label addresses
  - Second pass: Generates machine code and resolves symbols
- **Symbol table management**
  - 23 predefined symbols (R0-R15, SP, LCL, ARG, THIS, THAT, SCREEN, KBD)
  - Label symbols for jump destinations
  - Variable symbols allocated from RAM address 16
- **Complete instruction support**
  - All 28 computation operations
  - All 8 destination combinations
  - All 8 jump conditions
- **Clean modular architecture** with separation of concerns

## Project Structure

```
nand2tetris-assembler-rust/
├── src/
│   ├── main.rs           # Main assembler orchestration (two-pass algorithm)
│   ├── parser.rs         # Instruction parsing and analysis
│   ├── symbol_table.rs   # Symbol-to-address mapping
│   └── code.rs           # Binary code generation
├── input/
│   └── program.asm       # Input assembly file
├── output/
│   └── program.hack      # Output binary file
├── Cargo.toml
├── CLAUDE.md             # Detailed implementation guide
└── README.md
```

## Module Architecture

### Parser Module (`parser.rs`)
Reads and parses Hack assembly instructions.

**Key responsibilities:**
- File reading and preprocessing (comment/whitespace removal)
- Instruction type identification (A/C/L)
- Field extraction (symbol, dest, comp, jump)

**Main API:**
```rust
pub struct Parser { /* ... */ }

impl Parser {
    pub fn new(filename: &str) -> io::Result<Self>
    pub fn advance(&mut self) -> bool
    pub fn instruction_type(&self) -> InstructionType
    pub fn symbol(&self) -> &str
    pub fn dest(&self) -> Option<&str>
    pub fn comp(&self) -> Option<&str>
    pub fn jump(&self) -> Option<&str>
}
```

### SymbolTable Module (`symbol_table.rs`)
Manages symbolic names and their memory addresses.

**Key responsibilities:**
- Initialization with predefined symbols
- Label address registration (first pass)
- Variable address allocation (second pass)
- Symbol lookup

**Main API:**
```rust
pub struct SymbolTable { /* ... */ }

impl SymbolTable {
    pub fn new() -> Self
    pub fn add_entry(&mut self, symbol: String, address: u16)
    pub fn contains(&self, symbol: &str) -> bool
    pub fn get_address(&self, symbol: &str) -> Option<u16>
}
```

### Code Module (`code.rs`)
Translates mnemonics into binary codes.

**Key responsibilities:**
- Destination code generation (3 bits)
- Computation code generation (7 bits: a + cccccc)
- Jump condition code generation (3 bits)

**Main API:**
```rust
pub fn dest(mnemonic: &str) -> &str  // Returns 3-bit binary
pub fn comp(mnemonic: &str) -> &str  // Returns 7-bit binary
pub fn jump(mnemonic: &str) -> &str  // Returns 3-bit binary
```

## Usage

### Building the Project

```bash
cargo build --release
```

### Running the Assembler

Place your assembly file in the `input/` directory as `program.asm`, then run:

```bash
cargo run
```

The assembled binary will be written to `output/program.hack`.

### Example

**Input** (`input/program.asm`):
```asm
// Computes R2 = max(R0, R1)
   @R0
   D=M
   @R1
   D=D-M
   @ITSR0
   D;JGT
   @R1
   D=M
   @R2
   M=D
   @END
   0;JMP
(ITSR0)
   @R0
   D=M
   @R2
   M=D
(END)
   @END
   0;JMP
```

**Output** (`output/program.hack`):
```
0000000000000000
1111110000010000
0000000000000001
1111010011010000
0000000000001100
1110001100000001
0000000000000001
1111110000010000
0000000000000010
1110001100001000
0000000000010000
1110101010000111
0000000000000000
1111110000010000
0000000000000010
1110001100001000
0000000000010000
1110101010000111
```

## Binary Encoding Reference

### C-Instruction Format
```
111a cccccc ddd jjj
│││ │       │   └─── jump (3 bits)
│││ │       └─────── dest (3 bits)
│││ └─────────────── comp (6 bits)
││└───────────────── a-bit (0=A register, 1=M register)
└└────────────────── C-instruction opcode
```

### Instruction Examples

| Assembly | Binary | Explanation |
|----------|--------|-------------|
| `@17` | `0000000000010001` | Load constant 17 into A register |
| `D=A` | `1110110000010000` | Copy A to D (dest=D, comp=A) |
| `D;JGT` | `1110001100000001` | Jump if D > 0 (comp=D, jump=JGT) |
| `M=D+1` | `1110011111001000` | Store D+1 in memory (dest=M, comp=D+1) |

## Known Issues

Per the Nand2Tetris specification, the assembler treats these destination codes as equivalent:
- `DM` and `MD` both map to binary `011`
- `ADM` and `AMD` both map to binary `111`

This is a known quirk of the Hack specification and is handled correctly by the code module.

## Implementation Details

### Two-Pass Algorithm

**First Pass:**
1. Initialize symbol table with 23 predefined symbols
2. Read through source file
3. Track ROM address (instruction counter)
4. When encountering `(LABEL)`, add to symbol table with current ROM address
5. Labels don't increment ROM address (they're pseudo-instructions)

**Second Pass:**
1. Read through source file again
2. For each instruction:
   - **A-instruction with constant**: Convert number to 16-bit binary
   - **A-instruction with symbol**: Look up or allocate address, then convert to binary
   - **C-instruction**: Extract dest/comp/jump, translate each field, assemble into 16 bits
   - **L-instruction**: Skip (already processed)
3. Write binary output to .hack file

### Memory Allocation

- **ROM (Instructions)**: Addresses 0-32767
  - Each instruction occupies one ROM address
  - Labels mark ROM addresses but don't consume space
- **RAM (Data)**: Addresses 0-24576
  - 0-15: Predefined registers (R0-R15, also SP, LCL, ARG, THIS, THAT)
  - 16-16383: User variables (allocated on first use)
  - 16384-24575: Screen memory map
  - 24576: Keyboard register

## Testing

The assembler has been tested with the Nand2Tetris provided test programs including:
- Add.asm (simple arithmetic)
- Max.asm (branching with labels)
- Programs with variables and symbols

All outputs match the reference assembler exactly.

## Resources

- [Nand2Tetris Website](https://www.nand2tetris.org/)
- [Hack Assembly Language Specification](https://www.nand2tetris.org/_files/ugd/44046b_89a8e226476741a3b7c5204575b8a0b2.pdf)
- [Course Book](https://www.nand2tetris.org/book): "The Elements of Computing Systems" by Nisan and Schocken

## License

This is an educational project for the Nand2Tetris course.

## Author

Implementation in Rust as part of the Nand2Tetris course learning journey.
