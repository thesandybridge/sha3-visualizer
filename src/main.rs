//! SHA-3 Visualizer
//!
//! A real-time 3D visualization of the SHA-3 (Keccak) cryptographic hash function.
//! This application provides an interactive step-by-step visualization of the SHA-3
//! algorithm's internal state transformations, displaying each bit as a colored cube
//! in a 3D matrix representing the 1600-bit Keccak state.
//!
//! # Features
//!
//! - Interactive stepping through SHA-3's 5 transformation functions (θ, ρ, π, χ, ι)
//! - Color-coded visualization of each transformation step
//! - Multiple input methods: command-line arguments, stdin, or default
//! - Real-time hash verification against the standard library implementation
//! - Camera controls for exploring the 3D state matrix
//!
//! # Usage
//!
//! ```bash
//! # Use default input "Hello SHA-3!"
//! cargo run --release
//!
//! # Hash a specific string
//! cargo run --release "your message here"
//!
//! # Hash from stdin
//! echo "test" | cargo run --release
//! ```

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use clap::Parser;
use sha3::{Digest, Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use std::io::{self, Read};

mod sha3_impl;
use sha3_impl::{KeccakState, Sha3Variant};

/// Command-line arguments for the SHA-3 visualizer
#[derive(Parser)]
#[command(name = "sha3-visualizer")]
#[command(about = "A real-time 3D visualization of the SHA-3 hash function")]
#[command(long_about = "Visualize the SHA-3 cryptographic hash function step-by-step in 3D. \
Watch as each transformation (Theta, Rho, Pi, Chi, Iota) modifies the 1600-bit \
internal state represented as a 5×5×64 matrix of colored cubes.")]
struct Args {
    /// Input string to hash (if not provided, reads from stdin or uses default)
    input: Option<String>,
}

/// Main entry point for the SHA-3 visualizer application.
/// 
/// Parses command-line arguments, determines input source (CLI, stdin, or default),
/// and initializes the Bevy application with all necessary systems and plugins.
fn main() {
    let args = Args::parse();
    
    // Determine input source: CLI argument, stdin, or default
    let input_string = get_input_string(args.input);

    // Initialize and run the Bevy application
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        .insert_resource(InputString(input_string))
        .insert_resource(SelectedVariant(Sha3Variant::Sha3_256)) // Default to SHA3-256
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, (handle_input, update_animation, update_cubes, update_ui, handle_variant_change))
        .run();
}

/// Determines the input string from various sources.
/// 
/// Priority:
/// 1. Command-line argument if provided
/// 2. Stdin if data is piped in
/// 3. Default "Hello SHA-3!" for interactive terminal sessions
/// 
/// # Arguments
/// 
/// * `cli_input` - Optional input string from command-line arguments
/// 
/// # Returns
/// 
/// The input string to be hashed
fn get_input_string(cli_input: Option<String>) -> String {
    if let Some(input) = cli_input {
        input
    } else if atty::is(atty::Stream::Stdin) {
        // Terminal input, use default
        "Hello SHA-3!".to_string()
    } else {
        // Read from stdin (piped input)
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer.trim().to_string()
    }
}

/// Resource containing the input string to be hashed
#[derive(Resource)]
struct InputString(String);

/// Resource tracking the selected SHA-3 variant
#[derive(Resource)]
struct SelectedVariant(Sha3Variant);

/// Main visualization state resource containing the Keccak algorithm state and UI controls
#[derive(Resource)]
struct KeccakVisualization {
    /// The current state of the SHA-3/Keccak algorithm
    state: KeccakState,
    /// Whether automatic animation is currently playing
    is_playing: bool,
    /// Timer for automatic stepping when animation is enabled
    animation_timer: Timer,
    /// The original input text being hashed
    input_text: String,
    /// Flag indicating whether the hash computation is complete
    is_complete: bool,
    /// The final hash value once computation is complete
    final_hash: String,
}

/// Component marking a cube entity that represents a single bit in the Keccak state
#[derive(Component)]
struct BitCube {
    /// X coordinate in the 5×5 lane grid (0-4)
    lane_x: usize,
    /// Y coordinate representing bit position within lane (0-63)
    bit_y: usize,
    /// Z coordinate in the 5×5 lane grid (0-4)
    lane_z: usize,
}

// UI component markers for text elements

/// Component marking the UI text displaying the current transformation step
#[derive(Component)]
struct StepText;

/// Component marking the UI text displaying the current round number
#[derive(Component)]
struct RoundText;

/// Component marking the UI text displaying the input string
#[derive(Component)]
struct InputText;

