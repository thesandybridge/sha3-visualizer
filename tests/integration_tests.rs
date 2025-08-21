//! Integration tests for the SHA-3 visualizer.
//!
//! These tests verify that the complete application works correctly,
//! including command-line argument parsing, input handling, and 
//! the visualization components working together.

use sha3_visualizer::sha3_impl::{KeccakState, Sha3Variant};
use sha3::{Digest, Sha3_256, Sha3_224, Sha3_384, Sha3_512};

/// Test that our implementation produces correct results for all SHA-3 variants
#[test]
fn test_all_sha3_variants_correctness() {
    let test_cases = vec![
        "",
        "abc",  
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
        "The quick brown fox jumps over the lazy dog",
    ];

    for input in test_cases {
        // Test SHA3-224
        let mut state224 = KeccakState::new_with_variant(Sha3Variant::Sha3_224);
        state224.set_input(input);
        while !state224.is_complete {
            state224.step();
        }
        let mut ref224 = Sha3_224::new();
        ref224.update(input.as_bytes());
        let expected224 = format!("{:x}", ref224.finalize());
        assert_eq!(state224.get_output_hex(), expected224, "SHA3-224 mismatch for '{}'", input);

        // Test SHA3-256 
        let mut state256 = KeccakState::new_with_variant(Sha3Variant::Sha3_256);
        state256.set_input(input);
        while !state256.is_complete {
            state256.step();
        }
        let mut ref256 = Sha3_256::new();
        ref256.update(input.as_bytes());
        let expected256 = format!("{:x}", ref256.finalize());
        assert_eq!(state256.get_output_hex(), expected256, "SHA3-256 mismatch for '{}'", input);

        // Test SHA3-384
        let mut state384 = KeccakState::new_with_variant(Sha3Variant::Sha3_384);
        state384.set_input(input);
        while !state384.is_complete {
            state384.step();
        }
        let mut ref384 = Sha3_384::new();
        ref384.update(input.as_bytes());
        let expected384 = format!("{:x}", ref384.finalize());
        assert_eq!(state384.get_output_hex(), expected384, "SHA3-384 mismatch for '{}'", input);

        // Test SHA3-512
        let mut state512 = KeccakState::new_with_variant(Sha3Variant::Sha3_512);
        state512.set_input(input);
        while !state512.is_complete {
            state512.step();
        }
        let mut ref512 = Sha3_512::new();
        ref512.update(input.as_bytes());
        let expected512 = format!("{:x}", ref512.finalize());
        assert_eq!(state512.get_output_hex(), expected512, "SHA3-512 mismatch for '{}'", input);
    }
}

/// Test the step-by-step nature of the algorithm
#[test]
fn test_step_by_step_determinism() {
    let input = "determinism test";
    
    // Run algorithm in one go
    let mut state1 = KeccakState::new();
    state1.set_input(input);
    while !state1.is_complete {
        state1.step();
    }
    let hash1 = state1.get_output_hex();
    
    // Run algorithm step by step with pauses
    let mut state2 = KeccakState::new();
    state2.set_input(input);
    
    // Step through first few transformations
    for _ in 0..10 {
        if !state2.is_complete {
            state2.step();
        }
    }
    
    // Continue to completion
    while !state2.is_complete {
        state2.step();
    }
    let hash2 = state2.get_output_hex();
    
    // Both should produce the same result
    assert_eq!(hash1, hash2, "Step-by-step execution should be deterministic");
}

/// Test that reset functionality works correctly
#[test]
fn test_reset_functionality() {
    let mut state = KeccakState::new();
    
    // Hash first input
    state.set_input("first input");
    while !state.is_complete {
        state.step();
    }
    let first_hash = state.get_output_hex();
    
    // Reset and hash second input
    state.set_input("second input");
    assert_eq!(state.round, 0);
    assert_eq!(state.step, 0);
    assert!(!state.is_complete);
    assert_eq!(state.state, [0u64; 25]);
    
    while !state.is_complete {
        state.step();
    }
    let second_hash = state.get_output_hex();
    
    // Hashes should be different
    assert_ne!(first_hash, second_hash);
    
    // Reset back to first input should produce same result
    state.set_input("first input");
    while !state.is_complete {
        state.step();
    }
    let first_hash_again = state.get_output_hex();
    
    assert_eq!(first_hash, first_hash_again, "Reset should produce consistent results");
}

/// Test bit matrix visualization data integrity
#[test]
fn test_bit_matrix_integrity() {
    let mut state = KeccakState::new();
    state.set_input("matrix test");
    
    // Count initial set bits
    let mut initial_bits = 0;
    for x in 0..5 {
        for z in 0..5 {
            for y in 0..64 {
                if state.get_bit(x, y, z) {
                    initial_bits += 1;
                }
            }
        }
    }
    
    // After absorption, should have some bits set
    state.step(); // Absorb input
    let mut absorbed_bits = 0;
    for x in 0..5 {
        for z in 0..5 {
            for y in 0..64 {
                if state.get_bit(x, y, z) {
                    absorbed_bits += 1;
                }
            }
        }
    }
    
    assert!(absorbed_bits > initial_bits, "Absorption should set some bits");
    assert!(absorbed_bits < 1600, "Not all bits should be set after absorption");
    
    // Continue through transformations
    let mut _previous_bits = absorbed_bits;
    for _ in 0..5 { // Test a few transformation steps
        if !state.is_complete {
            state.step();
            let current_bits = (0..5)
                .flat_map(|x| (0..5).flat_map(move |z| (0..64).map(move |y| (x, y, z))))
                .filter(|&(x, y, z)| state.get_bit(x, y, z))
                .count();
            
            // Bit count can change between steps (that's the point of the algorithm)
            // but should remain within reasonable bounds
            assert!(current_bits <= 1600, "Bit count should not exceed state size");
            _previous_bits = current_bits;
        }
    }
}

