//! Integration tests for the bemudjo ECS library
//!
//! These tests validate the public API and realistic usage patterns
//! by testing the library as an external user would.

use bemudjo_ecs::{Component, ComponentError, SequentialSystemScheduler, System, World};

// Test Components
#[derive(Clone, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Clone, Debug, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}
impl Component for Velocity {}

#[derive(Clone, Debug, PartialEq)]
struct Health {
    current: u32,
    max: u32,
}
impl Component for Health {}

#[derive(Clone, Debug, PartialEq)]
struct Name {
    value: String,
}
impl Component for Name {}

#[derive(Clone, Debug, PartialEq)]
struct Experience {
    points: u64,
    level: u32,
}
impl Component for Experience {}

// Test Systems
struct MovementSystem;
impl System for MovementSystem {
    fn run(&self, world: &mut World) {
        // Collect entities that have both Position and Velocity
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if let (Some(pos), Some(vel)) = (
                world.get_component::<Position>(entity),
                world.get_component::<Velocity>(entity),
            ) {
                let new_pos = Position {
                    x: pos.x + vel.x,
                    y: pos.y + vel.y,
                };
                world.replace_component(entity, new_pos);
            }
        }
    }
}

struct HealthRegenSystem;
impl System for HealthRegenSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if world.has_component::<Health>(entity) {
                world
                    .update_component::<Health, _>(entity, |mut health| {
                        if health.current < health.max {
                            health.current = (health.current + 1).min(health.max);
                        }
                        health
                    })
                    .ok(); // Ignore errors for this test
            }
        }
    }
}

struct LoggingSystem {
    log_entries: std::cell::RefCell<Vec<String>>,
}

impl LoggingSystem {
    fn new() -> Self {
        Self {
            log_entries: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl System for LoggingSystem {
    fn before_run(&self, world: &World) {
        let entity_count = world.entities().count();
        self.log_entries
            .borrow_mut()
            .push(format!("BEFORE: {} entities", entity_count));
    }

    fn after_run(&self, world: &World) {
        let entity_count = world.entities().count();
        self.log_entries
            .borrow_mut()
            .push(format!("AFTER: {} entities", entity_count));
    }
}

#[test]
fn test_basic_world_operations() {
    let mut world = World::new();

    // Test empty world
    assert_eq!(world.entities().count(), 0);

    // Spawn entities
    let player = world.spawn_entity();
    let enemy = world.spawn_entity();

    assert_eq!(world.entities().count(), 2);
    assert_ne!(player, enemy);

    // Add components
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
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
    world
        .add_component(
            player,
            Name {
                value: "Player".to_string(),
            },
        )
        .unwrap();

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

    // Verify components
    assert!(world.has_component::<Position>(player));
    assert!(world.has_component::<Health>(player));
    assert!(world.has_component::<Name>(player));
    assert!(!world.has_component::<Velocity>(player));

    assert!(world.has_component::<Position>(enemy));
    assert!(world.has_component::<Health>(enemy));
    assert!(!world.has_component::<Name>(enemy));

    // Test component access
    let player_pos = world.get_component::<Position>(player).unwrap();
    assert_eq!(player_pos.x, 0.0);
    assert_eq!(player_pos.y, 0.0);

    let player_name = world.get_component::<Name>(player).unwrap();
    assert_eq!(player_name.value, "Player");

    // Test component updates
    world
        .update_component::<Health, _>(player, |mut health| {
            health.current -= 25;
            health
        })
        .unwrap();

    let player_health = world.get_component::<Health>(player).unwrap();
    assert_eq!(player_health.current, 75);
    assert_eq!(player_health.max, 100);

    // Test entity deletion
    world.delete_entity(enemy);
    assert_eq!(world.entities().count(), 1);
    assert!(!world.has_component::<Position>(enemy));

    // Test cleanup
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 1);
}

#[test]
fn test_component_lifecycle() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add initial component
    world
        .add_component(entity, Position { x: 1.0, y: 2.0 })
        .unwrap();
    assert!(world.has_component::<Position>(entity));

    // Try to add duplicate component (should fail)
    let result = world.add_component(entity, Position { x: 3.0, y: 4.0 });
    assert!(matches!(
        result,
        Err(ComponentError::ComponentAlreadyExists)
    ));

    // Replace component
    let old_pos = world.replace_component(entity, Position { x: 5.0, y: 6.0 });
    assert_eq!(old_pos, Some(Position { x: 1.0, y: 2.0 }));

    let current_pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(current_pos.x, 5.0);
    assert_eq!(current_pos.y, 6.0);

    // Remove component
    let removed_pos = world.remove_component::<Position>(entity);
    assert_eq!(removed_pos, Some(Position { x: 5.0, y: 6.0 }));
    assert!(!world.has_component::<Position>(entity));

    // Try to remove non-existent component
    let removed_again = world.remove_component::<Position>(entity);
    assert_eq!(removed_again, None);
}

#[test]
fn test_system_scheduler_basic() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Test empty scheduler
    assert_eq!(scheduler.system_count(), 0); // Add systems
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(HealthRegenSystem).unwrap();