/// Component marking the UI text displaying the hash output
#[derive(Component)]
struct OutputText;

/// Component marking the UI text displaying completion status
#[derive(Component)]
struct StatusText;

/// Component marking the UI text displaying SHA-3 capacity/rate information
#[derive(Component)]
struct CapacityText;

/// Sets up the 3D visualization scene including the Keccak state matrix, camera, and lighting.
/// 
/// This function initializes:
/// - The SHA-3/Keccak state with the provided input
/// - 1600 cube entities representing the state matrix (5×5×64 bits)
/// - Camera with orbital controls
/// - Directional lighting for proper cube visibility
/// - Initial materials for active/inactive bit visualization
/// 
/// # Arguments
/// 
/// * `commands` - Bevy commands for spawning entities
/// * `meshes` - Asset collection for storing the cube mesh
/// * `materials` - Asset collection for storing cube materials
/// * `input_string` - The input string to initialize the hash state
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input_string: Res<InputString>,
    selected_variant: Res<SelectedVariant>,
) {
    // Initialize Keccak state with provided input and selected variant
    let mut keccak_state = KeccakState::new_with_variant(selected_variant.0);
    keccak_state.set_input(&input_string.0);

    // Create cube mesh - optimized size to prevent visual collisions
    let cube_mesh = meshes.add(Cuboid::new(0.08, 0.08, 0.08));
    
    // Create initial materials for bit visualization
    let active_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 1.0),
        emissive: bevy::color::LinearRgba::new(0.1, 0.4, 0.5, 1.0),
        metallic: 0.1,
        perceptual_roughness: 0.3,
        ..default()
    });
    let inactive_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.25),
        metallic: 0.05,
        perceptual_roughness: 0.8,
        ..default()
    });

    // Create the 3D matrix of cubes representing the 1600-bit Keccak state
    // Layout: 5x5 lanes, each 64 bits high = 1600 total bits
    for lane_x in 0..5 {
        for lane_z in 0..5 {
            for bit_y in 0..64 {
                // Calculate world position for this bit
                let pos_x = (lane_x as f32 - 2.0) * 0.6;  // Center X around origin
                let pos_y = bit_y as f32 * 0.12;          // Stack bits vertically
                let pos_z = (lane_z as f32 - 2.0) * 0.6;  // Center Z around origin

                // Determine initial material based on bit value
                let is_set = keccak_state.get_bit(lane_x, bit_y, lane_z);
                let material = if is_set {
                    active_material.clone()
                } else {
                    inactive_material.clone()
                };

                // Spawn cube entity with position and bit metadata
                commands.spawn((
                    PbrBundle {
                        mesh: cube_mesh.clone(),
                        material,
                        transform: Transform::from_translation(Vec3::new(pos_x, pos_y, pos_z)),
                        ..default()
                    },
                    BitCube { lane_x, bit_y, lane_z },
                ));
            }
        }
    }

    // Initialize the main visualization state resource
    commands.insert_resource(KeccakVisualization {
        state: keccak_state,
        is_playing: false,
        animation_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        input_text: input_string.0.clone(),
        is_complete: false,
        final_hash: String::new(),
    });

    // Setup camera with orbital controls positioned to view the entire matrix
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(4.0, 6.0, 8.0)
                .looking_at(Vec3::new(0.0, 3.8, 0.0), Vec3::Y), // Focus on matrix center
            ..default()
        },
        PanOrbitCamera {
            focus: Vec3::new(0.0, 3.8, 0.0), // Orbit around vertical center of 64-bit stack
            radius: Some(10.0),
            ..default()
        },
    ));

    // Add directional lighting with shadow support
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4
        )),
        ..default()
    });
}

