// SymbolTable module for Hack Assembly Language
// This module manages the symbol table that maps symbolic names to memory addresses
// It handles three types of symbols:
// 1. Predefined symbols (R0-R15, SP, LCL, ARG, THIS, THAT, SCREEN, KBD)
// 2. Label symbols (xxx) - mark instruction memory locations
// 3. Variable symbols @xxx - allocated to RAM starting at address 16

use std::collections::HashMap;

/// SymbolTable stores mappings from symbol names to memory addresses
pub struct SymbolTable {
    /// HashMap storing symbol name -> address mappings
    /// Using String as key (owned) and u16 for addresses (0-32767)
    table: HashMap<String, u16>,
}

impl SymbolTable {
    /// Creates a new SymbolTable initialized with all 23 predefined symbols
    ///
    /// Predefined symbols include:
    /// - R0-R15: Virtual registers (addresses 0-15)
    /// - SP, LCL, ARG, THIS, THAT: Pointer symbols (addresses 0-4)
    /// - SCREEN: Screen memory map base (address 16384)
    /// - KBD: Keyboard register (address 24576)
    ///
    /// # Returns
    /// * `SymbolTable` - A new symbol table with predefined symbols
    ///
    /// # Example
    /// ```
    /// let mut symbol_table = SymbolTable::new();
    /// assert!(symbol_table.contains("R0"));
    /// assert!(symbol_table.contains("SCREEN"));
    /// ```
    pub fn new() -> Self {
        // Create an empty HashMap
        let mut table = HashMap::new();

        // Add virtual registers R0-R15
        // We use a loop to avoid repetition
        for i in 0..=15 {
            // format! creates a String like "R0", "R1", etc.
            table.insert(format!("R{}", i), i);
        }

        // Add special pointer symbols
        // Note: SP=R0, LCL=R1, etc. (same addresses as registers)
        table.insert("SP".to_string(), 0);    // Stack Pointer
        table.insert("LCL".to_string(), 1);   // Local segment base
        table.insert("ARG".to_string(), 2);   // Argument segment base
        table.insert("THIS".to_string(), 3);  // This pointer
        table.insert("THAT".to_string(), 4);  // That pointer

        // Add I/O memory-mapped symbols
        table.insert("SCREEN".to_string(), 16384);  // Screen memory base (0x4000)
        table.insert("KBD".to_string(), 24576);     // Keyboard register (0x6000)

        // Return the SymbolTable with the initialized table
        SymbolTable { table }
    }

    /// Adds a new symbol-address pair to the symbol table
    ///
    /// # Arguments
    /// * `symbol` - The symbol name (e.g., "LOOP", "sum", "i")
    /// * `address` - The memory address to associate with the symbol
    ///
    /// # Example
    /// ```
    /// symbol_table.add_entry("LOOP".to_string(), 10);
    /// symbol_table.add_entry("sum".to_string(), 16);
    /// ```
    pub fn add_entry(&mut self, symbol: String, address: u16) {
        // Insert the symbol-address pair into the HashMap
        // If the symbol already exists, this will overwrite it
        self.table.insert(symbol, address);
    }

    /// Checks if a symbol exists in the symbol table
    ///
    /// # Arguments
    /// * `symbol` - The symbol name to look up
    ///
    /// # Returns
    /// * `bool` - true if the symbol exists, false otherwise
    ///
    /// # Example
    /// ```
    /// if symbol_table.contains("LOOP") {
    ///     println!("LOOP is defined");
    /// }
    /// ```
    pub fn contains(&self, symbol: &str) -> bool {
        // HashMap's contains_key() returns true if the key exists
        // We use &str here because we don't need to own the string
        self.table.contains_key(symbol)
    }

    /// Gets the address associated with a symbol
    ///
    /// # Arguments
    /// * `symbol` - The symbol name to look up
    ///
    /// # Returns
    /// * `Option<u16>` - Some(address) if found, None if not found
    ///
    /// # Example
    /// ```
    /// match symbol_table.get_address("R0") {
    ///     Some(addr) => println!("R0 is at address {}", addr),
    ///     None => println!("Symbol not found"),
    /// }
    /// ```
    pub fn get_address(&self, symbol: &str) -> Option<u16> {
        // HashMap's get() returns Option<&u16>
        // We use copied() to convert &u16 to u16
        // This is safe because u16 implements Copy trait
        self.table.get(symbol).copied()
    }

    /// Returns the number of symbols in the table
    ///
    /// Useful for debugging and testing
    ///
    /// # Returns
    /// * `usize` - The number of symbols currently in the table
    pub fn size(&self) -> usize {
        self.table.len()
    }
}