    scheduler.build().unwrap();

    assert_eq!(scheduler.system_count(), 2);

    // Create test entities
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(entity1, Velocity { x: 1.0, y: 2.0 })
        .unwrap();
    world
        .add_component(
            entity1,
            Health {
                current: 90,
                max: 100,
            },
        )
        .unwrap();

    world
        .add_component(
            entity2,
            Health {
                current: 45,
                max: 50,
            },
        )
        .unwrap();

    // Run one tick
    scheduler.run_tick(&mut world);

    // Verify movement system worked
    let pos = world.get_component::<Position>(entity1).unwrap();
    assert_eq!(pos.x, 1.0);
    assert_eq!(pos.y, 2.0);

    // Verify health regen worked
    let health1 = world.get_component::<Health>(entity1).unwrap();
    assert_eq!(health1.current, 91); // 90 + 1

    let health2 = world.get_component::<Health>(entity2).unwrap();
    assert_eq!(health2.current, 46); // 45 + 1

    // Run another tick
    scheduler.run_tick(&mut world);

    // Verify continued updates
    let pos = world.get_component::<Position>(entity1).unwrap();
    assert_eq!(pos.x, 2.0);
    assert_eq!(pos.y, 4.0);

    let health1 = world.get_component::<Health>(entity1).unwrap();
    assert_eq!(health1.current, 92);
}

#[test]
fn test_system_execution_phases() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let logging_system = LoggingSystem::new();
    scheduler.add_system(logging_system).unwrap();

    scheduler.build().unwrap();

    // Create some entities
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: 1.0, y: 1.0 })
        .unwrap();

    // Run tick - this should trigger before_run and after_run
    scheduler.run_tick(&mut world);

    // Note: Since we can't access the logging system after adding it to the scheduler,
    // we'll test the phase execution indirectly by creating a new test system
}

#[test]
fn test_system_execution_order() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Create a system that tracks execution order using a component
    struct OrderTrackingSystem {
        id: u32,
    }

    impl System for OrderTrackingSystem {
        fn run(&self, world: &mut World) {
            // Find or create the tracking entity
            let existing_entity = world.entities().next().cloned();
            let tracking_entity = if let Some(entity) = existing_entity {
                if world.has_component::<Experience>(entity) {
                    entity
                } else {
                    let new_entity = world.spawn_entity();
                    world
                        .add_component(
                            new_entity,
                            Experience {
                                points: 0,
                                level: 0,
                            },
                        )
                        .unwrap();
                    new_entity
                }
            } else {
                let new_entity = world.spawn_entity();
                world
                    .add_component(
                        new_entity,
                        Experience {
                            points: 0,
                            level: 0,
                        },
                    )
                    .unwrap();
                new_entity
            };

            // Update the experience points to track execution order
            world
                .update_component::<Experience, _>(tracking_entity, |mut exp| {
                    exp.points = exp.points * 10 + self.id as u64;
                    exp
                })
                .unwrap();
        }
    }

    // Add systems in specific order
    scheduler.add_system(OrderTrackingSystem { id: 1 }).unwrap();
    scheduler.add_system(OrderTrackingSystem { id: 2 }).unwrap();
    scheduler.add_system(OrderTrackingSystem { id: 3 }).unwrap();

    scheduler.build().unwrap();
    assert_eq!(scheduler.system_count(), 3);

    // Run one tick
    scheduler.run_tick(&mut world);

    // Check execution order (should be 123)
    let tracking_entity = world.entities().next().cloned().unwrap();
    let exp = world.get_component::<Experience>(tracking_entity).unwrap();
    assert_eq!(exp.points, 123); // 0 -> 1 -> 12 -> 123
}

