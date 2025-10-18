// Parser module for Hack Assembly Language
// This module reads and parses .asm files, identifying instruction types
// and extracting components from each instruction

use std::fs::File;
use std::io::{BufRead, BufReader, Result as IoResult};

/// The three types of instructions in Hack assembly language
#[derive(Debug, PartialEq, Clone)]
pub enum InstructionType {
    /// A-instruction: @value or @symbol
    /// Examples: @2, @sum, @LOOP
    A,

    /// C-instruction: dest=comp;jump (dest and jump are optional)
    /// Examples: D=A, D=D+1, D;JGT, M=D+1;JLE
    C,

    /// L-instruction (Label/pseudo-instruction): (LABEL)
    /// Examples: (LOOP), (END)
    /// Note: Labels don't generate machine code, they mark addresses
    L,
}

/// Parser reads assembly code and provides methods to parse individual instructions
pub struct Parser {
    /// All non-empty, non-comment lines from the file
    lines: Vec<String>,

    /// Current position in the lines vector
    current_line: usize,

    /// The current instruction being processed (cleaned, no whitespace/comments)
    current_instruction: Option<String>,
}

impl Parser {
    /// Creates a new Parser by reading and cleaning all lines from the given file
    ///
    /// # Arguments
    /// * `filename` - Path to the .asm file to parse
    ///
    /// # Returns
    /// * `IoResult<Parser>` - A Result containing the Parser or an IO error
    ///
    /// # Example
    /// ```
    /// let parser = Parser::new("Add.asm")?;
    /// ```
    pub fn new(filename: &str) -> IoResult<Self> {
        // Open the file
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        // Read and clean all lines
        let mut cleaned_lines = Vec::new();

        for line in reader.lines() {
            let line = line?; // Unwrap the Result, propagate errors with ?

            // Clean the line (remove comments and whitespace)
            if let Some(cleaned) = Self::clean_line(&line) {
                cleaned_lines.push(cleaned);
            }
        }

        Ok(Parser {
            lines: cleaned_lines,
            current_line: 0,
            current_instruction: None,
        })
    }

