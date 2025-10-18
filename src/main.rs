// Hack Assembler - Main Driver
// This is the main program that orchestrates the two-pass assembly process
// It reads a .asm file and produces a .hack binary file

mod parser;
mod symbol_table;
mod code;

use parser::{Parser, InstructionType};
use symbol_table::SymbolTable;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║   Hack Assembler                       ║");
    println!("╚════════════════════════════════════════╝\n");

    // Define input and output files
    let input_file = "input/program.asm";
    let output_file = "output/program.hack";

    // Run the assembler
    match assemble(input_file, output_file) {
        Ok(()) => {
            println!("\n✓ Assembly completed successfully!");
            println!("  Input:  {}", input_file);
            println!("  Output: {}", output_file);
        }
        Err(e) => {
            eprintln!("\n✗ Assembly failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Main assembly function that orchestrates the two-pass process
///
/// # Arguments
/// * `input_file` - Path to the .asm source file
/// * `output_file` - Path to the .hack output file
///
/// # Returns
/// * `Result<(), String>` - Ok if successful, Err with error message otherwise
fn assemble(input_file: &str, output_file: &str) -> Result<(), String> {
    println!("Starting assembly process...\n");

    // Step 1: Initialize symbol table with predefined symbols
    println!("Step 1: Initializing symbol table...");
    let mut symbol_table = SymbolTable::new();
    println!("  ✓ Symbol table initialized with {} predefined symbols\n",
             symbol_table.size());

    // Step 2: First pass - add labels to symbol table
    println!("Step 2: First pass (scanning for labels)...");
    first_pass(input_file, &mut symbol_table)?;
    println!("  ✓ First pass complete\n");

    // Step 3: Second pass - generate machine code
    println!("Step 3: Second pass (generating machine code)...");
    second_pass(input_file, output_file, &mut symbol_table)?;
    println!("  ✓ Second pass complete\n");

    Ok(())
}

/// First pass: Scan the file and add all label declarations to the symbol table
///
/// This pass uses the Parser module to:
/// - Read through the entire file
/// - Count actual instructions (A and C instructions)
/// - When it finds a label (LABEL), adds it to symbol table with current ROM address
/// - Labels don't count as instructions (they're just markers)
///
/// # Arguments
/// * `filename` - Path to the .asm file
/// * `symbol_table` - Mutable reference to the symbol table
///
/// # Returns
/// * `Result<(), String>` - Ok if successful, Err with error message otherwise
fn first_pass(filename: &str, symbol_table: &mut SymbolTable) -> Result<(), String> {
    // Create a Parser to read the file
    let mut parser = Parser::new(filename)
        .map_err(|e| format!("Failed to open file '{}': {}", filename, e))?;

    let mut rom_address: u16 = 0;  // Current ROM address (instruction counter)
    let mut label_count = 0;       // Counter for labels found

    // Advance through all instructions
    while parser.advance() {
        // Check the instruction type using Parser's method
        match parser.instruction_type() {
            InstructionType::L => {
                // L-instruction: (LABEL)
                // Use Parser's symbol() method to extract label name
                let label = parser.symbol();

                // Add to symbol table with current ROM address
                symbol_table.add_entry(label.to_string(), rom_address);
                label_count += 1;
                println!("    Found label: {} -> {}", label, rom_address);

                // Important: Do NOT increment rom_address!
                // Labels are pseudo-instructions, they don't take up ROM space
            }
            InstructionType::A | InstructionType::C => {
                // A-instruction or C-instruction
                // These are actual instructions, so they take up ROM space
                rom_address += 1;
            }
        }
    }

    println!("    Total labels found: {}", label_count);
    println!("    Total instructions: {}", rom_address);

    Ok(())
}

/// Second pass: Generate machine code for all instructions
///
/// This pass uses the Parser module to:
/// - Read through the entire file again
/// - Translate each instruction to binary
/// - For A-instructions with symbols:
///   - If symbol exists in table, use its address
///   - If not, it's a new variable - allocate next RAM address (starting at 16)
/// - For C-instructions, translate using the code module
/// - Writes binary output to .hack file
///
/// # Arguments
/// * `input_filename` - Path to the .asm file
/// * `output_filename` - Path to the .hack file to create
/// * `symbol_table` - Mutable reference to the symbol table
///
/// # Returns
/// * `Result<(), String>` - Ok if successful, Err with error message otherwise
fn second_pass(
    input_filename: &str,
    output_filename: &str,
    symbol_table: &mut SymbolTable
) -> Result<(), String> {
    // Create a Parser to read the file (second time)
    let mut parser = Parser::new(input_filename)
        .map_err(|e| format!("Failed to open input file '{}': {}", input_filename, e))?;

    let mut next_var_address: u16 = 16;  // Variables start at RAM address 16
    let mut variable_count = 0;          // Counter for new variables allocated
    let mut binary_instructions: Vec<String> = Vec::new();  // Collect all binary instructions

    // Advance through all instructions
    while parser.advance() {
        // Determine instruction type and generate binary
        let binary = match parser.instruction_type() {
            InstructionType::A => {
                // A-instruction: @value or @symbol
                // Use Parser's symbol() method to extract the symbol/value
                let symbol_or_value = parser.symbol();

                // Try to parse as a number first
                let address = if let Ok(num) = symbol_or_value.parse::<u16>() {
                    // It's a numeric constant (e.g., @123)
                    num
                } else {
                    // It's a symbol (e.g., @sum, @LOOP, @R0)
                    if !symbol_table.contains(symbol_or_value) {
                        // New variable - allocate next available RAM address
                        symbol_table.add_entry(
                            symbol_or_value.to_string(),
                            next_var_address
                        );
                        println!("    Allocated variable: {} -> {}",
                                 symbol_or_value, next_var_address);
                        variable_count += 1;

                        let addr = next_var_address;
                        next_var_address += 1;
                        addr
                    } else {
                        // Look up address from symbol table
                        symbol_table.get_address(symbol_or_value)
                            .ok_or_else(|| format!("Symbol not found: {}", symbol_or_value))?
                    }
                };

                // Convert to 16-bit binary string
                // A-instructions always start with 0
                Some(format!("{:016b}", address))
            }
            InstructionType::L => {
                // L-instruction (label) - skip, already processed in Pass 1
                None
            }
            InstructionType::C => {
                // C-instruction: dest=comp;jump
                // Use Parser's methods to extract each field
                let dest = parser.dest().unwrap_or("");  // "" if no dest
                let comp = parser.comp().expect("C-instruction must have comp field");
                let jump = parser.jump().unwrap_or("");  // "" if no jump

                // Translate each field to binary using the code module
                let dest_bits = code::dest(dest);
                let comp_bits = code::comp(comp);
                let jump_bits = code::jump(jump);

                // Assemble: 111 + comp (7 bits) + dest (3 bits) + jump (3 bits)
                // Total: 16 bits
                Some(format!("111{}{}{}", comp_bits, dest_bits, jump_bits))
            }
        };

        // Collect binary instruction (if we generated any)
        if let Some(binary_str) = binary {
            binary_instructions.push(binary_str);
        }
    }

    // Write all instructions to output file with proper formatting
    let mut output = File::create(output_filename)
        .map_err(|e| format!("Failed to create output file '{}': {}", output_filename, e))?;

    // Join all instructions with newlines, no trailing newline
    let output_content = binary_instructions.join("\n");
    write!(output, "{}", output_content)
        .map_err(|e| format!("Failed to write to output file: {}", e))?;

    println!("    Total instructions written: {}", binary_instructions.len());
    println!("    New variables allocated: {}", variable_count);

    Ok(())
}