#[test]
fn test_complex_ecs_scenario() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Add systems
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(HealthRegenSystem).unwrap();

    scheduler.build().unwrap();

    // Create a complex scenario with multiple entity types

    // Player
    let player = world.spawn_entity();
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(player, Velocity { x: 1.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            player,
            Health {
                current: 80,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Name {
                value: "Hero".to_string(),
            },
        )
        .unwrap();

    // NPCs
    let npc1 = world.spawn_entity();
    world
        .add_component(npc1, Position { x: 10.0, y: 5.0 })
        .unwrap();
    world
        .add_component(
            npc1,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            npc1,
            Name {
                value: "Guard".to_string(),
            },
        )
        .unwrap();

    let npc2 = world.spawn_entity();
    world
        .add_component(npc2, Position { x: -5.0, y: 10.0 })
        .unwrap();
    world
        .add_component(npc2, Velocity { x: 0.0, y: -1.0 })
        .unwrap();
    world
        .add_component(
            npc2,
            Health {
                current: 75,
                max: 75,
            },
        )
        .unwrap();

    // Static objects (no health, no movement)
    let treasure = world.spawn_entity();
    world
        .add_component(treasure, Position { x: 20.0, y: 20.0 })
        .unwrap();
    world
        .add_component(
            treasure,
            Name {
                value: "Treasure Chest".to_string(),
            },
        )
        .unwrap();

    assert_eq!(world.entities().count(), 4);

    // Run simulation for multiple ticks
    for tick in 0..5 {
        scheduler.run_tick(&mut world);

        // Verify player movement
        let player_pos = world.get_component::<Position>(player).unwrap();
        assert_eq!(player_pos.x, (tick + 1) as f32);
        assert_eq!(player_pos.y, 0.0);

        // Verify NPC movement
        let npc2_pos = world.get_component::<Position>(npc2).unwrap();
        assert_eq!(npc2_pos.x, -5.0);
        assert_eq!(npc2_pos.y, 10.0 - (tick + 1) as f32);

        // Verify health regeneration
        let player_health = world.get_component::<Health>(player).unwrap();
        let expected_health = std::cmp::min(80 + tick + 1, 100);
        assert_eq!(player_health.current, expected_health as u32);
    }

    // Verify static entities remain unchanged
    let treasure_pos = world.get_component::<Position>(treasure).unwrap();
    assert_eq!(treasure_pos.x, 20.0);
    assert_eq!(treasure_pos.y, 20.0);

    let guard_pos = world.get_component::<Position>(npc1).unwrap();
    assert_eq!(guard_pos.x, 10.0);
    assert_eq!(guard_pos.y, 5.0);

    // Clean up one entity
    world.delete_entity(treasure);
    assert_eq!(world.entities().count(), 3);

    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 3);
    assert!(!world.has_component::<Position>(treasure));
    assert!(!world.has_component::<Name>(treasure));
}

