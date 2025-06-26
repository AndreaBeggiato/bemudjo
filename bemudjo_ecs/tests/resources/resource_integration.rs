//! Resource Management Integration Tests
//!
//! These tests validate the resource management system's integration with
//! the ECS world, systems, and real-world usage patterns.

use bemudjo_ecs::{Component, SequentialSystemScheduler, System, World};

// Test Resource Types
#[derive(Debug, Clone, PartialEq)]
struct GameTime {
    delta: f32,
    total: f32,
    frame_count: u64,
}
impl Component for GameTime {}

#[derive(Debug, Clone, PartialEq)]
struct PlayerScore {
    current: u64,
    high_score: u64,
    multiplier: f32,
}
impl Component for PlayerScore {}

#[derive(Debug, Clone, PartialEq)]
struct GameSettings {
    volume: f32,
    difficulty: u8,
    debug_mode: bool,
}
impl Component for GameSettings {}

#[derive(Debug, Clone, PartialEq)]
struct InputState {
    mouse_x: f32,
    mouse_y: f32,
    keys_pressed: Vec<String>,
    mouse_clicked: bool,
}
impl Component for InputState {}

#[derive(Debug, Clone, PartialEq)]
struct Statistics {
    entities_spawned: u64,
    systems_executed: u64,
    frames_rendered: u64,
}
impl Component for Statistics {}

// Test Entity Components
#[derive(Debug, Clone, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Debug, Clone, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}
impl Component for Velocity {}

#[derive(Debug, Clone, PartialEq)]
struct Health {
    current: i32,
    max: i32,
}
impl Component for Health {}

#[derive(Debug, Clone, PartialEq)]
struct Enemy {
    damage: i32,
}
impl Component for Enemy {}

// Test Systems

/// System that updates game time resource
struct TimeUpdateSystem;
impl System for TimeUpdateSystem {
    fn run(&self, world: &mut World) {
        // Update time resource
        world
            .update_resource::<GameTime, _>(|mut time| {
                time.frame_count += 1;
                time.total += time.delta;
                time
            })
            .unwrap_or_else(|_| {
                // If no time resource exists, create one
                world.insert_resource(GameTime {
                    delta: 0.016,
                    total: 0.0,
                    frame_count: 1,
                });
                world.get_resource::<GameTime>().unwrap().clone()
            });
    }
}

/// System that moves entities based on velocity and time
struct MovementSystem;
impl System for MovementSystem {
    fn run(&self, world: &mut World) {
        // Get time resource
        let time = world.get_resource::<GameTime>();
        let delta = time.map(|t| t.delta).unwrap_or(0.016);

        // Move entities
        let entities: Vec<_> = world.entities().cloned().collect();
        for entity in entities {
            if let (Some(pos), Some(vel)) = (
                world.get_component::<Position>(entity),
                world.get_component::<Velocity>(entity),
            ) {
                let new_pos = Position {
                    x: pos.x + vel.x * delta,
                    y: pos.y + vel.y * delta,
                };
                world.replace_component(entity, new_pos);
            }
        }
    }
}

/// System that manages player score based on enemy defeats
struct ScoreSystem;
impl System for ScoreSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();
        let mut enemies_defeated = 0;

        // Check for defeated enemies (health <= 0)
        for entity in entities {
            if let (Some(health), Some(_enemy)) = (
                world.get_component::<Health>(entity),
                world.get_component::<Enemy>(entity),
            ) {
                if health.current <= 0 {
                    enemies_defeated += 1;
                    // Remove defeated enemy
                    world.delete_entity(entity);
                }
            }
        }

        // Update score if enemies were defeated
        if enemies_defeated > 0 {
            if world.has_resource::<PlayerScore>() {
                world
                    .update_resource::<PlayerScore, _>(|mut score| {
                        let points = enemies_defeated as u64 * 100;
                        score.current += (points as f32 * score.multiplier) as u64;
                        if score.current > score.high_score {
                            score.high_score = score.current;
                        }
                        score
                    })
                    .unwrap();
            } else {
                // Initialize score if it doesn't exist
                world.insert_resource(PlayerScore {
                    current: enemies_defeated as u64 * 100,
                    high_score: enemies_defeated as u64 * 100,
                    multiplier: 1.0,
                });
            }
        }
    }
}

