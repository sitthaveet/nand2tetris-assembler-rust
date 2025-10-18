// Code module for Hack Assembly Language
// This module translates Hack assembly mnemonics into binary codes
//
// C-instruction format: 111a cccccc ddd jjj (16 bits)
// - Bits 15-13: 111 (opcode for C-instruction)
// - Bit 12 (a): 0 = use A register, 1 = use M (Memory[A])
// - Bits 11-6 (c1-c6): Computation bits
// - Bits 5-3 (d1-d3): Destination bits
// - Bits 2-0 (j1-j2-j3): Jump bits
//
// This module provides three functions:
// - dest(mnemonic) -> 3-bit binary string
// - comp(mnemonic) -> 7-bit binary string (includes 'a' bit)
// - jump(mnemonic) -> 3-bit binary string

/// Translates a dest mnemonic to its 3-bit binary code
///
/// The dest field specifies where to store the ALU output.
/// Format: d1 d2 d3 where d1=A, d2=D, d3=M
///
/// # Arguments
/// * `mnemonic` - The destination mnemonic (e.g., "D", "M", "AMD")
///
/// # Returns
/// * `&str` - A 3-bit binary string
///
/// # Panics
/// Panics if given an unknown dest mnemonic
///
/// # Example
/// ```
/// assert_eq!(code::dest("D"), "010");
/// assert_eq!(code::dest("AMD"), "111");
/// assert_eq!(code::dest(""), "000");  // null destination
/// ```
pub fn dest(mnemonic: &str) -> &str {
    // Use match expression to map mnemonic to binary code
    // The dest bits are: d1(A) d2(D) d3(M)
    // For example, "AMD" means store in all three: A=1, D=1, M=1 → "111"

    match mnemonic {
        ""    => "000",  // null - don't store anywhere
        "M"   => "001",  // Store in M only:   A=0, D=0, M=1
        "D"   => "010",  // Store in D only:   A=0, D=1, M=0
        "MD"  => "011",  // Store in M and D:  A=0, D=1, M=1
        "A"   => "100",  // Store in A only:   A=1, D=0, M=0
        "AM"  => "101",  // Store in A and M:  A=1, D=0, M=1
        "AD"  => "110",  // Store in A and D:  A=1, D=1, M=0
        "AMD" => "111",  // Store in all:      A=1, D=1, M=1

        // Handle the known bug: DM and MD are the same, ADM and AMD are the same
        "DM"  => "011",  // Same as MD
        "ADM" => "111",  // Same as AMD

        // If we get an unknown mnemonic, panic with error message
        _ => panic!("Unknown dest mnemonic: '{}'", mnemonic),
    }
}

/// Translates a comp mnemonic to its 7-bit binary code
///
/// The comp field specifies what computation to perform.
/// Format: a c1 c2 c3 c4 c5 c6
/// - a=0 means use A register
/// - a=1 means use M (Memory[A])
///
/// # Arguments
/// * `mnemonic` - The computation mnemonic (e.g., "D+A", "M-1", "!D")
///
/// # Returns
/// * `&str` - A 7-bit binary string (a + c1-c6)
///
/// # Panics
/// Panics if given an unknown comp mnemonic
///
/// # Example
/// ```
/// assert_eq!(code::comp("0"), "0101010");
/// assert_eq!(code::comp("D+A"), "0000010");
/// assert_eq!(code::comp("D+M"), "1000010");  // Note: a=1 for M operations
/// ```
pub fn comp(mnemonic: &str) -> &str {
    // Use match expression to map comp mnemonic to binary code
    // Format: a c1 c2 c3 c4 c5 c6 (7 bits total)
    // When a=0, operations use A register
    // When a=1, operations use M (Memory[A])

    match mnemonic {
        // Constants (a=0)
        "0"   => "0101010",  // Zero
        "1"   => "0111111",  // One
        "-1"  => "0111010",  // Negative one

        // Single operands with A register (a=0)
        "D"   => "0001100",  // D register value
        "A"   => "0110000",  // A register value
        "!D"  => "0001101",  // NOT D (bitwise)
        "!A"  => "0110001",  // NOT A (bitwise)
        "-D"  => "0001111",  // Negate D (two's complement)
        "-A"  => "0110011",  // Negate A (two's complement)

        // Increment/Decrement with A (a=0)
        "D+1" => "0011111",  // D plus 1
        "A+1" => "0110111",  // A plus 1
        "D-1" => "0001110",  // D minus 1
        "A-1" => "0110010",  // A minus 1

        // Two operand ALU operations with A (a=0)
        "D+A" => "0000010",  // D plus A
        "D-A" => "0010011",  // D minus A
        "A-D" => "0000111",  // A minus D
        "D&A" => "0000000",  // D AND A (bitwise)
        "D|A" => "0010101",  // D OR A (bitwise)

        // Single operands with M register (a=1)
        "M"   => "1110000",  // Memory[A] value
        "!M"  => "1110001",  // NOT M (bitwise)
        "-M"  => "1110011",  // Negate M (two's complement)

        // Increment/Decrement with M (a=1)
        "M+1" => "1110111",  // M plus 1
        "M-1" => "1110010",  // M minus 1

        // Two operand ALU operations with M (a=1)
        "D+M" => "1000010",  // D plus M
        "D-M" => "1010011",  // D minus M
        "M-D" => "1000111",  // M minus D
        "D&M" => "1000000",  // D AND M (bitwise)
        "D|M" => "1010101",  // D OR M (bitwise)

        // If we get an unknown mnemonic, panic with error message
        _ => panic!("Unknown comp mnemonic: '{}'", mnemonic),
    }
}

/// Translates a jump mnemonic to its 3-bit binary code
///
/// The jump field specifies the jump condition based on ALU output.
/// Format: j1 j2 j3
///
/// # Arguments
/// * `mnemonic` - The jump mnemonic (e.g., "JGT", "JEQ", "JMP")
///
/// # Returns
/// * `&str` - A 3-bit binary string
///
/// # Panics
/// Panics if given an unknown jump mnemonic
///
/// # Example
/// ```
/// assert_eq!(code::jump(""), "000");     // null (no jump)
/// assert_eq!(code::jump("JGT"), "001");
/// assert_eq!(code::jump("JMP"), "111");  // unconditional jump
/// ```
pub fn jump(mnemonic: &str) -> &str {
    // Use match expression to map jump mnemonic to binary code
    // The jump is based on comparing the ALU output to zero
    // Format: j1 j2 j3 where:
    //   j1 = jump if out < 0 (negative)
    //   j2 = jump if out = 0 (zero)
    //   j3 = jump if out > 0 (positive)

    match mnemonic {
        ""    => "000",  // null - no jump (continue to next instruction)
        "JGT" => "001",  // Jump if Greater Than:     out > 0  (j3=1)
        "JEQ" => "010",  // Jump if EQual:            out = 0  (j2=1)
        "JGE" => "011",  // Jump if Greater or Equal: out >= 0 (j2=1, j3=1)
        "JLT" => "100",  // Jump if Less Than:        out < 0  (j1=1)
        "JNE" => "101",  // Jump if Not Equal:        out != 0 (j1=1, j3=1)
        "JLE" => "110",  // Jump if Less or Equal:    out <= 0 (j1=1, j2=1)
        "JMP" => "111",  // Jump (unconditional):     always   (all=1)

        // If we get an unknown mnemonic, panic with error message
        _ => panic!("Unknown jump mnemonic: '{}'", mnemonic),
    }
}