/// Sets up the user interface overlays including input/output display and control information.
/// 
/// Creates a two-panel UI:
/// - Top panel: Shows input string, hash output, and technical parameters
/// - Bottom panel: Shows current round/step and control instructions
fn setup_ui(mut commands: Commands) {
    // Root UI container
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        // Top panel - Input and Output info
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
            ..default()
        }).with_children(|parent| {
            // Left side - Input
            parent.spawn((
                TextBundle::from_section(
                    "Input: \"Hello SHA-3!\"",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                InputText,
            ));
            
            // Right side - Output and status
            parent.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "Output: (calculating...)",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::srgb(1.0, 0.8, 0.2), // Orange for output
                            ..default()
                        },
                    ),
                    OutputText,
                ));
                parent.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::srgb(0.2, 1.0, 0.2), // Green for completion
                            ..default()
                        },
                    ),
                    StatusText,
                ));
                parent.spawn((
                    TextBundle::from_section(
                        "Capacity: 512 bits | Rate: 1088 bits | Output: 256 bits | SHA3-256",
                        TextStyle {
                            font_size: 14.0,
                            color: Color::srgb(0.7, 0.7, 0.7), // Light gray for technical info
                            ..default()
                        },
                    ),
                    CapacityText,
                ));
            });
        });

        // Bottom panel - Controls and status
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
            ..default()
        }).with_children(|parent| {
            // Left side - Current state
            parent.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "Round: 0",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::srgb(0.2, 0.8, 1.0),
                            ..default()
                        },
                    ),
                    RoundText,
                ));
                parent.spawn((
                    TextBundle::from_section(
                        "Step: Theta - Column mixing",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::srgb(0.2, 0.8, 1.0),
                            ..default()
                        },
                    ),
                    StepText,
                ));
            });

            // Right side - Controls and color legend
            parent.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "ENTER: Step | P: Play/Pause | R: Reset | F: Fast-Forward | 1-4: Change Variant | Q: Quit",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.8, 0.8, 0.8),
                        ..default()
                    },
                ));
                parent.spawn(TextBundle::from_section(
                    "Colors: Red=Theta | Green=Rho | Blue=Pi | Magenta=Chi | Yellow=Iota",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::srgb(0.6, 0.6, 0.6),
                        ..default()
                    },
                ));
            });
        });
    });
}

/// Updates all UI text elements to reflect the current state of the SHA-3 computation.
/// 
/// This system runs every frame and updates:
/// - Input text display (truncated if too long)
/// - Hash output (current or final)
/// - Completion status
/// - Technical parameters (capacity, rate, output length)
/// - Current round and step information
/// 
/// Only updates when the visualization state has changed for performance.
fn update_ui(
    visualization: Res<KeccakVisualization>,
    mut round_query: Query<&mut Text, (With<RoundText>, Without<StepText>, Without<InputText>, Without<OutputText>, Without<StatusText>, Without<CapacityText>)>,
    mut step_query: Query<&mut Text, (With<StepText>, Without<RoundText>, Without<InputText>, Without<OutputText>, Without<StatusText>, Without<CapacityText>)>,
    mut input_query: Query<&mut Text, (With<InputText>, Without<StepText>, Without<RoundText>, Without<OutputText>, Without<StatusText>, Without<CapacityText>)>,
    mut output_query: Query<&mut Text, (With<OutputText>, Without<StepText>, Without<RoundText>, Without<InputText>, Without<StatusText>, Without<CapacityText>)>,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<StepText>, Without<RoundText>, Without<InputText>, Without<OutputText>, Without<CapacityText>)>,
    mut capacity_query: Query<&mut Text, (With<CapacityText>, Without<StepText>, Without<RoundText>, Without<InputText>, Without<OutputText>, Without<StatusText>)>,
) {
    // Only update UI when visualization state changes for performance
    if !visualization.is_changed() {
        return;
    }

    // Update input text display with truncation for long inputs
    if let Ok(mut text) = input_query.get_single_mut() {
        text.sections[0].value = format!("Input: \"{}\"", 
            if visualization.input_text.len() > 50 {
                format!("{}...", &visualization.input_text[..50])
            } else {
                visualization.input_text.clone()
            }
        );
    }

    // Update output text (current hash state)
    if let Ok(mut text) = output_query.get_single_mut() {
        if visualization.is_complete && !visualization.final_hash.is_empty() {
            text.sections[0].value = format!("Final Hash: {}", visualization.final_hash);
        } else {
            let current_hash = visualization.state.get_output_hex();
            text.sections[0].value = format!("Output: {}", 
                if current_hash.len() > 32 {
                    format!("{}...", &current_hash[..32])
                } else {
                    current_hash
                }
            );
        }
    }

    // Update status text
    if let Ok(mut text) = status_query.get_single_mut() {
        if visualization.is_complete {
            text.sections[0].value = "✓ HASH COMPLETE - Copy from terminal".to_string();
        } else {
            text.sections[0].value = "Computing...".to_string();
        }
    }

    // Update capacity info (only needs to be set once, but we'll update it for completeness)
    if let Ok(mut text) = capacity_query.get_single_mut() {
        text.sections[0].value = format!(
            "Capacity: {} bits | Rate: {} bits | Output: {} bits | {}",
            visualization.state.get_capacity(),
            visualization.state.get_rate(),
            visualization.state.get_output_length(),
            visualization.state.get_variant().name()
        );
    }

    // Update round text
    if let Ok(mut text) = round_query.get_single_mut() {
        if visualization.state.is_complete {
            text.sections[0].value = "Round: 24/24".to_string();
        } else {
            text.sections[0].value = format!("Round: {}/24", visualization.state.round + 1);
        }
    }

    // Update step text with descriptions
    if let Ok(mut text) = step_query.get_single_mut() {
        if visualization.state.is_complete {
            text.sections[0].value = "Step: Complete - Final hash ready".to_string();
        } else {
            let (step_name, description) = match visualization.state.step {
                0 => ("Theta", "Column mixing"),
                1 => ("Rho", "Bit rotation"),
                2 => ("Pi", "Lane rearrangement"),
                3 => ("Chi", "Non-linear transformation"),
                4 => ("Iota", "Round constant addition"),
                _ => ("Unknown", ""),
            };
            text.sections[0].value = format!("Step: {} - {}", step_name, description);
        }
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut visualization: ResMut<KeccakVisualization>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        println!("Quitting SHA-3 Visualizer...");
        exit.send(bevy::app::AppExit::Success);
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Enter) {
        
        visualization.state.step();
        
        // Check if we just completed the algorithm
        if visualization.state.is_complete && !visualization.is_complete {
            visualization.is_complete = true;
            visualization.final_hash = visualization.state.get_output_hex();
            
            // Verify against standard library
            verify_hash_with_variant(&visualization.final_hash, &visualization.input_text, visualization.state.get_variant());
        }
        
        println!("SHA-3 step: Round {}, Step {}", visualization.state.round, visualization.state.step);
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        let input_text = visualization.input_text.clone();
        visualization.state = KeccakState::new();
        visualization.state.set_input(&input_text);
        visualization.is_complete = false;
        visualization.final_hash.clear();
        println!("Reset SHA-3 state");
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        visualization.is_playing = !visualization.is_playing;
        println!("Animation {}", if visualization.is_playing { "playing" } else { "paused" });
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        println!("Fast-forwarding to completion...");
        
        // Run all steps until complete
        while !visualization.state.is_complete {
            visualization.state.step();
        }
        
        // Set completion state
        if !visualization.is_complete {
            visualization.is_complete = true;
            visualization.final_hash = visualization.state.get_output_hex();
            
            // Verify against standard library
            verify_hash_with_variant(&visualization.final_hash, &visualization.input_text, visualization.state.get_variant());
        }
        
        println!("Fast-forward complete! Final state: Round {}, Step {}", 
                visualization.state.round, visualization.state.step);
    }
}

