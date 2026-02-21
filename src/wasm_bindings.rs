use wasm_bindgen::prelude::*;

use crate::sha3_impl::KeccakState;

#[wasm_bindgen]
pub struct Sha3WasmState {
    inner: KeccakState,
}

#[wasm_bindgen]
impl Sha3WasmState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: KeccakState::new(),
        }
    }

    pub fn set_input(&mut self, input: &str) {
        self.inner.set_input(input);
    }

    pub fn step(&mut self) {
        self.inner.step();
    }

    pub fn is_complete(&self) -> bool {
        self.inner.is_complete
    }

    pub fn get_round(&self) -> usize {
        self.inner.round
    }

    pub fn get_step_index(&self) -> usize {
        self.inner.step
    }

    pub fn get_current_step_name(&self) -> String {
        self.inner.get_current_step_name()
    }

    /// Returns the lane value at (x, z) as f64. Precision is limited for large u64 values,
    /// but sufficient for visual density calculations.
    pub fn get_lane(&self, x: usize, z: usize) -> f64 {
        if x >= 5 || z >= 5 {
            return 0.0;
        }
        self.inner.state[x + 5 * z] as f64
    }

    /// Returns the full 1600-bit state as 200 bytes (25 lanes × 8 bytes, little-endian).
    /// Efficient for bulk transfer to JS for visualization.
    pub fn get_state_as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(200);
        for &lane in &self.inner.state {
            bytes.extend_from_slice(&lane.to_le_bytes());
        }
        bytes
    }

    pub fn get_output_hex(&self) -> String {
        self.inner.get_output_hex()
    }
}
