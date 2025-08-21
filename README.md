# SHA-3 Visualizer

A real-time 3D visualization of the SHA-3 (Keccak) cryptographic hash function with step-by-step execution, built with Rust and Bevy.

## What is This?

This visualizer shows the internal state of the SHA-3/Keccak hash function as a 3D matrix of cubes. Each cube represents a single bit in the algorithm's 1600-bit internal state, arranged as a 5×5×64 matrix structure. Watch each of the 5 transformation steps with color-coded visualization and step through 24 rounds to see how SHA-3 processes your input.

![SHA-3 Visualization](screenshot.png)

## What You're Looking At

### The Matrix Structure
- **5×5 grid**: Represents the 25 "lanes" of the Keccak state
- **64 bits high**: Each lane is a 64-bit integer, stacked vertically
- **Total: 1600 bits** (5 × 5 × 64 = 1600)

### Visual Elements
- **Color-coded transformations**: Each step has its own color
  - **Red**: θ (Theta) - Column mixing
  - **Green**: ρ (Rho) - Bit rotation
  - **Blue**: π (Pi) - Lane rearrangement
  - **Magenta**: χ (Chi) - Non-linear transformation
  - **Yellow**: ι (Iota) - Round constant addition
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

#### Basic Usage (Default Input)
```bash
# Clone the repository
git clone <repository-url>
cd sha3-visualizer

# Run with default input "Hello SHA-3!"
cargo run --release
```

#### Command Line Input
```bash
# Hash a specific string
cargo run --release "My custom message"

# Hash multiple words
cargo run --release "The quick brown fox"
```

#### Using stdin (Pipe Input)
```bash
# Hash file contents
cat myfile.txt | cargo run --release

# Hash command output
echo "Hello World" | cargo run --release

# Hash from another program
curl -s https://example.com | cargo run --release
```

The `--release` flag is recommended for better performance with 1600 cubes.

## Controls & Keybinds

### Camera Controls
- **Left mouse + drag**: Orbit around the matrix
- **Right mouse + drag**: Pan the view
- **Mouse wheel**: Zoom in/out

### SHA-3 Algorithm Controls
- **ENTER**: Step through one transformation at a time
- **P**: Toggle automatic animation (plays/pauses stepping every second)
- **R**: Reset to initial state with the same input
- **F**: Fast-forward to completion (run all remaining steps instantly)

### Visual Interface
- **Top Panel**: Shows input string and current hash output
- **Bottom Panel**: Displays current round, step, and color legend
- **Real-time Updates**: Hash output updates as you step through
- **Completion Status**: Shows when hash is complete with verification

### Viewing Tips
- Start by orbiting around to get familiar with the 3D structure
- Zoom out to see the full 5×5×64 matrix
- Use manual stepping (ENTER) to see exactly how each transformation affects the bits
- Watch the color changes to understand which transformation is active
- Use automatic mode (P) to watch the full algorithm run continuously
- Use fast-forward (F) to jump to the final result

## Understanding the Visualization

### Initial State
When you start the program, you'll see the SHA-3 state after absorbing your input. Some bits are already set from this initial processing phase.

### Watching Transformations
As you step through or animate, watch for:
- **θ (Theta - Red)**: Watch columns affect each other through XOR operations
- **ρ (Rho - Green)**: See bits rotate within their vertical lanes
- **π (Pi - Blue)**: Observe the entire grid rearrange as lanes move position
- **χ (Chi - Magenta)**: Notice non-linear bit interactions (the security step)
- **ι (Iota - Yellow)**: See the first lane (bottom-left corner) change with round constants

### Pattern Recognition
- Early rounds show clear patterns from the input
- Later rounds appear increasingly random due to mixing
- The final state (after 24 rounds) should look completely scrambled
- This scrambling is what makes SHA-3 cryptographically secure

### Hash Verification
The program automatically verifies your hash against Rust's standard `sha3` library and displays the result in both the UI and terminal output.

## Input/Output Features

### Input Methods
1. **Command line argument**: `cargo run --release "your text here"`
2. **Standard input (stdin)**: `echo "text" | cargo run --release`
3. **Default**: Uses "Hello SHA-3!" if no input provided

### Output Methods
- **Visual**: Real-time hash display in the UI
- **Terminal**: Final hash printed to stdout when complete
- **Verification**: Automatic comparison with reference implementation

