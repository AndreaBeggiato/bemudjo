//! Resource Lifecycle Integration Tests
//!
//! Tests focused on resource creation, modification, removal,
//! and lifecycle management in the ECS world.

use bemudjo_ecs::{Component, World};

// Test Resource Types
#[derive(Debug, Clone, PartialEq)]
struct GameTime {
    elapsed: f64,
    delta: f32,
    frame_count: u64,
}
impl Component for GameTime {}

#[derive(Debug, Clone, PartialEq)]
struct GameConfig {
    difficulty: u8,
    volume: f32,
    debug_mode: bool,
}
impl Component for GameConfig {}

#[derive(Debug, Clone, PartialEq)]
struct PlayerStats {
    score: u64,
    lives: u32,
    level: u32,
}
impl Component for PlayerStats {}

#[derive(Debug, Clone, PartialEq)]
struct InputState {
    keys_pressed: Vec<String>,
    mouse_x: f32,
    mouse_y: f32,
    mouse_buttons: u8,
}
impl Component for InputState {}

#[derive(Debug, Clone, PartialEq)]
struct NetworkInfo {
    connected: bool,
    player_count: u32,
    latency: u32,
}
impl Component for NetworkInfo {}

#[derive(Debug, Clone, PartialEq)]
struct RenderSettings {
    resolution_width: u32,
    resolution_height: u32,
    vsync: bool,
    fullscreen: bool,
}
impl Component for RenderSettings {}

#[derive(Debug, Clone, PartialEq)]
struct AudioSettings {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    muted: bool,
}
impl Component for AudioSettings {}

#[test]
fn test_basic_resource_lifecycle() {
    let mut world = World::new();

    // Initially no resources
    assert!(!world.has_resource::<GameTime>());
    assert!(world.get_resource::<GameTime>().is_none());

    // Insert resource
    world.insert_resource(GameTime {
        elapsed: 0.0,
        delta: 0.016,
        frame_count: 0,
    });

    assert!(world.has_resource::<GameTime>());
    let time = world.get_resource::<GameTime>().unwrap();
    assert_eq!(time.elapsed, 0.0);
    assert_eq!(time.delta, 0.016);
    assert_eq!(time.frame_count, 0); // Update resource
    let updated_time = world
        .update_resource::<GameTime, _>(|mut time| {
            // Better approach: do arithmetic in f64 to avoid precision loss
            time.elapsed += time.delta as f64;
            time.frame_count += 1;
            time
        })
        .unwrap();

    // With proper f64 arithmetic, we should get exact equality
    // The delta value 0.016 is exactly representable in both f32 and f64
    let expected_elapsed = 0.0_f64 + 0.016_f32 as f64;
    assert_eq!(updated_time.elapsed, expected_elapsed);
    assert_eq!(updated_time.frame_count, 1);

    // Verify resource was updated in world
    let current_elapsed = {
        let current_time = world.get_resource::<GameTime>().unwrap();
        assert_eq!(current_time.elapsed, expected_elapsed);
        assert_eq!(current_time.frame_count, 1);
        current_time.elapsed
    };

    // Remove resource
    let removed_time = world.remove_resource::<GameTime>();
    let expected_time = GameTime {
        elapsed: current_elapsed, // Use the actual value to avoid precision issues
        delta: 0.016,
        frame_count: 1,
    };
    assert_eq!(removed_time, Some(expected_time));

    assert!(!world.has_resource::<GameTime>());
    assert!(world.get_resource::<GameTime>().is_none());

    // Try to remove again (should return None)
    let removed_again = world.remove_resource::<GameTime>();
    assert_eq!(removed_again, None);
}

#[test]
fn test_multiple_resources_lifecycle() {
    let mut world = World::new();

    // Insert multiple resources
    world.insert_resource(GameConfig {
        difficulty: 2,
        volume: 0.8,
        debug_mode: false,
    });

    world.insert_resource(PlayerStats {
        score: 1000,
        lives: 3,
        level: 5,
    });

    world.insert_resource(InputState {
        keys_pressed: vec!["W".to_string(), "A".to_string()],
        mouse_x: 100.0,
        mouse_y: 200.0,
        mouse_buttons: 1,
    });

    // Verify all resources exist
    assert!(world.has_resource::<GameConfig>());
    assert!(world.has_resource::<PlayerStats>());
    assert!(world.has_resource::<InputState>());

    // Update multiple resources
    world
        .update_resource::<PlayerStats, _>(|mut stats| {
            stats.score += 500;
            stats.level += 1;
            stats
        })
        .unwrap();

    world
        .update_resource::<GameConfig, _>(|mut config| {
            config.difficulty = 3;
            config.debug_mode = true;
            config
        })
        .unwrap();

    // Verify updates
    let stats = world.get_resource::<PlayerStats>().unwrap();
    assert_eq!(stats.score, 1500);
    assert_eq!(stats.level, 6);

    let config = world.get_resource::<GameConfig>().unwrap();
    assert_eq!(config.difficulty, 3);
    assert!(config.debug_mode);

    // Remove one resource
    world.remove_resource::<InputState>();

    // Verify others remain
    assert!(world.has_resource::<GameConfig>());
    assert!(world.has_resource::<PlayerStats>());
    assert!(!world.has_resource::<InputState>());

    // Replace existing resource
    world.insert_resource(PlayerStats {
        score: 0,
        lives: 5,
        level: 1,
    });

    let new_stats = world.get_resource::<PlayerStats>().unwrap();
    assert_eq!(new_stats.score, 0);
    assert_eq!(new_stats.lives, 5);
    assert_eq!(new_stats.level, 1);
}

