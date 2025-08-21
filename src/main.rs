use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use clap::Parser;
use sha3::{Digest, Sha3_256};
use std::io::{self, Read};

mod sha3_impl;
use sha3_impl::KeccakState;

#[derive(Parser)]
#[command(name = "sha3-visualizer")]
#[command(about = "A real-time 3D visualization of the SHA-3 hash function")]
struct Args {
    /// Input string to hash (if not provided, reads from stdin or uses default)
    input: Option<String>,
}

fn main() {
    let args = Args::parse();
    
    // Get input string from command line, stdin, or default
    let input_string = if let Some(input) = args.input {
        input
    } else if atty::is(atty::Stream::Stdin) {
        // Terminal input, use default
        "Hello SHA-3!".to_string()
    } else {
        // Read from stdin (piped input)
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Failed to read from stdin");
        buffer.trim().to_string()
    };

    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        .insert_resource(InputString(input_string))
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, (handle_input, update_animation, update_cubes, update_ui))
        .run();
}

#[derive(Resource)]
struct InputString(String);

#[derive(Resource)]
struct KeccakVisualization {
    state: KeccakState,
    is_playing: bool,
    animation_timer: Timer,
    input_text: String,
    is_complete: bool,
    final_hash: String,
}

#[derive(Component)]
struct BitCube {
    lane_x: usize,
    bit_y: usize,
    lane_z: usize,
}

#[derive(Component)]
struct StepText;

#[derive(Component)]
struct RoundText;

#[derive(Component)]
struct InputText;

#[derive(Component)]
struct OutputText;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct CapacityText;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input_string: Res<InputString>,
) {
    // Initialize Keccak state with provided input
    let mut keccak_state = KeccakState::new();
    keccak_state.set_input(&input_string.0);
    
    // Manually set some bits for visualization
    for i in 0..5 {
        for j in 0..5 {
            for k in 0..10 {
                if (i + j + k) % 3 == 0 {
                    let lane_idx = i + 5 * j;
                    if lane_idx < 25 {
                        keccak_state.state[lane_idx] |= 1u64 << k;
                    }
                }
            }
        }
    }

    // Create cube mesh and materials - smaller cubes to prevent collisions
    let cube_mesh = meshes.add(Cuboid::new(0.08, 0.08, 0.08));
    
    // Materials are created dynamically in update_cubes function
    
    // Default active material (cyan)
    let active_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 1.0), // Bright cyan for active bits
        emissive: bevy::color::LinearRgba::new(0.1, 0.4, 0.5, 1.0),
        metallic: 0.1,
        perceptual_roughness: 0.3,
        ..default()
    });
    let inactive_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.25), // Dark blue-gray for inactive bits
        metallic: 0.05,
        perceptual_roughness: 0.8,
        ..default()
    });

    // Create cubes for the 5x5x64 matrix
    for lane_x in 0..5 {
        for lane_z in 0..5 {
            for bit_y in 0..64 {
                let pos_x = (lane_x as f32 - 2.0) * 0.6;  // Perfect spacing for 5 lanes
                let pos_y = bit_y as f32 * 0.12;         // Perfect spacing for 64 bits  
                let pos_z = (lane_z as f32 - 2.0) * 0.6;

                let is_set = keccak_state.get_bit(lane_x, bit_y, lane_z);
                // Start with default cyan color
                let material = if is_set {
                    active_material.clone()
                } else {
                    inactive_material.clone()
                };

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

    // Add visualization resource
    commands.insert_resource(KeccakVisualization {
        state: keccak_state,
        is_playing: false,
        animation_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        input_text: input_string.0.clone(),
        is_complete: false,
        final_hash: String::new(),
    });

    // Add camera with pan/orbit controls - positioned to see full matrix
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(4.0, 6.0, 8.0)
                .looking_at(Vec3::new(0.0, 3.8, 0.0), Vec3::Y), // Look at center of 64-bit tall matrix
            ..default()
        },
        PanOrbitCamera {
            focus: Vec3::new(0.0, 3.8, 0.0), // Orbit around matrix center
            radius: Some(10.0),
            ..default()
        },
    ));

    // Add lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -std::f32::consts::FRAC_PI_4)),
        ..default()
    });
}

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
                        "Capacity: 512 bits | Rate: 1088 bits | SHA3-256",
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
                    "ENTER: Step | P: Play/Pause | R: Reset",
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