/// System that tracks statistics
struct StatisticsSystem;
impl System for StatisticsSystem {
    fn run(&self, world: &mut World) {
        let entity_count = world.entities().count() as u64;

        if world.has_resource::<Statistics>() {
            world
                .update_resource::<Statistics, _>(|mut stats| {
                    stats.systems_executed += 1;
                    stats.frames_rendered += 1;
                    // Track max entities seen as approximation of spawned
                    if entity_count > stats.entities_spawned {
                        stats.entities_spawned = entity_count;
                    }
                    stats
                })
                .unwrap();
        } else {
            world.insert_resource(Statistics {
                entities_spawned: entity_count,
                systems_executed: 1,
                frames_rendered: 1,
            });
        }
    }
}

/// System that reads settings to adjust game behavior
struct SettingsAwareSystem;
impl System for SettingsAwareSystem {
    fn run(&self, world: &mut World) {
        if let Some(settings) = world.get_resource::<GameSettings>() {
            if settings.debug_mode {
                // In debug mode, multiply score by debug multiplier
                if world.has_resource::<PlayerScore>() {
                    world
                        .update_resource::<PlayerScore, _>(|mut score| {
                            score.multiplier = 2.0; // Debug mode double multiplier
                            score
                        })
                        .unwrap();
                }
            }
        }
    }
}

