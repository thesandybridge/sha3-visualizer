pub mod constants;

use constants::*;

#[derive(Debug, Clone)]
pub struct KeccakState {
    pub state: [u64; 25], // 5x5 array of 64-bit lanes
    pub round: usize,
    pub step: usize, // Current step within the round (0-4 for θ,ρ,π,χ,ι)
    #[allow(dead_code)]
    input_data: Vec<u8>,
    #[allow(dead_code)]
    absorbed: usize,
    #[allow(dead_code)]
    rate: usize, // Rate in bytes (1600 - capacity) / 8
    #[allow(dead_code)]
    capacity: usize, // Capacity in bits
    #[allow(dead_code)]
    output_length: usize, // Desired output length in bytes
    #[allow(dead_code)]
    step_history: Vec<StepHistory>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StepHistory {
    pub round: usize,
    pub step: usize,
    pub step_name: String,
    pub state_before: [u64; 25],
    pub state_after: [u64; 25],
    pub description: String,
}

impl KeccakState {
    pub fn new() -> Self {
        Self {
            state: [0u64; 25],
            round: 0,
            step: 0,
            input_data: Vec::new(),
            absorbed: 0,
            rate: 1088 / 8, // SHA3-256 rate
            capacity: 512,
            output_length: 32, // SHA3-256 output
            step_history: Vec::new(),
        }
    }

    #[allow(dead_code)]
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
            input_data: Vec::new(),
            absorbed: 0,
            rate: rate / 8,
            capacity,
            output_length,
            step_history: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn set_input(&mut self, data: &str) {
        self.input_data = data.as_bytes().to_vec();
        self.absorbed = 0;
        self.round = 0;
        self.step = 0;
        self.state = [0u64; 25];
        self.step_history.clear();
        self.prepare_input_without_permutation();
    }

    // Prepare input for visualization without running the permutation
    fn prepare_input_without_permutation(&mut self) {
        // Pad the message
        let mut padded = self.input_data.clone();
        
        // SHA-3 padding: append 0x06, then pad with zeros, then set the last bit
        padded.push(0x06);
        while padded.len() % self.rate != (self.rate - 1) {
            padded.push(0x00);
        }
        padded.push(0x80);

        // XOR the first (and typically only) block into state
        // For visualization, we only process the first block to start at round 0
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
        
        // Reset to beginning of permutation for visualization
        self.round = 0;
        self.step = 0;
    }

    pub fn get_bit(&self, x: usize, y: usize, z: usize) -> bool {
        if x >= 5 || y >= 64 || z >= 5 {
            return false;
        }
        let _lane_index = 5 * y + x; // Wait, this is wrong. Let me fix this.
        let lane_index = 5 * x + z; // Correct indexing: x + 5*z
        let bit_position = y;
        (self.state[lane_index] >> bit_position) & 1 == 1
    }

    #[allow(dead_code)]
    pub fn get_lane(&self, x: usize, z: usize) -> u64 {
        if x >= 5 || z >= 5 {
            return 0;
        }
        self.state[x + 5 * z]
    }

    #[allow(dead_code)]
    pub fn set_lane(&mut self, x: usize, z: usize, value: u64) {
        if x < 5 && z < 5 {
            self.state[x + 5 * z] = value;
        }
    }

    #[allow(dead_code)]
    fn absorb_phase(&mut self) {
        // Pad the message
        let mut padded = self.input_data.clone();
        
        // SHA-3 padding: append 0x06, then pad with zeros, then set the last bit
        padded.push(0x06);
        while padded.len() % self.rate != (self.rate - 1) {
            padded.push(0x00);
        }
        padded.push(0x80);

        // Absorb blocks
        for chunk in padded.chunks(self.rate) {
            // XOR chunk into state
            for (i, &byte) in chunk.iter().enumerate() {
                let lane_index = i / 8;
                let byte_index = i % 8;
                if lane_index < 25 {
                    let shift = byte_index * 8;
                    self.state[lane_index] ^= (byte as u64) << shift;
                }
            }
            
            // Apply Keccak-f[1600] permutation
            self.apply_keccak_f();
        }
    }