fn update_animation(
    time: Res<Time>,
    mut visualization: ResMut<KeccakVisualization>,
) {
    if visualization.is_playing && !visualization.is_complete {
        visualization.animation_timer.tick(time.delta());
        if visualization.animation_timer.just_finished() {
            let prev_round = visualization.state.round;
            let prev_step = visualization.state.step;
            
            visualization.state.step();
            
            // Check if we just completed the algorithm (finished round 23, step 4)
            if prev_round == 23 && prev_step == 4 && visualization.state.round == 0 {
                visualization.is_complete = true;
                visualization.final_hash = visualization.state.get_output_hex();
                visualization.is_playing = false; // Stop animation when complete
                
                // Verify against standard library
                let mut hasher = Sha3_256::new();
                hasher.update(visualization.input_text.as_bytes());
                let expected = format!("{:x}", hasher.finalize());
                
                println!("SHA-3 COMPLETE!");
                println!("Our hash:      {}", visualization.final_hash);
                println!("Expected hash: {}", expected);
                if visualization.final_hash == expected {
                    println!("✓ HASH VERIFICATION PASSED!");
                } else {
                    println!("✗ Hash verification failed - implementation may have bugs");
                }
            }
        }
    }
}

fn update_cubes(
    visualization: Res<KeccakVisualization>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Handle<StandardMaterial>, &BitCube)>,
) {
    if !visualization.is_changed() {
        return;
    }

    // Create color-coded materials for the current transformation step
    let active_material = create_step_material(&mut materials, visualization.state.step);
    
    let inactive_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.25), // Dark blue-gray for inactive bits
        metallic: 0.05,
        perceptual_roughness: 0.8,
        ..default()
    });

    for (mut material_handle, bit_cube) in query.iter_mut() {
        let is_set = visualization.state.get_bit(bit_cube.lane_x, bit_cube.bit_y, bit_cube.lane_z);
        *material_handle = if is_set {
            active_material.clone()
        } else {
            inactive_material.clone()
        };
    }
}

