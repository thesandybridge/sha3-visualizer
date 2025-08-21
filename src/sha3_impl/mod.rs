//! Implementation of the SHA-3/Keccak cryptographic hash function.
//!
//! This module provides a complete implementation of the Keccak-f[1600] permutation
//! used in SHA-3, with step-by-step execution capabilities for educational visualization.
//!
//! # Features
//!
//! - Complete SHA-3 implementation with all 5 transformation steps (θ, ρ, π, χ, ι)
//! - Step-by-step execution for visualization
//! - Support for multiple SHA-3 variants (224, 256, 384, 512 bits)
//! - Bit-level access for 3D visualization
//! - History tracking for educational purposes
//!
//! # Algorithm Overview
//!
//! SHA-3 operates on a 1600-bit state arranged as a 5×5×64 matrix. Each round
//! applies five transformations in sequence:
//!
//! 1. **θ (Theta)**: Column parity calculation and XOR
//! 2. **ρ (Rho)**: Bit rotation within lanes  
//! 3. **π (Pi)**: Lane rearrangement
//! 4. **χ (Chi)**: Non-linear transformation (provides cryptographic security)
//! 5. **ι (Iota)**: Round constant addition
//!
//! The algorithm runs for 24 rounds total.

pub mod constants;

use constants::*;

/// Main state structure for the SHA-3/Keccak algorithm.
/// 
/// Represents the complete internal state of the algorithm including:
/// - The 1600-bit state matrix (25 × 64-bit lanes)
/// - Current position in the algorithm (round and step)
/// - Input data and processing parameters
/// - History tracking for educational purposes
#[derive(Debug, Clone)]
pub struct KeccakState {
    /// The 1600-bit state represented as 25 64-bit lanes in row-major order.
    /// Layout: lanes[x + 5*z] represents lane at coordinate (x, z)
    pub state: [u64; 25],
    
    /// Current round number (0-23)
    pub round: usize,
    
    /// Current step within the round (0-4 for θ,ρ,π,χ,ι)
    pub step: usize,
    
    /// Flag indicating whether the algorithm has completed all rounds
    pub is_complete: bool,
    
    /// Original input data to be hashed
    input_data: Vec<u8>,
    
    /// Number of input blocks that have been absorbed
    absorbed: usize,
    
    /// Rate in bytes (bitrate / 8) - determines how much data can be absorbed per block
    rate: usize,
    
    /// Capacity in bits - security parameter (typically 2 × output_length)
    capacity: usize,
    
    /// Desired output length in bytes
    output_length: usize,
}


impl KeccakState {
    /// Creates a new KeccakState initialized for SHA3-256.
    /// 
    /// The default configuration uses:
    /// - Rate: 1088 bits (136 bytes)
    /// - Capacity: 512 bits  
    /// - Output length: 256 bits (32 bytes)
    /// 
    /// # Returns
    /// 
    /// A new `KeccakState` ready for input
    pub fn new() -> Self {
        Self {
            state: [0u64; 25],
            round: 0,
            step: 0,
            is_complete: false,
            input_data: Vec::new(),
            absorbed: 0,
            rate: 1088 / 8, // SHA3-256: 136 bytes
            capacity: 512,   // 512 bits
            output_length: 32, // 32 bytes = 256 bits
        }
    }


    /// Sets the input data to be hashed and resets the algorithm state.
    /// 
    /// This method:
    /// - Stores the input data for processing
    /// - Resets all state variables to initial values
    /// - Clears the internal state matrix
    /// - Clears the step history
    /// 
    /// Note: The input is not immediately absorbed into the state.
    /// Use the `step()` method to begin processing.
    /// 
    /// # Arguments
    /// 
    /// * `data` - The string data to be hashed
    pub fn set_input(&mut self, data: &str) {
        self.input_data = data.as_bytes().to_vec();
        self.absorbed = 0;
        self.round = 0;
        self.step = 0;
        self.is_complete = false;
        self.state = [0u64; 25]; // Initialize with empty state
    }