### Terminal Output Example
```
SHA-3 step: Round 1, Step 0
SHA-3 step: Round 1, Step 1
...
SHA-3 COMPLETE!
Our hash:      a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a
Expected hash: a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a
✓ HASH VERIFICATION PASSED!
```

## Technical Details

### Architecture
- **Engine**: Bevy 0.14 (Rust game engine)
- **Rendering**: 3D PBR (Physically Based Rendering)
- **Performance**: ~1600 cube entities updated in real-time
- **Input Handling**: Command-line parsing with `clap`
- **TTY Detection**: Automatic stdin detection using `atty`

### SHA-3 Implementation
- Complete Keccak-f[1600] permutation
- Proper padding and absorption phase
- All 5 transformation functions (θ, ρ, π, χ, ι)
- 24-round processing cycle
- SHA3-256 output (256-bit hash)
- Verification against standard library

### Code Structure
```
src/
├── main.rs           # Bevy app setup, input handling, and visualization logic
├── sha3_impl/
│   ├── mod.rs        # Complete SHA-3/Keccak implementation
│   └── constants.rs  # Round constants and rotation offsets
└── ui.rs             # UI components and rendering (legacy)
```

## Educational Value

This visualizer helps understand:
- **Bit-level cryptography**: See exactly how individual bits change
- **Step-by-step execution**: Understand each transformation's purpose
- **Visual learning**: Color-coded steps make the algorithm intuitive
- **Avalanche effect**: Small input changes cause dramatic state changes
- **Diffusion**: How local changes spread throughout the entire state
- **Non-linear security**: Why χ (Chi) is the critical security transformation
- **Hash verification**: Learn about cryptographic correctness

## Use Cases

### Educational
- Computer science courses on cryptography
- Understanding hash function internals
- Visual learning of the Keccak algorithm
- Demonstrating cryptographic properties

### Development
- Testing SHA-3 implementations
- Debugging hash function behavior
- Comparing different inputs
- Understanding performance characteristics

### Research
- Analyzing bit patterns in cryptographic functions
- Studying the diffusion properties of Keccak
- Investigating round-by-round transformations

## Troubleshooting

### Performance Issues
- Use `cargo run --release` for better performance
- Lower graphics settings in your system if frames drop
- The visualizer renders 1600 cubes, so older hardware may struggle
- Close other graphics-intensive applications

### Visual Issues
- If cubes appear black, ensure proper graphics drivers are installed
- If the matrix looks wrong, try resetting with 'R'
- If camera feels sluggish, try zooming out first
- Ensure your system supports OpenGL/Vulkan/Metal

### Build Issues
- Ensure you have the latest Rust stable toolchain
- On Linux, you may need additional graphics libraries (`sudo apt-get install pkg-config libx11-dev libasound2-dev libudev-dev`)
- On Windows, ensure you have Visual C++ redistributables
- On macOS, ensure Xcode command line tools are installed

### Input Issues
- Large inputs may take time to process - use fast-forward (F)
- Binary data may not display well in the UI text fields
- Very long inputs are truncated in the UI but fully processed

## Advanced Usage

### Scripting
```bash
# Batch process multiple inputs
for input in "test1" "test2" "test3"; do
  echo "$input" | cargo run --release
done

# Hash file and save visualization steps
echo "monitoring" | cargo run --release > output.log
```

### Performance Testing
```bash
# Time hash completion
time (echo "performance test" | cargo run --release)

# Memory usage monitoring
/usr/bin/time -v cargo run --release "memory test"
```

## Contributing

This is an educational project. Potential improvements:
- Support for other SHA-3 variants (SHA3-224, SHA3-384, SHA3-512)
- SHAKE128/SHAKE256 visualization
- Export functionality for animations or screenshots
- Sound effects synchronized with transformations
- Comparison mode with other hash functions (SHA-256, Blake3)
- Interactive input editing within the visualizer
- Mobile/web version using wasm-bindgen

## License

[Your license here]

## References

- [Keccak Team Official Site](https://keccak.team/)
- [NIST SHA-3 Standard](https://csrc.nist.gov/publications/detail/fips/202/final)
- [Keccak Reference Implementation](https://github.com/XKCP/XKCP)
- [Bevy Game Engine](https://bevyengine.org/)
- [Understanding Keccak/SHA-3](https://keccak.team/keccak_specs_summary.html)