#[test]
fn test_resource_update_error_handling() {
    let mut world = World::new();

    // Try to update non-existent resource
    let result = world.update_resource::<GameTime, _>(|mut time| {
        time.frame_count += 1;
        time
    });

    assert!(result.is_err());

    // Insert resource and try again
    world.insert_resource(GameTime {
        elapsed: 0.0,
        delta: 0.016,
        frame_count: 0,
    });

    let result = world.update_resource::<GameTime, _>(|mut time| {
        time.frame_count += 1;
        time
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap().frame_count, 1);
}

#[test]
fn test_resource_replacement_patterns() {
    let mut world = World::new();

    // Insert initial resource
    world.insert_resource(RenderSettings {
        resolution_width: 1920,
        resolution_height: 1080,
        vsync: true,
        fullscreen: false,
    });

    // Replace with new settings
    world.insert_resource(RenderSettings {
        resolution_width: 2560,
        resolution_height: 1440,
        vsync: false,
        fullscreen: true,
    });

    let settings = world.get_resource::<RenderSettings>().unwrap();
    assert_eq!(settings.resolution_width, 2560);
    assert_eq!(settings.resolution_height, 1440);
    assert!(!settings.vsync);
    assert!(settings.fullscreen);

    // Update existing resource
    world
        .update_resource::<RenderSettings, _>(|mut settings| {
            settings.vsync = true;
            settings
        })
        .unwrap();

    let updated_settings = world.get_resource::<RenderSettings>().unwrap();
    assert!(updated_settings.vsync);
    assert_eq!(updated_settings.resolution_width, 2560); // Other fields unchanged
}

#[test]
fn test_resource_lifecycle_with_complex_data() {
    let mut world = World::new();

    // Insert resource with complex data
    world.insert_resource(InputState {
        keys_pressed: vec![
            "W".to_string(),
            "A".to_string(),
            "S".to_string(),
            "D".to_string(),
            "Space".to_string(),
        ],
        mouse_x: 512.5,
        mouse_y: 384.2,
        mouse_buttons: 0b101, // Left and right buttons
    });

    // Update complex data
    world
        .update_resource::<InputState, _>(|mut input| {
            input.keys_pressed.push("Shift".to_string());
            input.keys_pressed.retain(|key| key != "S"); // Remove S key
            input.mouse_x += 10.0;
            input.mouse_buttons |= 0b010; // Add middle button
            input
        })
        .unwrap();

    let input = world.get_resource::<InputState>().unwrap();
    assert_eq!(input.keys_pressed.len(), 5); // W, A, D, Space, Shift
    assert!(input.keys_pressed.contains(&"Shift".to_string()));
    assert!(!input.keys_pressed.contains(&"S".to_string()));
    assert_eq!(input.mouse_x, 522.5);
    assert_eq!(input.mouse_buttons, 0b111); // All three buttons

    // Clear and rebuild
    world
        .update_resource::<InputState, _>(|mut input| {
            input.keys_pressed.clear();
            input.mouse_buttons = 0;
            input
        })
        .unwrap();

    let cleared_input = world.get_resource::<InputState>().unwrap();
    assert!(cleared_input.keys_pressed.is_empty());
    assert_eq!(cleared_input.mouse_buttons, 0);
}

#[test]
fn test_resource_lifecycle_stress() {
    let mut world = World::new();

    // Create and destroy resources in cycles
    for cycle in 0..100 {
        // Insert multiple resources
        world.insert_resource(GameTime {
            elapsed: cycle as f64,
            delta: 0.016,
            frame_count: cycle as u64,
        });

        world.insert_resource(PlayerStats {
            score: (cycle * 100) as u64,
            lives: 3,
            level: (cycle / 10) as u32,
        });

        world.insert_resource(NetworkInfo {
            connected: cycle % 2 == 0,
            player_count: (cycle % 8) as u32,
            latency: (cycle * 5) as u32,
        });

        // Update resources
        world
            .update_resource::<GameTime, _>(|mut time| {
                time.frame_count += 1;
                time
            })
            .unwrap();

        world
            .update_resource::<PlayerStats, _>(|mut stats| {
                stats.score += 50;
                stats
            })
            .unwrap();

        // Conditionally remove resources
        if cycle % 3 == 0 {
            world.remove_resource::<NetworkInfo>();
        }

        if cycle % 5 == 0 {
            world.remove_resource::<PlayerStats>();
        }

        // Verify state
        assert!(world.has_resource::<GameTime>());

        if cycle % 5 != 0 {
            assert!(world.has_resource::<PlayerStats>());
        }

        if cycle % 3 != 0 {
            assert!(world.has_resource::<NetworkInfo>());
        }
    }

    // Final cleanup
    world.remove_resource::<GameTime>();
    world.remove_resource::<PlayerStats>();
    world.remove_resource::<NetworkInfo>();

    assert!(!world.has_resource::<GameTime>());
    assert!(!world.has_resource::<PlayerStats>());
    assert!(!world.has_resource::<NetworkInfo>());
}

#[test]
fn test_resource_independence() {
    let mut world = World::new();

    // Insert different resource types
    world.insert_resource(GameConfig {
        difficulty: 1,
        volume: 0.5,
        debug_mode: false,
    });

    world.insert_resource(AudioSettings {
        master_volume: 1.0,
        music_volume: 0.8,
        sfx_volume: 0.9,
        muted: false,
    });

    world.insert_resource(RenderSettings {
        resolution_width: 1920,
        resolution_height: 1080,
        vsync: true,
        fullscreen: false,
    });

    // Update one resource
    world
        .update_resource::<GameConfig, _>(|mut config| {
            config.difficulty = 5;
            config.debug_mode = true;
            config
        })
        .unwrap();

    // Verify others are unchanged
    let audio = world.get_resource::<AudioSettings>().unwrap();
    assert_eq!(audio.master_volume, 1.0);
    assert!(!audio.muted);

    let render = world.get_resource::<RenderSettings>().unwrap();
    assert_eq!(render.resolution_width, 1920);
    assert!(render.vsync);

    // Remove one resource
    world.remove_resource::<AudioSettings>();

    // Verify others still exist
    assert!(world.has_resource::<GameConfig>());
    assert!(!world.has_resource::<AudioSettings>());
    assert!(world.has_resource::<RenderSettings>());

    let config = world.get_resource::<GameConfig>().unwrap();
    assert_eq!(config.difficulty, 5);
    assert!(config.debug_mode);
}

#[test]
fn test_resource_state_consistency() {
    let mut world = World::new();

    // Insert initial state
    world.insert_resource(PlayerStats {
        score: 0,
        lives: 3,
        level: 1,
    });

    world.insert_resource(GameTime {
        elapsed: 0.0,
        delta: 0.016,
        frame_count: 0,
    });

    // Simulate game loop with consistent updates
    for frame in 1..=1000 {
        // Update time
        world
            .update_resource::<GameTime, _>(|mut time| {
                time.elapsed += time.delta as f64;
                time.frame_count += 1;
                time
            })
            .unwrap();

        // Update score based on time
        if frame % 60 == 0 {
            // Every second (60 FPS)
            world
                .update_resource::<PlayerStats, _>(|mut stats| {
                    stats.score += 100;
                    if stats.score % 1000 == 0 {
                        stats.level += 1;
                    }
                    stats
                })
                .unwrap();
        }

        // Verify consistency
        let time = world.get_resource::<GameTime>().unwrap();
        let stats = world.get_resource::<PlayerStats>().unwrap();

        assert_eq!(time.frame_count, frame);
        assert!((time.elapsed - (frame as f64 * 0.016)).abs() < 0.001);

        let expected_score = ((frame / 60) * 100) as u64;
        assert_eq!(stats.score, expected_score);

        let expected_level = (1 + (expected_score / 1000)) as u32;
        assert_eq!(stats.level, expected_level);
    }
}

#[test]
fn test_resource_cloning_and_ownership() {
    let mut world = World::new();

    // Insert resource
    world.insert_resource(InputState {
        keys_pressed: vec!["A".to_string(), "B".to_string()],
        mouse_x: 100.0,
        mouse_y: 200.0,
        mouse_buttons: 1,
    });

    // Get reference and clone
    let input_ref = world.get_resource::<InputState>().unwrap();
    let input_clone = input_ref.clone();

    assert_eq!(input_ref.keys_pressed, input_clone.keys_pressed);
    assert_eq!(input_ref.mouse_x, input_clone.mouse_x);

    // Update original
    world
        .update_resource::<InputState, _>(|mut input| {
            input.keys_pressed.push("C".to_string());
            input.mouse_x = 150.0;
            input
        })
        .unwrap();

    // Clone should be unchanged
    assert_eq!(input_clone.keys_pressed.len(), 2);
    assert_eq!(input_clone.mouse_x, 100.0);

    // New reference should show changes
    let updated_input = world.get_resource::<InputState>().unwrap();
    assert_eq!(updated_input.keys_pressed.len(), 3);
    assert_eq!(updated_input.mouse_x, 150.0);
}

#[test]
fn test_resource_type_safety() {
    let mut world = World::new();

    // Insert different types with similar data
    world.insert_resource(GameConfig {
        difficulty: 5,
        volume: 0.8,
        debug_mode: true,
    });

    world.insert_resource(AudioSettings {
        master_volume: 0.8, // Same value as GameConfig.volume
        music_volume: 0.7,
        sfx_volume: 0.9,
        muted: false,
    });

    // Each type should be independent
    let config = world.get_resource::<GameConfig>().unwrap();
    let audio = world.get_resource::<AudioSettings>().unwrap();

    assert_eq!(config.volume, 0.8);
    assert_eq!(audio.master_volume, 0.8);

    // Update one type
    world
        .update_resource::<GameConfig, _>(|mut config| {
            config.volume = 0.5;
            config
        })
        .unwrap();

    // Other type should be unchanged
    let updated_config = world.get_resource::<GameConfig>().unwrap();
    let unchanged_audio = world.get_resource::<AudioSettings>().unwrap();

    assert_eq!(updated_config.volume, 0.5);
    assert_eq!(unchanged_audio.master_volume, 0.8); // Unchanged

    // Remove one type
    world.remove_resource::<GameConfig>();

    // Other should still exist
    assert!(!world.has_resource::<GameConfig>());
    assert!(world.has_resource::<AudioSettings>());
}

#[test]
fn test_resource_large_data_lifecycle() {
    let mut world = World::new();

    // Insert resource with large data
    let large_keys: Vec<String> = (0..10000).map(|i| format!("Key{}", i)).collect();

    world.insert_resource(InputState {
        keys_pressed: large_keys.clone(),
        mouse_x: 0.0,
        mouse_y: 0.0,
        mouse_buttons: 0,
    });

    // Verify large data was stored correctly
    let input = world.get_resource::<InputState>().unwrap();
    assert_eq!(input.keys_pressed.len(), 10000);
    assert_eq!(input.keys_pressed[0], "Key0");
    assert_eq!(input.keys_pressed[9999], "Key9999");

    // Update large data
    world
        .update_resource::<InputState, _>(|mut input| {
            input.keys_pressed.reverse();
            input
        })
        .unwrap();

    let updated_input = world.get_resource::<InputState>().unwrap();
    assert_eq!(updated_input.keys_pressed[0], "Key9999");
    assert_eq!(updated_input.keys_pressed[9999], "Key0");

    // Remove large resource
    let removed = world.remove_resource::<InputState>();
    assert!(removed.is_some());

    let removed_input = removed.unwrap();
    assert_eq!(removed_input.keys_pressed.len(), 10000);
    assert_eq!(removed_input.keys_pressed[0], "Key9999");
}

#[test]
fn test_resource_update_return_values() {
    let mut world = World::new();

    world.insert_resource(PlayerStats {
        score: 100,
        lives: 3,
        level: 1,
    });

    // Update and capture return value
    let result1 = world
        .update_resource::<PlayerStats, _>(|mut stats| {
            stats.score += 50;
            stats
        })
        .unwrap();

    assert_eq!(result1.score, 150);
    assert_eq!(result1.lives, 3);
    assert_eq!(result1.level, 1);

    // Chain updates using return values
    let result2 = world
        .update_resource::<PlayerStats, _>(|mut stats| {
            if stats.score >= 150 {
                stats.level += 1;
                stats.score = 0; // Reset score for next level
            }
            stats
        })
        .unwrap();

    assert_eq!(result2.score, 0);
    assert_eq!(result2.level, 2);

    // Verify world state matches return value
    let current_stats = world.get_resource::<PlayerStats>().unwrap();
    assert_eq!(current_stats.score, result2.score);
    assert_eq!(current_stats.level, result2.level);
}