/// Creates a color-coded material for the specified transformation step.
/// 
/// # Color Scheme
/// - Step 0 (Theta): Red - Column mixing operations
/// - Step 1 (Rho): Green - Bit rotation within lanes  
/// - Step 2 (Pi): Blue - Lane rearrangement
/// - Step 3 (Chi): Magenta - Non-linear transformation
/// - Step 4 (Iota): Yellow - Round constant addition
/// - Default: Cyan - Initial or unknown state
/// 
/// # Arguments
/// 
/// * `materials` - Mutable reference to the material asset collection
/// * `step` - Current transformation step (0-4)
/// 
/// # Returns
/// 
/// Handle to the created material with appropriate color and properties
fn create_step_material(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    step: usize,
) -> Handle<StandardMaterial> {
    let (base_color, emissive) = match step {
        0 => (Color::srgb(1.0, 0.3, 0.3), bevy::color::LinearRgba::new(0.3, 0.1, 0.1, 1.0)), // Red for Theta
        1 => (Color::srgb(0.3, 1.0, 0.3), bevy::color::LinearRgba::new(0.1, 0.3, 0.1, 1.0)), // Green for Rho
        2 => (Color::srgb(0.3, 0.3, 1.0), bevy::color::LinearRgba::new(0.1, 0.1, 0.3, 1.0)), // Blue for Pi
        3 => (Color::srgb(1.0, 0.3, 1.0), bevy::color::LinearRgba::new(0.3, 0.1, 0.3, 1.0)), // Magenta for Chi
        4 => (Color::srgb(1.0, 1.0, 0.3), bevy::color::LinearRgba::new(0.3, 0.3, 0.1, 1.0)), // Yellow for Iota
        _ => (Color::srgb(0.2, 0.8, 1.0), bevy::color::LinearRgba::new(0.1, 0.4, 0.5, 1.0)), // Default cyan
    };

    materials.add(StandardMaterial {
        base_color,
        emissive,
        metallic: 0.1,
        perceptual_roughness: 0.3,
        ..default()
    })
}

/// Handles variant switching via keyboard input (1-4 keys)
fn handle_variant_change(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected_variant: ResMut<SelectedVariant>,
    mut visualization: ResMut<KeccakVisualization>,
) {
    let new_variant = if keyboard_input.just_pressed(KeyCode::Digit1) {
        Some(Sha3Variant::Sha3_224)
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        Some(Sha3Variant::Sha3_256)
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        Some(Sha3Variant::Sha3_384)
    } else if keyboard_input.just_pressed(KeyCode::Digit4) {
        Some(Sha3Variant::Sha3_512)
    } else {
        None
    };

    if let Some(variant) = new_variant {
        if variant != selected_variant.0 {
            selected_variant.0 = variant;
            
            // Reset the visualization with the new variant
            let input_text = visualization.input_text.clone();
            visualization.state = KeccakState::new_with_variant(variant);
            visualization.state.set_input(&input_text);
            visualization.is_complete = false;
            visualization.final_hash.clear();
            
            println!("Switched to {} - Variant changed, state reset", variant.name());
        }
    }
}

/// Verifies the computed hash against the standard library implementation for the given variant
fn verify_hash_with_variant(our_hash: &str, input: &str, variant: Sha3Variant) -> bool {
    let expected = match variant {
        Sha3Variant::Sha3_224 => {
            let mut hasher = Sha3_224::new();
            hasher.update(input.as_bytes());
            format!("{:x}", hasher.finalize())
        },
        Sha3Variant::Sha3_256 => {
            let mut hasher = Sha3_256::new();
            hasher.update(input.as_bytes());
            format!("{:x}", hasher.finalize())
        },
        Sha3Variant::Sha3_384 => {
            let mut hasher = Sha3_384::new();
            hasher.update(input.as_bytes());
            format!("{:x}", hasher.finalize())
        },
        Sha3Variant::Sha3_512 => {
            let mut hasher = Sha3_512::new();
            hasher.update(input.as_bytes());
            format!("{:x}", hasher.finalize())
        },
    };

    println!("SHA-3 COMPLETE!");
    println!("Our hash:      {}", our_hash);
    println!("Expected hash: {}", expected);
    
    let matches = our_hash == expected;
    if matches {
        println!("✓ HASH VERIFICATION PASSED!");
    } else {
        println!("✗ Hash verification failed - implementation may have bugs");
    }
    
    matches
}