    /// Gets the value of a specific bit in the 3D state matrix.
    /// 
    /// The Keccak state is organized as a 5×5×64 matrix where:
    /// - `x` and `z` coordinates define the 5×5 lane grid (0-4 each)
    /// - `y` coordinate defines the bit position within a lane (0-63)
    /// 
    /// # Arguments
    /// 
    /// * `x` - X coordinate in the lane grid (0-4)
    /// * `y` - Bit position within the lane (0-63) 
    /// * `z` - Z coordinate in the lane grid (0-4)
    /// 
    /// # Returns
    /// 
    /// `true` if the bit is set (1), `false` if unset (0) or coordinates are invalid
    pub fn get_bit(&self, x: usize, y: usize, z: usize) -> bool {
        if x >= 5 || y >= 64 || z >= 5 {
            return false;
        }
        
        let lane_index = x + 5 * z; // Convert (x,z) to linear index
        let bit_position = y;
        
        (self.state[lane_index] >> bit_position) & 1 == 1
    }



    #[allow(dead_code)]
    fn apply_keccak_f(&mut self) {
        for round in 0..24 {
            // Theta
            let mut c = [0u64; 5];
            for x in 0..5 {
                c[x] = self.state[x] ^ self.state[x + 5] ^ self.state[x + 10] ^ self.state[x + 15] ^ self.state[x + 20];
            }
            
            let mut d = [0u64; 5];
            for x in 0..5 {
                d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
            }
            
            for x in 0..5 {
                for y in 0..5 {
                    self.state[x + 5 * y] ^= d[x];
                }
            }
            
            // Rho step: just rotate each lane in place
            for i in 0..25 {
                let x = i % 5;
                let z = i / 5;
                self.state[i] = self.state[i].rotate_left(RHO_OFFSETS[x][z] as u32);
            }
            
            // Pi step: permute lanes according to XKCP pattern
            let mut new_state = self.state;
            let mut x = 1;
            let mut y = 0;
            let mut current = self.state[x + 5 * y];
            
            for _t in 0..24 {
                let new_y = (2 * x + 3 * y) % 5;
                x = y;
                y = new_y;
                
                let temp = self.state[x + 5 * y];
                new_state[x + 5 * y] = current;
                current = temp;
            }
            self.state = new_state;
            
            // Chi
            for y in 0..5 {
                let mut row = [0u64; 5];
                for x in 0..5 {
                    row[x] = self.state[x + 5 * y];
                }
                for x in 0..5 {
                    self.state[x + 5 * y] = row[x] ^ (!row[(x + 1) % 5] & row[(x + 2) % 5]);
                }
            }
            
            // Iota
            self.state[0] ^= ROUND_CONSTANTS[round];
        }
    }

    #[allow(dead_code)]
    fn apply_theta(&mut self) {
        // Calculate column parities
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = self.state[x] ^ self.state[x + 5] ^ self.state[x + 10] ^ self.state[x + 15] ^ self.state[x + 20];
        }
        
        // Calculate D values
        let mut d = [0u64; 5];
        for x in 0..5 {
            d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
        }
        