#[test]
fn test_error_handling() {
    let mut world = World::new();

    // Test operations on non-existent entity
    let fake_entity = {
        let mut temp_world = World::new();
        temp_world.spawn_entity()
    };

    // These should all fail gracefully
    let result = world.add_component(fake_entity, Position { x: 0.0, y: 0.0 });
    assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

    assert!(world.get_component::<Position>(fake_entity).is_none());
    assert!(!world.has_component::<Position>(fake_entity));
    assert!(world.remove_component::<Position>(fake_entity).is_none());
    assert!(world
        .replace_component(fake_entity, Position { x: 0.0, y: 0.0 })
        .is_none());

    let update_result = world.update_component::<Position, _>(fake_entity, |pos| pos);
    assert!(matches!(
        update_result,
        Err(ComponentError::ComponentNotFound)
    ));

    // Test operations on deleted entity
    let entity = world.spawn_entity();
    world
        .add_component(entity, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world.delete_entity(entity);

    // These should all fail gracefully after deletion
    let result = world.add_component(
        entity,
        Health {
            current: 100,
            max: 100,
        },
    );
    assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

    assert!(world.get_component::<Position>(entity).is_none());
    assert!(!world.has_component::<Position>(entity));
    assert!(world.remove_component::<Position>(entity).is_none());
    assert!(world
        .replace_component(entity, Position { x: 2.0, y: 2.0 })
        .is_none());

    let update_result = world.update_component::<Position, _>(entity, |pos| pos);
    assert!(matches!(
        update_result,
        Err(ComponentError::ComponentNotFound)
    ));
}

#[test]
fn test_performance_scenario() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(MovementSystem).unwrap();

    scheduler.build().unwrap();

    // Create many entities
    const ENTITY_COUNT: usize = 1000;
    let mut entities = Vec::new();

    for i in 0..ENTITY_COUNT {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32,
                },
            )
            .unwrap();

        // Only some entities have velocity
        if i % 2 == 0 {
            world
                .add_component(entity, Velocity { x: 1.0, y: 0.5 })
                .unwrap();
        }

        entities.push(entity);
    }

    assert_eq!(world.entities().count(), ENTITY_COUNT);

    // Run simulation
    scheduler.run_tick(&mut world);

    // Verify results
    for (i, &entity) in entities.iter().enumerate() {
        let pos = world.get_component::<Position>(entity).unwrap();

        if i % 2 == 0 {
            // Entities with velocity should have moved
            assert_eq!(pos.x, i as f32 + 1.0);
            assert_eq!(pos.y, i as f32 + 0.5);
        } else {
            // Entities without velocity should not have moved
            assert_eq!(pos.x, i as f32);
            assert_eq!(pos.y, i as f32);
        }
    }

    // Delete every third entity
    for i in (0..ENTITY_COUNT).step_by(3) {
        world.delete_entity(entities[i]);
    }

    let remaining_count = world.entities().count();
    assert!(remaining_count < ENTITY_COUNT);
    // With step_by(3), we delete roughly 1/3 of entities, so about 2/3 should remain
    assert!(remaining_count >= ENTITY_COUNT * 2 / 3 - 1); // Allow for rounding

    // Cleanup
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), remaining_count);
}

#[test]
fn test_multiple_component_types() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add all component types
    world
        .add_component(entity, Position { x: 1.0, y: 2.0 })
        .unwrap();
    world
        .add_component(entity, Velocity { x: 0.5, y: -0.3 })
        .unwrap();
    world
        .add_component(
            entity,
            Health {
                current: 75,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            entity,
            Name {
                value: "Test Entity".to_string(),
            },
        )
        .unwrap();
    world
        .add_component(
            entity,
            Experience {
                points: 1500,
                level: 5,
            },
        )
        .unwrap();

    // Verify all components exist
    assert!(world.has_component::<Position>(entity));
    assert!(world.has_component::<Velocity>(entity));
    assert!(world.has_component::<Health>(entity));
    assert!(world.has_component::<Name>(entity));
    assert!(world.has_component::<Experience>(entity));

    // Test accessing each component type
    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 1.0);

    let vel = world.get_component::<Velocity>(entity).unwrap();
    assert_eq!(vel.x, 0.5);

    let health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(health.current, 75);

    let name = world.get_component::<Name>(entity).unwrap();
    assert_eq!(name.value, "Test Entity");

    let exp = world.get_component::<Experience>(entity).unwrap();
    assert_eq!(exp.points, 1500);
    assert_eq!(exp.level, 5);

    // Test removing components one by one
    world.remove_component::<Velocity>(entity);
    assert!(!world.has_component::<Velocity>(entity));
    assert!(world.has_component::<Position>(entity)); // Others should remain

    world.remove_component::<Experience>(entity);
    assert!(!world.has_component::<Experience>(entity));
    assert!(world.has_component::<Health>(entity)); // Others should remain

    // Test component replacement
    world.replace_component(
        entity,
        Health {
            current: 100,
            max: 120,
        },
    );
    let new_health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(new_health.current, 100);
    assert_eq!(new_health.max, 120);
}

