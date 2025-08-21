//! Constants for the SHA-3/Keccak algorithm.
//!
//! This module contains the fixed constants required for the Keccak-f[1600] permutation:
//! - Round constants for the Iota (ι) transformation
//! - Rotation offsets for the Rho (ρ) transformation
//!
//! These constants are derived from the official Keccak specification and are essential
//! for the cryptographic security of the algorithm.

/// Round constants for the Iota (ι) transformation step.
/// 
/// These 24 constants are XORed with the first lane (position [0,0]) during each round
/// of the Keccak-f[1600] permutation. Each constant is carefully chosen to ensure
/// cryptographic security by breaking symmetry in the algorithm.
/// 
/// The constants are generated using a linear feedback shift register (LFSR) as
/// specified in the Keccak documentation.
pub const ROUND_CONSTANTS: [u64; 24] = [
    0x0000000000000001,
    0x0000000000008082,
    0x800000000000808A,
    0x8000000080008000,
    0x000000000000808B,
    0x0000000080000001,
    0x8000000080008081,
    0x8000000000008009,
    0x000000000000008A,
    0x0000000000000088,
    0x0000000080008009,
    0x000000008000000A,
    0x000000008000808B,
    0x800000000000008B,
    0x8000000000008089,
    0x8000000000008003,
    0x8000000000008002,
    0x8000000000000080,
    0x000000000000800A,
    0x800000008000000A,
    0x8000000080008081,
    0x8000000000008080,
    0x0000000080000001,
    0x8000000080008008,
];

/// Rotation offsets for the Rho (ρ) transformation step.
/// 
/// This 5×5 matrix specifies how many bit positions to rotate each lane during
/// the Rho step. The offsets are indexed as `RHO_OFFSETS[x][z]` where:
/// - `x` is the X coordinate (0-4)
/// - `z` is the Z coordinate (0-4)
/// 
/// These offsets ensure that each bit position in the state is rotated by a
/// different amount, providing optimal diffusion properties.
/// 
/// Note: The offset for lane [0,0] is always 0 (no rotation).
pub const RHO_OFFSETS: [[usize; 5]; 5] = [
    [0, 36, 3, 41, 18],
    [1, 44, 10, 45, 2],
    [62, 6, 43, 15, 61],
    [28, 55, 25, 21, 56],
    [27, 20, 39, 8, 14],
];