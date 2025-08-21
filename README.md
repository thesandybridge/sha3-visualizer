# SHA-3 Visualizer

A real-time 3D visualization of the SHA-3 (Keccak) cryptographic hash function, built with Rust and Bevy.

## What is This?

This visualizer shows the internal state of the SHA-3/Keccak hash function as a 3D matrix of cubes. Each cube represents a single bit in the algorithm's 1600-bit internal state, arranged as a 5×5×64 matrix structure.

![SHA-3 Visualization](screenshot.png)

## What You're Looking At

### The Matrix Structure
- **5×5 grid**: Represents the 25 "lanes" of the Keccak state
- **64 bits high**: Each lane is a 64-bit integer, stacked vertically
- **Total: 1600 bits** (5 × 5 × 64 = 1600)

### Visual Elements
- **Cyan/Blue cubes**: Represent bits set to `1`
- **Dark gray cubes**: Represent bits set to `0`
- **Glowing effect**: Active bits have an emissive glow to make them stand out

## How SHA-3 Works

SHA-3 uses the Keccak-f[1600] permutation, which applies 5 transformations in each round:

1. **θ (Theta)**: Column parity calculation - XORs neighboring columns
2. **ρ (Rho)**: Bit rotation within each lane
3. **π (Pi)**: Lane permutation - rearranges the 5×5 grid
4. **χ (Chi)**: Non-linear transformation - the only non-linear step
5. **ι (Iota)**: Round constant addition - adds a round-specific constant

This process repeats for **24 rounds** total, thoroughly mixing the input data.

## Installation & Running

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable version)
- A graphics card with OpenGL/Vulkan/Metal support

### Building and Running
```bash
# Clone the repository
git clone <repository-url>
cd sha3-visualizer

# Run the visualizer
cargo run --release
```

The `--release` flag is recommended for better performance with 1600 cubes.

## Controls

### Camera Controls
- **Left mouse + drag**: Orbit around the matrix
- **Right mouse + drag**: Pan the view
- **Mouse wheel**: Zoom in/out

### SHA-3 Algorithm Controls
- **ENTER**: Step through one transformation at a time
- **P**: Toggle automatic animation (steps every second)
- **R**: Reset to initial state with input "Hello SHA-3!"

### Viewing Tips
- Start by orbiting around to get familiar with the 3D structure
- Zoom out to see the full 5×5×64 matrix
- Use manual stepping (ENTER) to see exactly how each transformation affects the bits
- Use automatic mode (P) to watch the full algorithm run continuously

## Understanding the Visualization

### Initial State
When you start the program, you'll see the SHA-3 state after absorbing the input "Hello SHA-3!". Some bits are already set from this initial processing.

### Watching Transformations
As you step through or animate:
- **θ (Theta)**: Watch columns affect each other
- **ρ (Rho)**: See bits rotate within their vertical lanes
- **π (Pi)**: Observe the entire grid rearrange
- **χ (Chi)**: Notice non-linear bit interactions
- **ι (Iota)**: See the first lane (bottom-left corner) change

### Pattern Recognition
- Early rounds show clear patterns from the input
- Later rounds appear increasingly random
- The final state (after 24 rounds) should look completely scrambled
- This scrambling is what makes SHA-3 cryptographically secure

## Technical Details

### Architecture
- **Engine**: Bevy (Rust game engine)
- **Rendering**: 3D PBR (Physically Based Rendering)
- **Performance**: ~1600 cube entities updated in real-time

### SHA-3 Implementation
- Complete Keccak-f[1600] permutation
- Proper padding and absorption phase
- All 5 transformation functions (θ, ρ, π, χ, ι)
- 24-round processing cycle

### Code Structure
```
src/
├── main.rs           # Bevy app setup and visualization logic
└── sha3/
    ├── mod.rs        # Complete SHA-3/Keccak implementation
    └── constants.rs  # Round constants and rotation offsets
```

## Educational Value

This visualizer helps understand:
- **Bit-level cryptography**: See exactly how bits change
- **Symmetric operations**: θ, ρ, and π are reversible
- **Non-linear security**: χ provides cryptographic strength
- **Avalanche effect**: Small input changes cause dramatic state changes
- **Diffusion**: How local changes spread throughout the state

## Troubleshooting

### Performance Issues
- Use `cargo run --release` for better performance
- Lower graphics settings in your system if frames drop
- The visualizer renders 1600 cubes, so older hardware may struggle

### Visual Issues
- If cubes appear black, ensure proper graphics drivers
- If the matrix looks wrong, try resetting with 'R'
- If camera feels sluggish, try zooming out first

### Build Issues
- Ensure you have the latest Rust stable
- On Linux, you may need additional graphics libraries
- On Windows, ensure you have Visual C++ redistributables

## Contributing

This is an educational project. Potential improvements:
- Different color schemes for each transformation type
- Add input field to hash custom text
- Export animation as video
- Add sound effects synchronized with bit changes
- Implement other hash functions (MD5, SHA-256) for comparison

## License

[Your license here]

## References

- [Keccak Team Official Site](https://keccak.team/)
- [NIST SHA-3 Standard](https://csrc.nist.gov/publications/detail/fips/202/final)
- [Keccak Reference Implementation](https://github.com/XKCP/XKCP)