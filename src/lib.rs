//! SHA-3 Visualizer Library
//!
//! A comprehensive implementation of the SHA-3 (Keccak) cryptographic hash function
//! with step-by-step execution capabilities for educational visualization.
//!
//! # Features
//!
//! - Complete SHA-3/Keccak implementation with all transformation steps
//! - Step-by-step execution for educational purposes  
//! - Support for all SHA-3 variants (224, 256, 384, 512 bits)
//! - 3D visualization support through bit-level state access
//! - Comprehensive testing and verification against standard implementations
//!
//! # Example
//!
//! ```rust
//! use sha3_visualizer::sha3_impl::KeccakState;
//!
//! // Create a new SHA-3 state and process input
//! let mut state = KeccakState::new();
//! state.set_input("Hello SHA-3!");
//!
//! // Step through the algorithm
//! while !state.is_complete {
//!     state.step();
//! }
//!
//! // Get the final hash
//! let hash = state.get_output_hex();
//! println!("SHA3-256: {}", hash);
//! ```
//!
//! # Modules
//!
//! - [`sha3_impl`]: Core SHA-3/Keccak implementation with step-by-step execution
//!
//! # Educational Use
//!
//! This library is designed for educational purposes, allowing users to:
//! - Understand the internal workings of SHA-3
//! - Visualize how each transformation affects the algorithm state
//! - Step through the algorithm at their own pace
//! - Verify implementation correctness against standard libraries

pub mod sha3_impl;

#[cfg(target_arch = "wasm32")]
pub mod wasm_bindings;

// Re-export commonly used types for convenience
pub use sha3_impl::{KeccakState, Sha3Variant};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library description
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");