    #[allow(dead_code)]
    fn apply_keccak_f(&mut self) {
        for round in 0..24 {
            self.round = round;
            
            // θ (Theta) step
            self.step = 0;
            self.apply_theta();
            
            // ρ (Rho) step  
            self.step = 1;
            self.apply_rho();
            
            // π (Pi) step
            self.step = 2;
            self.apply_pi();
            
            // χ (Chi) step
            self.step = 3;
            self.apply_chi();
            
            // ι (Iota) step
            self.step = 4;
            self.apply_iota(round);
        }
    }

    #[allow(dead_code)]
    fn apply_theta(&mut self) {
        let state_before = self.state;
        
        // Calculate column parities
        let mut c = [0u64; 5];
        for x in 0..5 {
            for z in 0..5 {
                c[x] ^= self.get_lane(x, z);
            }
        }
        
        // Calculate D values
        let mut d = [0u64; 5];
        for x in 0..5 {
            d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
        }
        
        // Apply theta
        for x in 0..5 {
            for z in 0..5 {
                let old_value = self.get_lane(x, z);
                self.set_lane(x, z, old_value ^ d[x]);
            }
        }

        self.record_step("Theta (θ)", state_before, "Column parity calculation and XOR");
    }

    #[allow(dead_code)]
    fn apply_rho(&mut self) {
        let state_before = self.state;
        
        let mut new_state = self.state;
        for x in 0..5 {
            for z in 0..5 {
                let offset = RHO_OFFSETS[x][z];
                new_state[x + 5 * z] = self.state[x + 5 * z].rotate_left(offset as u32);
            }
        }
        self.state = new_state;

        self.record_step("Rho (ρ)", state_before, "Bit rotation within lanes");
    }

    #[allow(dead_code)]
    fn apply_pi(&mut self) {
        let state_before = self.state;
        
        let mut new_state = [0u64; 25];
        for x in 0..5 {
            for z in 0..5 {
                let new_x = (x + 3 * z) % 5;
                let new_z = x;
                new_state[new_x + 5 * new_z] = self.state[x + 5 * z];
            }
        }
        self.state = new_state;

        self.record_step("Pi (π)", state_before, "Lane permutation");
    }

    #[allow(dead_code)]
    fn apply_chi(&mut self) {
        let state_before = self.state;
        
        let mut new_state = [0u64; 25];
        for x in 0..5 {
            for z in 0..5 {
                let lane = self.get_lane(x, z) ^ 
                    ((!self.get_lane((x + 1) % 5, z)) & self.get_lane((x + 2) % 5, z));
                new_state[x + 5 * z] = lane;
            }
        }
        self.state = new_state;

        self.record_step("Chi (χ)", state_before, "Non-linear transformation");
    }

    #[allow(dead_code)]
    fn apply_iota(&mut self, round: usize) {
        let state_before = self.state;
        
        self.state[0] ^= ROUND_CONSTANTS[round];

        self.record_step("Iota (ι)", state_before, &format!("Round constant XOR (round {})", round));
    }

    #[allow(dead_code)]
    fn record_step(&mut self, step_name: &str, state_before: [u64; 25], description: &str) {
        self.step_history.push(StepHistory {
            round: self.round,
            step: self.step,
            step_name: step_name.to_string(),
            state_before,
            state_after: self.state,
            description: description.to_string(),
        });
    }

    pub fn step(&mut self) {
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
                    // Reset to beginning for continuous demo
                    self.round = 0;
                    self.step = 0;
                    println!("Keccak-f permutation complete! Restarting from round 0");
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
    pub fn get_history(&self) -> &Vec<StepHistory> {
        &self.step_history
    }

    #[allow(dead_code)]
    pub fn get_output_hex(&self) -> String {
        let mut output = String::new();
        for i in 0..(self.output_length / 8) {
            if i < 25 {
                output.push_str(&format!("{:016x}", self.state[i].swap_bytes()));
            }
        }
        output[0..self.output_length * 2].to_string()
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
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Sha3Variant {
    Sha3_224,
    Sha3_256, 
    Sha3_384,
    Sha3_512,
}