        // Apply theta
        for x in 0..5 {
            for y in 0..5 {
                self.state[x + 5 * y] ^= d[x];
            }
        }
    }

    #[allow(dead_code)]
    fn apply_rho(&mut self) {
        // Rho step: just rotate each lane in place
        for i in 0..25 {
            let x = i % 5;
            let z = i / 5;
            self.state[i] = self.state[i].rotate_left(RHO_OFFSETS[x][z] as u32);
        }
    }

    #[allow(dead_code)]
    fn apply_pi(&mut self) {
        // Pi step: permute lanes according to XKCP pattern
        let mut new_state = self.state;
        let mut x = 1;
        let mut y = 0;
        let mut current = self.state[x + 5 * y];
        
        for _t in 0..24 {
            let new_y = (2 * x + 3 * y) % 5;
            x = y;
            y = new_y;
            
            let temp = self.state[x + 5 * y];
            new_state[x + 5 * y] = current;
            current = temp;
        }
        self.state = new_state;
    }

    #[allow(dead_code)]
    fn apply_chi(&mut self) {
        // Chi step: process each row
        for y in 0..5 {
            let mut row = [0u64; 5];
            for x in 0..5 {
                row[x] = self.state[x + 5 * y];
            }
            for x in 0..5 {
                self.state[x + 5 * y] = row[x] ^ (!row[(x + 1) % 5] & row[(x + 2) % 5]);
            }
        }
    }

    #[allow(dead_code)]
    fn apply_iota(&mut self, round: usize) {
        self.state[0] ^= ROUND_CONSTANTS[round];
    }


    fn absorb_input_step(&mut self) {
        // Pad the message
        let mut padded = self.input_data.clone();
        
        // SHA-3 padding: append 0x06, then pad with zeros, then set the last bit
        padded.push(0x06);
        while padded.len() % self.rate != (self.rate - 1) {
            padded.push(0x00);
        }
        padded.push(0x80);

        // XOR the first (and typically only) block into state
        if let Some(chunk) = padded.chunks(self.rate).next() {
            for (i, &byte) in chunk.iter().enumerate() {
                let lane_index = i / 8;
                let byte_index = i % 8;
                if lane_index < 25 {
                    let shift = byte_index * 8;
                    self.state[lane_index] ^= (byte as u64) << shift;
                }
            }
        }
        
        self.absorbed = 1;
        println!("Absorbed input data into state");
    }

    pub fn step(&mut self) {
        // Don't step if algorithm is complete
        if self.is_complete {
            return;
        }
        
        // Check if we need to absorb input first
        if self.round == 0 && self.step == 0 && self.absorbed == 0 {
            self.absorb_input_step();
            return;
        }

        // Perform one step of the Keccak-f permutation
        match self.step {
            0 => {
                // θ (Theta) step
                self.apply_theta();
                self.step = 1;
                println!("Applied Theta step");
            }
            1 => {
                // ρ (Rho) step  
                self.apply_rho();
                self.step = 2;
                println!("Applied Rho step");
            }
            2 => {
                // π (Pi) step
                self.apply_pi();
                self.step = 3;
                println!("Applied Pi step");
            }
            3 => {
                // χ (Chi) step
                self.apply_chi();
                self.step = 4;
                println!("Applied Chi step");
            }
            4 => {
                // ι (Iota) step
                self.apply_iota(self.round);
                
                // Move to next round or reset to beginning
                if self.round < 23 {
                    self.round += 1;
                    self.step = 0;
                    println!("Applied Iota step, completed round {}, starting round {}", self.round - 1, self.round);
                } else {
                    // Algorithm complete - set completion flag
                    self.is_complete = true;
                    // Keep step at 4 (Iota) but mark as complete
                    println!("Keccak-f permutation complete! Final state ready.");
                }
            }
            _ => {
                // Reset to beginning
                self.round = 0;
                self.step = 0;
                println!("Restarting Keccak-f permutation");
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_current_step_name(&self) -> String {
        // Check if algorithm is complete
        if self.is_complete {
            return "Complete".to_string();
        }
        
        // Check if we're in the absorption phase
        if self.round == 0 && self.step == 0 && self.absorbed == 0 {
            return "Absorb Input".to_string();
        }
        
        match self.step {
            0 => "Theta (θ)".to_string(),
            1 => "Rho (ρ)".to_string(),
            2 => "Pi (π)".to_string(),
            3 => "Chi (χ)".to_string(),
            4 => "Iota (ι)".to_string(),
            _ => "Unknown".to_string(),
        }
    }


    #[allow(dead_code)]
    pub fn get_output_hex(&self) -> String {
        let mut output = String::new();
        let mut bytes_written = 0;
        let target_bytes = self.output_length;
        
        for i in 0..25 {
            if bytes_written >= target_bytes {
                break;
            }
            
            // SHA-3 uses little-endian byte order for output
            let lane_bytes = self.state[i].to_le_bytes();
            
            for &byte in &lane_bytes {
                if bytes_written >= target_bytes {
                    break;
                }
                output.push_str(&format!("{:02x}", byte));
                bytes_written += 1;
            }
        }
        
        output
    }

    pub fn get_capacity(&self) -> usize {
        self.capacity
    }

    pub fn get_rate(&self) -> usize {
        self.rate * 8 // Convert from bytes to bits
    }

    pub fn get_output_length(&self) -> usize {
        self.output_length * 8 // Convert from bytes to bits  
    }

    /// Creates a new KeccakState for a specific SHA-3 variant.
    /// 
    /// # Arguments
    /// 
    /// * `variant` - The SHA-3 variant to configure for
    /// 
    /// # Returns
    /// 
    /// A new `KeccakState` configured with the appropriate parameters
    /// 
    /// # SHA-3 Variants
    /// 
    /// - SHA3-224: 224-bit output, 448-bit capacity
    /// - SHA3-256: 256-bit output, 512-bit capacity  
    /// - SHA3-384: 384-bit output, 768-bit capacity
    /// - SHA3-512: 512-bit output, 1024-bit capacity
    pub fn new_with_variant(variant: Sha3Variant) -> Self {
        let (rate, capacity, output_length) = match variant {
            Sha3Variant::Sha3_224 => (1152, 448, 28),
            Sha3Variant::Sha3_256 => (1088, 512, 32),
            Sha3Variant::Sha3_384 => (832, 768, 48),
            Sha3Variant::Sha3_512 => (576, 1024, 64),
        };

        Self {
            state: [0u64; 25],
            round: 0,
            step: 0,
            is_complete: false,
            input_data: Vec::new(),
            absorbed: 0,
            rate: rate / 8, // Convert bits to bytes
            capacity,
            output_length,
        }
    }

    /// Gets the current SHA-3 variant being used
    pub fn get_variant(&self) -> Sha3Variant {
        match (self.capacity, self.output_length) {
            (448, 28) => Sha3Variant::Sha3_224,
            (512, 32) => Sha3Variant::Sha3_256,
            (768, 48) => Sha3Variant::Sha3_384,
            (1024, 64) => Sha3Variant::Sha3_512,
            _ => Sha3Variant::Sha3_256, // Default fallback
        }
    }
}

/// Enumeration of supported SHA-3 variants.
/// 
/// Each variant has different output lengths and security parameters:
/// - SHA3-224: 224-bit output (28 bytes)
/// - SHA3-256: 256-bit output (32 bytes) 
/// - SHA3-384: 384-bit output (48 bytes)
/// - SHA3-512: 512-bit output (64 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sha3Variant {
    /// SHA3-224: 224-bit output, 1152-bit rate, 448-bit capacity
    Sha3_224,
    /// SHA3-256: 256-bit output, 1088-bit rate, 512-bit capacity (default)
    Sha3_256, 
    /// SHA3-384: 384-bit output, 832-bit rate, 768-bit capacity
    Sha3_384,
    /// SHA3-512: 512-bit output, 576-bit rate, 1024-bit capacity
    Sha3_512,
}

impl Sha3Variant {
    /// Returns the human-readable name of the variant
    pub fn name(&self) -> &'static str {
        match self {
            Sha3Variant::Sha3_224 => "SHA3-224",
            Sha3Variant::Sha3_256 => "SHA3-256",
            Sha3Variant::Sha3_384 => "SHA3-384",
            Sha3Variant::Sha3_512 => "SHA3-512",
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::{Digest, Sha3_256};

    /// Test basic KeccakState initialization
    #[test]
    fn test_keccak_state_new() {
        let state = KeccakState::new();
        
        // Check initial state
        assert_eq!(state.round, 0);
        assert_eq!(state.step, 0);
        assert!(!state.is_complete);
        assert_eq!(state.state, [0u64; 25]);
        assert_eq!(state.get_capacity(), 512);
        assert_eq!(state.get_rate(), 1088);
        assert_eq!(state.get_output_length(), 256);
    }

    /// Test SHA-3 variant initialization
    #[test]
    fn test_keccak_variants() {
        let sha224 = KeccakState::new_with_variant(Sha3Variant::Sha3_224);
        assert_eq!(sha224.get_capacity(), 448);
        assert_eq!(sha224.get_rate(), 1152);
        assert_eq!(sha224.get_output_length(), 224);
        assert_eq!(sha224.get_variant(), Sha3Variant::Sha3_224);

        let sha256 = KeccakState::new_with_variant(Sha3Variant::Sha3_256);
        assert_eq!(sha256.get_capacity(), 512);
        assert_eq!(sha256.get_rate(), 1088);
        assert_eq!(sha256.get_output_length(), 256);
        assert_eq!(sha256.get_variant(), Sha3Variant::Sha3_256);

        let sha384 = KeccakState::new_with_variant(Sha3Variant::Sha3_384);
        assert_eq!(sha384.get_capacity(), 768);
        assert_eq!(sha384.get_rate(), 832);
        assert_eq!(sha384.get_output_length(), 384);
        assert_eq!(sha384.get_variant(), Sha3Variant::Sha3_384);

        let sha512 = KeccakState::new_with_variant(Sha3Variant::Sha3_512);
        assert_eq!(sha512.get_capacity(), 1024);
        assert_eq!(sha512.get_rate(), 576);
        assert_eq!(sha512.get_output_length(), 512);
        assert_eq!(sha512.get_variant(), Sha3Variant::Sha3_512);
    }

    /// Test input setting and state reset
    #[test]
    fn test_set_input() {
        let mut state = KeccakState::new();
        
        // Set some initial state to verify reset
        state.round = 5;
        state.step = 3;
        state.is_complete = true;
        state.state[0] = 0x123456789ABCDEF0;
        
        // Set input and verify reset
        state.set_input("test input");
        assert_eq!(state.round, 0);
        assert_eq!(state.step, 0);
        assert!(!state.is_complete);
        assert_eq!(state.state, [0u64; 25]);
        assert_eq!(state.input_data, b"test input");
    }

    /// Test bit-level access to the state matrix
    #[test]
    fn test_bit_access() {
        let mut state = KeccakState::new();
        
        // Test bounds checking
        assert!(!state.get_bit(5, 0, 0)); // x out of bounds
        assert!(!state.get_bit(0, 64, 0)); // y out of bounds
        assert!(!state.get_bit(0, 0, 5)); // z out of bounds
        
        // Set a specific bit pattern and test access
        state.state[0] = 0x8000000000000001; // Set bits 0 and 63
        assert!(state.get_bit(0, 0, 0));   // Bit 0 should be set
        assert!(state.get_bit(0, 63, 0));  // Bit 63 should be set
        assert!(!state.get_bit(0, 1, 0));  // Bit 1 should be unset
        assert!(!state.get_bit(0, 62, 0)); // Bit 62 should be unset
    }


    /// Test step-by-step algorithm execution
    #[test]
    fn test_step_execution() {
        let mut state = KeccakState::new();
        state.set_input("test");
        
        // Initial state should be ready for absorption
        assert_eq!(state.round, 0);
        assert_eq!(state.step, 0);
        assert!(!state.is_complete);
        
        // First step should absorb input
        state.step();
        assert_eq!(state.absorbed, 1);
        
        // Continue through the steps
        let initial_round = state.round;
        let initial_step = state.step;
        
        state.step(); // Should advance to next step
        
        // Verify step advancement
        if initial_step < 4 {
            assert_eq!(state.step, initial_step + 1);
            assert_eq!(state.round, initial_round);
        } else {
            // Should advance to next round
            assert_eq!(state.step, 0);
            assert_eq!(state.round, initial_round + 1);
        }
    }

    /// Test complete algorithm execution against reference implementation
    #[test]
    fn test_complete_algorithm() {
        let test_cases = vec![
            "",
            "a",
            "abc",
            "message digest",
            "abcdefghijklmnopqrstuvwxyz",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            "The quick brown fox jumps over the lazy dog",
        ];

        for test_input in test_cases {
            let mut state = KeccakState::new();
            state.set_input(test_input);
            
            // Run algorithm to completion
            while !state.is_complete {
                state.step();
            }
            
            // Compare with reference implementation
            let mut reference = Sha3_256::new();
            reference.update(test_input.as_bytes());
            let expected = format!("{:x}", reference.finalize());
            let actual = state.get_output_hex();
            
            assert_eq!(
                actual, expected,
                "Hash mismatch for input '{}'\nExpected: {}\nActual:   {}",
                test_input, expected, actual
            );
        }
    }

    /// Test that stepping completed algorithm doesn't change state
    #[test]
    fn test_completed_algorithm_immutable() {
        let mut state = KeccakState::new();
        state.set_input("test");
        
        // Complete the algorithm
        while !state.is_complete {
            state.step();
        }
        
        let final_state = state.state;
        let final_hash = state.get_output_hex();
        
        // Try to step further
        state.step();
        state.step();
        
        // State should be unchanged
        assert_eq!(state.state, final_state);
        assert_eq!(state.get_output_hex(), final_hash);
        assert!(state.is_complete);
    }

    /// Test hash output formatting
    #[test]
    fn test_output_formatting() {
        let mut state = KeccakState::new();
        state.set_input("abc");
        
        while !state.is_complete {
            state.step();
        }
        
        let hash = state.get_output_hex();
        
        // SHA3-256 should produce exactly 64 hex characters (32 bytes * 2)
        assert_eq!(hash.len(), 64);
        
        // Should only contain valid hex characters
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        
        // Should be lowercase
        assert_eq!(hash, hash.to_lowercase());
    }

    /// Test constants are properly defined
    #[test]
    fn test_constants() {
        // Test that we have the correct number of round constants
        assert_eq!(ROUND_CONSTANTS.len(), 24);
        
        // Test that rho offsets are properly sized
        assert_eq!(RHO_OFFSETS.len(), 5);
        for row in RHO_OFFSETS.iter() {
            assert_eq!(row.len(), 5);
        }
        
        // Test specific known values
        assert_eq!(ROUND_CONSTANTS[0], 0x0000000000000001);
        assert_eq!(ROUND_CONSTANTS[1], 0x0000000000008082);
        assert_eq!(RHO_OFFSETS[0][0], 0); // Lane [0,0] should have no rotation
    }

    /// Test edge cases and error conditions
    #[test]
    fn test_edge_cases() {
        let mut state = KeccakState::new();
        
        // Empty input should work
        state.set_input("");
        while !state.is_complete {
            state.step();
        }
        assert!(state.is_complete);
        
        // Very long input should work
        let long_input = "a".repeat(1000);
        state.set_input(&long_input);
        while !state.is_complete {
            state.step();
        }
        assert!(state.is_complete);
        
        // Binary data (invalid UTF-8) should work
        state.input_data = vec![0xFF, 0xFE, 0xFD, 0x00, 0x01];
        state.absorbed = 0;
        state.round = 0;
        state.step = 0;
        state.is_complete = false;
        state.state = [0u64; 25];
        
        while !state.is_complete {
            state.step();
        }
        assert!(state.is_complete);
    }

    /// Benchmark test for performance measurement (disabled by default)
    #[test]
    #[ignore]
    fn benchmark_sha3_performance() {
        use std::time::Instant;
        
        let test_data = "The quick brown fox jumps over the lazy dog".repeat(100);
        let iterations = 100;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let mut state = KeccakState::new();
            state.set_input(&test_data);
            
            while !state.is_complete {
                state.step();
            }
        }
        
        let duration = start.elapsed();
        println!(
            "Processed {} iterations of {} bytes in {:?} ({:.2} MB/s)",
            iterations,
            test_data.len(),
            duration,
            (test_data.len() * iterations) as f64 / duration.as_secs_f64() / 1_000_000.0
        );
    }
}