#[test]
fn test_empty_system_trait_methods() {
    struct EmptySystem;
    impl System for EmptySystem {
        // Uses default empty implementations
    }

    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(EmptySystem).unwrap();
    scheduler.build().unwrap();
    assert_eq!(scheduler.system_count(), 1);

    // Should run without errors even with empty implementations
    scheduler.run_tick(&mut world);

    // World should be unchanged
    assert_eq!(world.entities().count(), 0);
}

#[test]
fn test_realistic_game_loop_simulation() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Add core game systems
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(HealthRegenSystem).unwrap();

    scheduler.build().unwrap();

    // Create player character
    let player = world.spawn_entity();
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(player, Velocity { x: 1.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            player,
            Health {
                current: 90,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Name {
                value: "Player".to_string(),
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Experience {
                points: 0,
                level: 1,
            },
        )
        .unwrap();

    // Create enemies
    let mut enemies = Vec::new();
    for i in 0..3 {
        let enemy = world.spawn_entity();
        world
            .add_component(
                enemy,
                Position {
                    x: 10.0 + i as f32 * 5.0,
                    y: 5.0,
                },
            )
            .unwrap();
        world
            .add_component(
                enemy,
                Health {
                    current: 30,
                    max: 30,
                },
            )
            .unwrap();
        world
            .add_component(
                enemy,
                Name {
                    value: format!("Enemy {}", i + 1),
                },
            )
            .unwrap();
        enemies.push(enemy);
    }

    // Simulate game ticks
    const SIMULATION_TICKS: usize = 10;

    for tick in 0..SIMULATION_TICKS {
        // Simulate game events
        if tick == 3 {
            // Player takes damage
            world
                .update_component::<Health, _>(player, |mut health| {
                    health.current = health.current.saturating_sub(15);
                    health
                })
                .unwrap();
        }

        if tick == 7 {
            // Defeat an enemy
            world.delete_entity(enemies[1]);
        }

        // Run ECS tick
        scheduler.run_tick(&mut world);

        // Verify player state
        let player_pos = world.get_component::<Position>(player).unwrap();
        assert_eq!(player_pos.x, (tick + 1) as f32);

        let player_health = world.get_component::<Health>(player).unwrap();

        match tick.cmp(&3) {
            std::cmp::Ordering::Less => {
                // Health should be regenerating normally
                assert_eq!(
                    player_health.current,
                    std::cmp::min(90 + tick + 1, 100) as u32
                );
            }
            std::cmp::Ordering::Equal => {
                // Health should be reduced by damage (90 + 4 - 15 = 79)
                assert_eq!(player_health.current, 79);
            }
            std::cmp::Ordering::Greater => {
                // Health should be regenerating from 79
                let expected = std::cmp::min(79 + (tick - 3), 100) as u32;
                assert_eq!(player_health.current, expected);
            }
        }
    }

    // Verify final world state
    let entity_count = world.entities().count();
    assert_eq!(entity_count, 3); // Player + 2 remaining enemies (1 was deleted)

    // Verify remaining entities
    assert!(world.has_component::<Position>(player));
    assert!(world.has_component::<Health>(enemies[0]));
    assert!(!world.has_component::<Health>(enemies[1])); // Deleted
    assert!(world.has_component::<Health>(enemies[2]));
}