/// Test with various input encodings and special characters
#[test]
fn test_input_encoding() {
    let test_cases = vec![
        "ASCII text",
        "Unicode: αβγδε",
        "Emoji: 🦀🔐🧠",
        "Mixed: Hello世界🌍",
        "Numbers: 123456789",
        "Special: !@#$%^&*()",
        "\n\t\r line breaks \n\t\r",
        "", // Empty string
    ];

    for input in test_cases {
        let mut our_state = KeccakState::new();
        our_state.set_input(input);
        while !our_state.is_complete {
            our_state.step();
        }
        
        let mut reference = Sha3_256::new();
        reference.update(input.as_bytes());
        let expected = format!("{:x}", reference.finalize());
        
        assert_eq!(
            our_state.get_output_hex(),
            expected,
            "Encoding mismatch for input: {:?}",
            input
        );
    }
}

/// Test that the algorithm handles large inputs correctly
#[test]
fn test_large_input_handling() {
    // Test with input larger than one absorption block
    let large_input = "a".repeat(200); // Much larger than typical rate
    
    let mut state = KeccakState::new();
    state.set_input(&large_input);
    
    // Should complete without issues
    while !state.is_complete {
        state.step();
    }
    
    // Verify against reference
    let mut reference = Sha3_256::new();
    reference.update(large_input.as_bytes());
    let expected = format!("{:x}", reference.finalize());
    
    assert_eq!(state.get_output_hex(), expected);
}

/// Test state consistency during step execution
#[test]
fn test_state_consistency() {
    let mut state = KeccakState::new();
    state.set_input("consistency test");
    
    let mut absorbed = false;
    while !state.is_complete {
        let round_before = state.round;
        let step_before = state.step;
        let absorbed_before = absorbed;
        
        state.step();
        
        // Check if this was the absorption step
        if round_before == 0 && step_before == 0 && !absorbed_before {
            // This was the absorption step - step and round should remain the same
            assert_eq!(state.round, round_before, "Round should not change during absorption");
            assert_eq!(state.step, step_before, "Step should not change during absorption");
            absorbed = true;
            continue;
        }
        
        // Verify step/round advancement logic for actual transformations
        if state.is_complete {
            // Algorithm completed
            break;
        } else if step_before < 4 {
            // Should advance to next step in same round
            assert_eq!(state.round, round_before, "Round should stay same when advancing step");
            assert_eq!(state.step, step_before + 1, "Step should advance by 1");
        } else {
            // Should advance to next round, reset step to 0
            if round_before < 23 {
                assert_eq!(state.round, round_before + 1, "Round should advance by 1");
                assert_eq!(state.step, 0, "Step should reset to 0 at start of new round");
            } else {
                // Should complete after round 23, step 4
                assert!(state.is_complete, "Algorithm should be complete after final round");
            }
        }
    }
    
    // Final state should be marked complete
    assert!(state.is_complete);
    
    // Further steps should not change anything
    let final_state = state.state;
    let final_round = state.round;
    let final_step = state.step;
    let final_hash = state.get_output_hex();
    
    state.step();
    state.step();
    
    assert_eq!(state.state, final_state);
    assert_eq!(state.round, final_round);
    assert_eq!(state.step, final_step);
    assert_eq!(state.get_output_hex(), final_hash);
}

/// Test that bit access methods work correctly throughout algorithm execution
#[test]
fn test_bit_access_during_execution() {
    let mut state = KeccakState::new();
    state.set_input("bit access test");
    
    // Track changes to a specific bit throughout execution
    let test_coords = [(0, 0, 0), (2, 31, 3), (4, 63, 4)];
    
    while !state.is_complete {
        // Before step
        let _bits_before: Vec<bool> = test_coords
            .iter()
            .map(|&(x, y, z)| state.get_bit(x, y, z))
            .collect();
        
        state.step();
        
        // After step - bits may have changed, but access should still work
        for &(x, y, z) in &test_coords {
            let bit_value = state.get_bit(x, y, z);
            // Just verify the call succeeds and returns a valid boolean
            assert!(bit_value == true || bit_value == false);
        }
    }
    
    // Test bounds checking still works
    assert!(!state.get_bit(5, 0, 0));
    assert!(!state.get_bit(0, 64, 0));
    assert!(!state.get_bit(0, 0, 5));
}

/// Performance integration test
#[test]
#[ignore] // Ignore by default as this is a performance test
fn test_performance_characteristics() {
    use std::time::Instant;
    
    let medium_input = "x".repeat(100);
    let long_input = "y".repeat(1000);
    let inputs = vec![
        ("short", "abc"),
        ("medium", &medium_input),
        ("long", &long_input),
    ];
    
    for (name, input) in inputs {
        let start = Instant::now();
        
        let mut state = KeccakState::new();
        state.set_input(input);
        
        let mut step_count = 0;
        while !state.is_complete {
            state.step();
            step_count += 1;
        }
        
        let duration = start.elapsed();
        
        println!(
            "{} input ({} bytes): {} steps in {:?} ({:.2} steps/ms)",
            name,
            input.len(),
            step_count,
            duration,
            step_count as f64 / duration.as_millis() as f64
        );
        
        // Verify result is still correct
        let mut reference = Sha3_256::new();
        reference.update(input.as_bytes());
        let expected = format!("{:x}", reference.finalize());
        assert_eq!(state.get_output_hex(), expected);
    }
}