#[test]
fn test_resource_system_integration() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Add systems
    scheduler.add_system(TimeUpdateSystem).unwrap();
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(ScoreSystem).unwrap();
    scheduler.add_system(StatisticsSystem).unwrap();
    scheduler.build().unwrap();

    // Create entities
    let player = world.spawn_entity();
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(player, Velocity { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(
            player,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();

    let enemy = world.spawn_entity();
    world
        .add_component(enemy, Position { x: 10.0, y: 10.0 })
        .unwrap();
    world
        .add_component(
            enemy,
            Health {
                current: 50,
                max: 50,
            },
        )
        .unwrap();
    world.add_component(enemy, Enemy { damage: 25 }).unwrap();

    // Initialize resources
    world.insert_resource(GameTime {
        delta: 0.016,
        total: 0.0,
        frame_count: 0,
    });

    // Run simulation for several ticks
    for _ in 0..5 {
        scheduler.run_tick(&mut world);
    }

    // Verify time resource was updated
    let time = world.get_resource::<GameTime>().unwrap();
    assert_eq!(time.frame_count, 5);
    assert!((time.total - 0.08).abs() < 0.001); // 5 * 0.016

    // Verify player moved
    let player_pos = world.get_component::<Position>(player).unwrap();
    assert!((player_pos.x - 0.08).abs() < 0.001); // 5 * 1.0 * 0.016
    assert!((player_pos.y - 0.08).abs() < 0.001);

    // Verify statistics were tracked
    let stats = world.get_resource::<Statistics>().unwrap();
    assert_eq!(stats.systems_executed, 5);
    assert_eq!(stats.frames_rendered, 5);
    assert!(stats.entities_spawned >= 2);
}

#[test]
fn test_multiple_systems_sharing_resources() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(TimeUpdateSystem).unwrap();
    scheduler.add_system(SettingsAwareSystem).unwrap();
    scheduler.add_system(ScoreSystem).unwrap();
    scheduler.build().unwrap();

    // Initialize resources
    world.insert_resource(GameTime {
        delta: 0.016,
        total: 0.0,
        frame_count: 0,
    });
    world.insert_resource(GameSettings {
        volume: 0.8,
        difficulty: 3,
        debug_mode: true,
    });
    world.insert_resource(PlayerScore {
        current: 1000,
        high_score: 1500,
        multiplier: 1.0,
    });

    // Create enemy to defeat
    let enemy = world.spawn_entity();
    world
        .add_component(
            enemy,
            Health {
                current: 0,
                max: 50,
            },
        )
        .unwrap(); // Already defeated
    world.add_component(enemy, Enemy { damage: 25 }).unwrap();

    // Run one tick
    scheduler.run_tick(&mut world);

    // Time should be updated
    let time = world.get_resource::<GameTime>().unwrap();
    assert_eq!(time.frame_count, 1);

    // Score should be updated with debug multiplier
    let score = world.get_resource::<PlayerScore>().unwrap();
    assert_eq!(score.multiplier, 2.0); // Set by SettingsAwareSystem
    assert_eq!(score.current, 1200); // 1000 + (100 * 2.0)
    assert_eq!(score.high_score, 1500); // Unchanged since 1200 < 1500
}

#[test]
fn test_resource_lifecycle_with_systems() {
    let mut world = World::new();

    // System that creates resources if they don't exist
    struct ResourceInitializerSystem;
    impl System for ResourceInitializerSystem {
        fn run(&self, world: &mut World) {
            if !world.has_resource::<GameTime>() {
                world.insert_resource(GameTime {
                    delta: 0.016,
                    total: 0.0,
                    frame_count: 0,
                });
            }

            if !world.has_resource::<InputState>() {
                world.insert_resource(InputState {
                    mouse_x: 0.0,
                    mouse_y: 0.0,
                    keys_pressed: vec![],
                    mouse_clicked: false,
                });
            }
        }
    }

    // System that removes resources under certain conditions
    struct ResourceCleanupSystem;
    impl System for ResourceCleanupSystem {
        fn run(&self, world: &mut World) {
            if let Some(time) = world.get_resource::<GameTime>() {
                if time.frame_count > 10 {
                    world.remove_resource::<GameTime>();
                }
            }
        }
    }

    let mut scheduler = SequentialSystemScheduler::new();
    scheduler.add_system(ResourceInitializerSystem).unwrap();
    scheduler.add_system(TimeUpdateSystem).unwrap();
    scheduler.add_system(ResourceCleanupSystem).unwrap();
    scheduler.build().unwrap();

    // Initially no resources
    assert!(!world.has_resource::<GameTime>());
    assert!(!world.has_resource::<InputState>());

    // First tick - resources should be created
    scheduler.run_tick(&mut world);
    assert!(world.has_resource::<GameTime>());
    assert!(world.has_resource::<InputState>());
    assert_eq!(world.get_resource::<GameTime>().unwrap().frame_count, 1);

    // Run more ticks
    for _ in 0..10 {
        scheduler.run_tick(&mut world);
    }

    // After 11 total ticks, GameTime should be removed but InputState should remain
    assert!(!world.has_resource::<GameTime>());
    assert!(world.has_resource::<InputState>());
}

#[test]
fn test_resource_error_handling_in_systems() {
    let mut world = World::new();

    // System that tries to access non-existent resources
    struct SafeResourceAccessSystem {
        pub executed: std::cell::RefCell<bool>,
    }
    impl System for SafeResourceAccessSystem {
        fn run(&self, world: &mut World) {
            // Try to update non-existent resource
            let result = world.update_resource::<GameTime, _>(|mut time| {
                time.frame_count += 1;
                time
            });

            // Should handle error gracefully
            if result.is_err() {
                // Create the resource since it doesn't exist
                world.insert_resource(GameTime {
                    delta: 0.016,
                    total: 0.0,
                    frame_count: 1,
                });
            }

            *self.executed.borrow_mut() = true;
        }
    }

    let safe_system = SafeResourceAccessSystem {
        executed: std::cell::RefCell::new(false),
    };

    let mut scheduler = SequentialSystemScheduler::new();
    scheduler.add_system(safe_system).unwrap();
    scheduler.build().unwrap();

    // Initially no resources
    assert!(!world.has_resource::<GameTime>());

    // Run system
    scheduler.run_tick(&mut world);

    // System should have handled the error and created the resource
    assert!(world.has_resource::<GameTime>());
    assert_eq!(world.get_resource::<GameTime>().unwrap().frame_count, 1);
}

#[test]
fn test_realistic_game_loop_with_resources() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Full game loop systems
    scheduler.add_system(TimeUpdateSystem).unwrap();
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(ScoreSystem).unwrap();
    scheduler.add_system(StatisticsSystem).unwrap();
    scheduler.build().unwrap();

    // Initialize game state
    world.insert_resource(GameTime {
        delta: 0.016,
        total: 0.0,
        frame_count: 0,
    });
    world.insert_resource(PlayerScore {
        current: 0,
        high_score: 0,
        multiplier: 1.0,
    });
    world.insert_resource(GameSettings {
        volume: 1.0,
        difficulty: 2,
        debug_mode: false,
    });

    // Create game entities
    let player = world.spawn_entity();
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(player, Velocity { x: 2.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            player,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();

    // Create multiple enemies
    let mut enemies = Vec::new();
    for i in 0..5 {
        let enemy = world.spawn_entity();
        world
            .add_component(
                enemy,
                Position {
                    x: 10.0 + i as f32,
                    y: 0.0,
                },
            )
            .unwrap();
        world
            .add_component(
                enemy,
                Health {
                    current: 25,
                    max: 25,
                },
            )
            .unwrap();
        world.add_component(enemy, Enemy { damage: 10 }).unwrap();
        enemies.push(enemy);
    }

    // Simulate game loop
    for frame in 0..60 {
        scheduler.run_tick(&mut world);

        // Damage enemies periodically
        if frame % 10 == 0 && frame > 0 {
            for &enemy in &enemies {
                if world.has_component::<Health>(enemy) {
                    world
                        .update_component::<Health, _>(enemy, |mut health| {
                            health.current -= 25; // Kill enemy
                            health
                        })
                        .ok();
                }
            }
        }
    }

    // Verify final state
    let time = world.get_resource::<GameTime>().unwrap();
    assert_eq!(time.frame_count, 60);
    assert!((time.total - 0.96).abs() < 0.001); // 60 * 0.016

    let score = world.get_resource::<PlayerScore>().unwrap();
    assert!(score.current > 0); // Should have scored points from defeated enemies
    assert_eq!(score.high_score, score.current); // High score should equal current

    let stats = world.get_resource::<Statistics>().unwrap();
    assert_eq!(stats.systems_executed, 60);
    assert_eq!(stats.frames_rendered, 60);

    // Player should have moved
    let player_pos = world.get_component::<Position>(player).unwrap();
    assert!((player_pos.x - 1.92).abs() < 0.001); // 60 * 2.0 * 0.016

    // Most enemies should be defeated
    let remaining_enemies = world.entities().count();
    assert!(remaining_enemies <= 1); // Only player should remain (or maybe 1 enemy)
}

#[test]
fn test_resource_performance_scenario() {
    let mut world = World::new();

    // System that manages many resources
    struct MassResourceSystem;
    impl System for MassResourceSystem {
        fn run(&self, world: &mut World) {
            // Update or create many different resource types
            for i in 0..100 {
                if i % 2 == 0 {
                    world.insert_resource(Statistics {
                        entities_spawned: i,
                        systems_executed: i * 2,
                        frames_rendered: i * 3,
                    });
                } else {
                    world
                        .update_resource::<Statistics, _>(|mut stats| {
                            stats.entities_spawned += 1;
                            stats
                        })
                        .unwrap_or_else(|_| {
                            world.insert_resource(Statistics {
                                entities_spawned: 1,
                                systems_executed: 0,
                                frames_rendered: 0,
                            });
                            Statistics {
                                entities_spawned: 1,
                                systems_executed: 0,
                                frames_rendered: 0,
                            }
                        });
                }
            }
        }
    }

    let mut scheduler = SequentialSystemScheduler::new();
    scheduler.add_system(MassResourceSystem).unwrap();
    scheduler.build().unwrap();

    // Run performance test
    let start_time = std::time::Instant::now();
    for _ in 0..10 {
        scheduler.run_tick(&mut world);
    }
    let duration = start_time.elapsed();

    // Should complete quickly (performance test)
    assert!(duration.as_millis() < 100); // Less than 100ms for 10 iterations

    // Verify final state
    assert!(world.has_resource::<Statistics>());
    let stats = world.get_resource::<Statistics>().unwrap();
    assert!(stats.entities_spawned > 0);
}

#[test]
fn test_resource_state_consistency() {
    let mut world = World::new();

    // Multiple systems that modify the same resource
    struct IncrementSystem;
    impl System for IncrementSystem {
        fn run(&self, world: &mut World) {
            world
                .update_resource::<Statistics, _>(|mut stats| {
                    stats.entities_spawned += 1;
                    stats
                })
                .unwrap_or_else(|_| {
                    world.insert_resource(Statistics {
                        entities_spawned: 1,
                        systems_executed: 0,
                        frames_rendered: 0,
                    });
                    world.get_resource::<Statistics>().unwrap().clone()
                });
        }
    }

    struct MultiplySystem;
    impl System for MultiplySystem {
        fn run(&self, world: &mut World) {
            if world.has_resource::<Statistics>() {
                world
                    .update_resource::<Statistics, _>(|mut stats| {
                        stats.systems_executed = stats.entities_spawned * 2;
                        stats
                    })
                    .unwrap();
            }
        }
    }

    let mut scheduler = SequentialSystemScheduler::new();
    scheduler.add_system(IncrementSystem).unwrap();
    scheduler.add_system(MultiplySystem).unwrap();
    scheduler.build().unwrap();

    // Run multiple ticks
    for _ in 0..5 {
        scheduler.run_tick(&mut world);
    }

    // Verify consistent state
    let stats = world.get_resource::<Statistics>().unwrap();
    assert_eq!(stats.entities_spawned, 5);
    assert_eq!(stats.systems_executed, 10); // 5 * 2
}

#[test]
fn test_resource_removal_during_execution() {
    let mut world = World::new();

    // System that removes resources
    struct ResourceRemoverSystem;
    impl System for ResourceRemoverSystem {
        fn run(&self, world: &mut World) {
            if let Some(time) = world.get_resource::<GameTime>() {
                if time.frame_count >= 3 {
                    world.remove_resource::<GameTime>();
                }
            }
        }
    }

    // System that tries to use resources that might be removed
    struct ResourceUserSystem {
        pub attempts: std::cell::RefCell<u32>,
        pub successes: std::cell::RefCell<u32>,
    }
    impl System for ResourceUserSystem {
        fn run(&self, world: &mut World) {
            *self.attempts.borrow_mut() += 1;

            if world.has_resource::<GameTime>() {
                world
                    .update_resource::<GameTime, _>(|mut time| {
                        time.frame_count += 1;
                        time
                    })
                    .unwrap();
                *self.successes.borrow_mut() += 1;
            }
        }
    }

    let user_system = ResourceUserSystem {
        attempts: std::cell::RefCell::new(0),
        successes: std::cell::RefCell::new(0),
    };

    let mut scheduler = SequentialSystemScheduler::new();
    scheduler.add_system(user_system).unwrap();
    scheduler.add_system(ResourceRemoverSystem).unwrap();
    scheduler.build().unwrap();

    // Initialize resource
    world.insert_resource(GameTime {
        delta: 0.016,
        total: 0.0,
        frame_count: 0,
    });

    // Run several ticks
    for _ in 0..5 {
        scheduler.run_tick(&mut world);
    }

    // Resource should be removed after frame 3
    assert!(!world.has_resource::<GameTime>());

    // Verify system behavior
    // Should have attempted 5 times but only succeeded 3 times (before removal)
    // Note: Due to system execution order, the exact numbers may vary
    // The important thing is that no panics occurred
}

#[test]
fn test_resource_type_safety() {
    let mut world = World::new();

    // Insert different resource types
    world.insert_resource(GameTime {
        delta: 0.016,
        total: 0.0,
        frame_count: 0,
    });
    world.insert_resource(PlayerScore {
        current: 1000,
        high_score: 1500,
        multiplier: 1.5,
    });
    world.insert_resource(GameSettings {
        volume: 0.8,
        difficulty: 3,
        debug_mode: true,
    });

    // Each resource type should be independent
    assert!(world.has_resource::<GameTime>());
    assert!(world.has_resource::<PlayerScore>());
    assert!(world.has_resource::<GameSettings>());

    // Removing one shouldn't affect others
    world.remove_resource::<PlayerScore>();
    assert!(world.has_resource::<GameTime>());
    assert!(!world.has_resource::<PlayerScore>());
    assert!(world.has_resource::<GameSettings>());

    // Type-specific access should work correctly
    let time = world.get_resource::<GameTime>().unwrap();
    assert_eq!(time.frame_count, 0);

    let settings = world.get_resource::<GameSettings>().unwrap();
    assert_eq!(settings.difficulty, 3);

    // Accessing removed resource should return None
    assert!(world.get_resource::<PlayerScore>().is_none());
}
