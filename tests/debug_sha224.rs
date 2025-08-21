use sha3_visualizer::sha3_impl::{KeccakState, Sha3Variant};
use sha3::{Digest, Sha3_224};

#[test]
fn debug_sha224_output_length() {
    let mut state = KeccakState::new_with_variant(Sha3Variant::Sha3_224);
    state.set_input("");  // Empty input like the failing test
    
    println!("SHA3-224 configuration: output_length={} bytes", state.get_output_length() / 8);
    
    while !state.is_complete {
        state.step();
    }
    
    let hash = state.get_output_hex();
    println!("Our hash length: {} chars (should be 56)", hash.len());
    println!("Our hash: {}", hash);
    
    // Compare with standard library
    let mut hasher = Sha3_224::new();
    hasher.update(b"");
    let expected = format!("{:x}", hasher.finalize());
    println!("Expected length: {} chars", expected.len());
    println!("Expected: {}", expected);
    
    // The test should pass once fixed
    assert_eq!(hash.len(), 56, "SHA3-224 should produce 56 hex characters");
    assert_eq!(hash, expected, "Hash should match standard library");
}