fn update_ui(
    visualization: Res<KeccakVisualization>,
    mut round_query: Query<&mut Text, (With<RoundText>, Without<StepText>, Without<InputText>, Without<OutputText>, Without<StatusText>, Without<CapacityText>)>,
    mut step_query: Query<&mut Text, (With<StepText>, Without<RoundText>, Without<InputText>, Without<OutputText>, Without<StatusText>, Without<CapacityText>)>,
    mut input_query: Query<&mut Text, (With<InputText>, Without<StepText>, Without<RoundText>, Without<OutputText>, Without<StatusText>, Without<CapacityText>)>,
    mut output_query: Query<&mut Text, (With<OutputText>, Without<StepText>, Without<RoundText>, Without<InputText>, Without<StatusText>, Without<CapacityText>)>,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<StepText>, Without<RoundText>, Without<InputText>, Without<OutputText>, Without<CapacityText>)>,
    mut capacity_query: Query<&mut Text, (With<CapacityText>, Without<StepText>, Without<RoundText>, Without<InputText>, Without<OutputText>, Without<StatusText>)>,
) {
    if !visualization.is_changed() {
        return;
    }

    // Update input text
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
            "Capacity: {} bits | Rate: {} bits | Output: {} bits",
            visualization.state.get_capacity(),
            visualization.state.get_rate(),
            visualization.state.get_output_length()
        );
    }

    // Update round text
    if let Ok(mut text) = round_query.get_single_mut() {
        text.sections[0].value = format!("Round: {}/24", visualization.state.round);
    }

    // Update step text with descriptions
    if let Ok(mut text) = step_query.get_single_mut() {
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

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut visualization: ResMut<KeccakVisualization>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        let prev_round = visualization.state.round;
        let prev_step = visualization.state.step;
        
        visualization.state.step();
        
        // Check if we just completed the algorithm (finished round 23, step 4)
        if prev_round == 23 && prev_step == 4 && visualization.state.round == 0 {
            visualization.is_complete = true;
            visualization.final_hash = visualization.state.get_output_hex();
            
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

    // Create step-specific materials
    let (active_material, _step_name) = match visualization.state.step {
        0 => (materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.3, 0.3), // Red for Theta
            emissive: bevy::color::LinearRgba::new(0.3, 0.1, 0.1, 1.0),
            metallic: 0.1,
            perceptual_roughness: 0.3,
            ..default()
        }), "Theta"),
        1 => (materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 1.0, 0.3), // Green for Rho
            emissive: bevy::color::LinearRgba::new(0.1, 0.3, 0.1, 1.0),
            metallic: 0.1,
            perceptual_roughness: 0.3,
            ..default()
        }), "Rho"),
        2 => (materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 1.0), // Blue for Pi
            emissive: bevy::color::LinearRgba::new(0.1, 0.1, 0.3, 1.0),
            metallic: 0.1,
            perceptual_roughness: 0.3,
            ..default()
        }), "Pi"),
        3 => (materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.3, 1.0), // Magenta for Chi
            emissive: bevy::color::LinearRgba::new(0.3, 0.1, 0.3, 1.0),
            metallic: 0.1,
            perceptual_roughness: 0.3,
            ..default()
        }), "Chi"),
        4 => (materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.3), // Yellow for Iota
            emissive: bevy::color::LinearRgba::new(0.3, 0.3, 0.1, 1.0),
            metallic: 0.1,
            perceptual_roughness: 0.3,
            ..default()
        }), "Iota"),
        _ => (materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 1.0), // Default cyan
            emissive: bevy::color::LinearRgba::new(0.1, 0.4, 0.5, 1.0),
            metallic: 0.1,
            perceptual_roughness: 0.3,
            ..default()
        }), "Default"),
    };
    
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