    /// Cleans a line by removing comments and whitespace
    ///
    /// # Arguments
    /// * `line` - The raw line from the file
    ///
    /// # Returns
    /// * `Option<String>` - Some(cleaned_line) if there's content, None if line is empty/comment-only
    ///
    /// # Example
    /// ```
    /// // "  @2  // Load 2  " becomes Some("@2")
    /// // "  // Just a comment  " becomes None
    /// // "   " becomes None
    /// ```
    fn clean_line(line: &str) -> Option<String> {
        // Find the position of "//" comment marker
        let without_comment = match line.split_once("//") {
            Some((code, _comment)) => code, // Take the part before //
            None => line,                    // No comment, use whole line
        };

        // Remove all whitespace (spaces and tabs)
        let trimmed = without_comment.trim();

        // Return None if empty, otherwise return the cleaned string
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Checks if there are more instructions to process
    ///
    /// # Returns
    /// * `bool` - true if there are more lines, false if we've reached the end
    ///
    /// # Example
    /// ```
    /// while parser.has_more_lines() {
    ///     parser.advance();
    ///     // process instruction
    /// }
    /// ```
    pub fn has_more_lines(&self) -> bool {
        self.current_line < self.lines.len()
    }

    /// Advances to the next instruction and makes it the current instruction
    ///
    /// # Returns
    /// * `bool` - true if successfully advanced, false if no more lines
    ///
    /// # Example
    /// ```
    /// if parser.advance() {
    ///     let instruction_type = parser.instruction_type();
    /// }
    /// ```
    pub fn advance(&mut self) -> bool {
        if self.has_more_lines() {
            // Get the next line and store it as current instruction
            self.current_instruction = Some(self.lines[self.current_line].clone());
            self.current_line += 1;
            true
        } else {
            // No more lines
            self.current_instruction = None;
            false
        }
    }

    /// Returns the type of the current instruction
    ///
    /// # Returns
    /// * `InstructionType` - A, C, or L
    ///
    /// # Panics
    /// Panics if called before advance() or after all lines are consumed
    ///
    /// # Example
    /// ```
    /// parser.advance();
    /// match parser.instruction_type() {
    ///     InstructionType::A => println!("A-instruction"),
    ///     InstructionType::C => println!("C-instruction"),
    ///     InstructionType::L => println!("Label"),
    /// }
    /// ```
    pub fn instruction_type(&self) -> InstructionType {
        let instruction = self.current_instruction
            .as_ref()
            .expect("No current instruction - call advance() first");

        // Check the first character to determine type
        match instruction.chars().next() {
            Some('@') => InstructionType::A,  // @value or @symbol
            Some('(') => InstructionType::L,  // (LABEL)
            _ => InstructionType::C,          // Everything else is C-instruction
        }
    }

    /// Returns the symbol or decimal value from an A-instruction or L-instruction
    ///
    /// For A-instructions: @xxx returns "xxx"
    /// For L-instructions: (xxx) returns "xxx"
    ///
    /// # Returns
    /// * `&str` - The symbol or value as a string slice
    ///
    /// # Panics
    /// Panics if called on a C-instruction
    ///
    /// # Example
    /// ```
    /// // For "@sum" returns "sum"
    /// // For "@123" returns "123"
    /// // For "(LOOP)" returns "LOOP"
    /// let symbol = parser.symbol();
    /// ```
    pub fn symbol(&self) -> &str {
        let instruction = self.current_instruction
            .as_ref()
            .expect("No current instruction");

        match self.instruction_type() {
            InstructionType::A => {
                // A-instruction: @xxx
                // Remove the '@' character (skip first character)
                &instruction[1..]
            }
            InstructionType::L => {
                // L-instruction: (xxx)
                // Remove the '(' and ')' characters
                &instruction[1..instruction.len() - 1]
            }
            InstructionType::C => {
                panic!("symbol() should not be called on C-instructions");
            }
        }
    }

    /// Returns the dest mnemonic from a C-instruction
    ///
    /// C-instruction format: dest=comp;jump
    /// This extracts the "dest" part
    ///
    /// # Returns
    /// * `Option<&str>` - Some(dest) if present, None if no destination
    ///
    /// # Example
    /// ```
    /// // "D=A" returns Some("D")
    /// // "M=D+1" returns Some("M")
    /// // "D+1;JGT" returns None (no =, so no destination)
    /// ```
    pub fn dest(&self) -> Option<&str> {
        let instruction = self.current_instruction
            .as_ref()
            .expect("No current instruction");

        // Only C-instructions have dest
        if self.instruction_type() != InstructionType::C {
            return None;
        }

        // Check if there's an '=' sign
        // If yes, everything before '=' is the destination
        match instruction.split_once('=') {
            Some((dest, _rest)) => Some(dest),
            None => None, // No '=' means no destination
        }
    }

    /// Returns the comp mnemonic from a C-instruction
    ///
    /// C-instruction format: dest=comp;jump
    /// This extracts the "comp" part
    ///
    /// # Returns
    /// * `Option<&str>` - Some(comp) if C-instruction, None otherwise
    ///
    /// # Example
    /// ```
    /// // "D=A" returns Some("A")
    /// // "D=D+1" returns Some("D+1")
    /// // "D+1;JGT" returns Some("D+1")
    /// // "0;JMP" returns Some("0")
    /// ```
    pub fn comp(&self) -> Option<&str> {
        let instruction = self.current_instruction
            .as_ref()
            .expect("No current instruction");

        // Only C-instructions have comp
        if self.instruction_type() != InstructionType::C {
            return None;
        }

        // First, split by ';' to separate comp from jump
        // Then, split by '=' to separate dest from comp

        // Step 1: Get the part before ';' (this contains dest=comp or just comp)
        let before_jump = match instruction.split_once(';') {
            Some((before, _jump)) => before,
            None => instruction.as_str(), // No jump, use whole instruction
        };

        // Step 2: Get the part after '=' (this is comp), or use the whole thing
        let comp = match before_jump.split_once('=') {
            Some((_dest, comp)) => comp,
            None => before_jump, // No dest, so it's just comp
        };

        Some(comp)
    }

    /// Returns the jump mnemonic from a C-instruction
    ///
    /// C-instruction format: dest=comp;jump
    /// This extracts the "jump" part
    ///
    /// # Returns
    /// * `Option<&str>` - Some(jump) if present, None if no jump condition
    ///
    /// # Example
    /// ```
    /// // "D=D+1;JLE" returns Some("JLE")
    /// // "D;JGT" returns Some("JGT")
    /// // "M=D" returns None (no jump)
    /// ```
    pub fn jump(&self) -> Option<&str> {
        let instruction = self.current_instruction
            .as_ref()
            .expect("No current instruction");

        // Only C-instructions have jump
        if self.instruction_type() != InstructionType::C {
            return None;
        }

        // Check if there's a ';' sign
        // If yes, everything after ';' is the jump condition
        match instruction.split_once(';') {
            Some((_rest, jump)) => Some(jump),
            None => None, // No ';' means no jump
        }
    }
}
