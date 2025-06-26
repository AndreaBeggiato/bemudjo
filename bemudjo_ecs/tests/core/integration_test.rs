//! World Operations Integration Tests
//!
//! Tests focused on core World API operations and their integration
//! with entity and component management.

use bemudjo_ecs::{Component, ComponentError, World};

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
struct Counter {
    value: i64,
}
impl Component for Counter {}

#[test]
fn test_world_initialization_and_state() {
    let world = World::new();

    // New world should be empty
    assert_eq!(world.entities().count(), 0);

    // Should be able to query for any component type (returns empty)
    assert!(!world
        .entities()
        .any(|&e| world.has_component::<Position>(e)));
    assert!(!world.entities().any(|&e| world.has_component::<Health>(e)));
}

#[test]
fn test_world_entity_management() {
    let mut world = World::new();

    // Spawn multiple entities
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    assert_eq!(world.entities().count(), 3);
    assert_ne!(entity1, entity2);
    assert_ne!(entity2, entity3);
    assert_ne!(entity1, entity3);

    // Verify entities exist in iteration
    let entities: Vec<_> = world.entities().cloned().collect();
    assert!(entities.contains(&entity1));
    assert!(entities.contains(&entity2));
    assert!(entities.contains(&entity3));
}

#[test]
fn test_world_component_operations() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add components
    world
        .add_component(entity, Position { x: 1.0, y: 2.0 })
        .unwrap();
    world
        .add_component(
            entity,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            entity,
            Name {
                value: "Test".to_string(),
            },
        )
        .unwrap();

    // Check component existence
    assert!(world.has_component::<Position>(entity));
    assert!(world.has_component::<Health>(entity));
    assert!(world.has_component::<Name>(entity));
    assert!(!world.has_component::<Velocity>(entity));

    // Get components
    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 1.0);
    assert_eq!(pos.y, 2.0);

    let health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(health.current, 100);
    assert_eq!(health.max, 100);

    let name = world.get_component::<Name>(entity).unwrap();
    assert_eq!(name.value, "Test");

    // Non-existent component should return None
    assert!(world.get_component::<Velocity>(entity).is_none());
}

#[test]
fn test_world_component_updates() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    world.add_component(entity, Counter { value: 0 }).unwrap();

    // Update component multiple times
    for i in 1..=5 {
        let result = world.update_component::<Counter, _>(entity, |mut counter| {
            counter.value += i;
            counter
        });

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.value, (1..=i).sum()); // Sum of 1+2+...+i
    }

    let final_counter = world.get_component::<Counter>(entity).unwrap();
    assert_eq!(final_counter.value, 15); // Sum of 1+2+3+4+5
}

#[test]
fn test_world_component_replacement() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add initial component
    world
        .add_component(entity, Position { x: 1.0, y: 1.0 })
        .unwrap();

    // Replace multiple times
    let old1 = world.replace_component(entity, Position { x: 2.0, y: 2.0 });
    assert_eq!(old1, Some(Position { x: 1.0, y: 1.0 }));

    let old2 = world.replace_component(entity, Position { x: 3.0, y: 3.0 });
    assert_eq!(old2, Some(Position { x: 2.0, y: 2.0 }));

    // Replace on entity without component
    let entity2 = world.spawn_entity();
    let old3 = world.replace_component(entity2, Position { x: 4.0, y: 4.0 });
    assert_eq!(old3, None);

    // Verify final states
    let pos1 = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos1.x, 3.0);
    assert_eq!(pos1.y, 3.0);

    let pos2 = world.get_component::<Position>(entity2).unwrap();
    assert_eq!(pos2.x, 4.0);
    assert_eq!(pos2.y, 4.0);
}

#[test]
fn test_world_component_removal() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add multiple components
    world
        .add_component(entity, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(entity, Velocity { x: 2.0, y: 2.0 })
        .unwrap();
    world
        .add_component(
            entity,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();

    assert!(world.has_component::<Position>(entity));
    assert!(world.has_component::<Velocity>(entity));
    assert!(world.has_component::<Health>(entity));

    // Remove components one by one
    let removed_vel = world.remove_component::<Velocity>(entity);
    assert_eq!(removed_vel, Some(Velocity { x: 2.0, y: 2.0 }));
    assert!(!world.has_component::<Velocity>(entity));
    assert!(world.has_component::<Position>(entity));
    assert!(world.has_component::<Health>(entity));

    let removed_pos = world.remove_component::<Position>(entity);
    assert_eq!(removed_pos, Some(Position { x: 1.0, y: 1.0 }));
    assert!(!world.has_component::<Position>(entity));
    assert!(world.has_component::<Health>(entity));

    // Try to remove non-existent component
    let removed_none = world.remove_component::<Velocity>(entity);
    assert_eq!(removed_none, None);

    // Entity should still exist
    assert_eq!(world.entities().count(), 1);
}

#[test]
fn test_world_entity_deletion() {
    let mut world = World::new();

    // Create entities with components
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(
            entity1,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();

    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world
        .add_component(entity2, Velocity { x: 1.0, y: 1.0 })
        .unwrap();

    world
        .add_component(
            entity3,
            Health {
                current: 50,
                max: 50,
            },
        )
        .unwrap();

    assert_eq!(world.entities().count(), 3);

    // Delete entity
    world.delete_entity(entity2);

    // Entity should be marked as deleted but still counted until cleanup
    assert_eq!(world.entities().count(), 2);
    assert!(!world.has_component::<Position>(entity2));
    assert!(!world.has_component::<Velocity>(entity2));

    // Other entities should be unaffected
    assert!(world.has_component::<Position>(entity1));
    assert!(world.has_component::<Health>(entity1));
    assert!(world.has_component::<Health>(entity3));

    // Cleanup
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 2);

    let remaining_entities: Vec<_> = world.entities().cloned().collect();
    assert!(remaining_entities.contains(&entity1));
    assert!(!remaining_entities.contains(&entity2));
    assert!(remaining_entities.contains(&entity3));
}

#[test]
fn test_world_error_handling() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add component
    world
        .add_component(entity, Position { x: 1.0, y: 1.0 })
        .unwrap();

    // Try to add duplicate
    let result = world.add_component(entity, Position { x: 2.0, y: 2.0 });
    assert!(matches!(
        result,
        Err(ComponentError::ComponentAlreadyExists)
    ));

    // Original component should be unchanged
    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 1.0);

    // Delete entity
    world.delete_entity(entity);

    // Operations on deleted entity should fail
    let result = world.add_component(entity, Velocity { x: 1.0, y: 1.0 });
    assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

    let result = world.update_component::<Position, _>(entity, |pos| pos);
    assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

    assert!(world.get_component::<Position>(entity).is_none());
    assert!(!world.has_component::<Position>(entity));
    assert!(world.remove_component::<Position>(entity).is_none());
    assert!(world
        .replace_component(entity, Position { x: 3.0, y: 3.0 })
        .is_none());
}

#[test]
fn test_world_concurrent_operations() {
    let mut world = World::new();

    // Create entities and perform many operations
    let mut entities = Vec::new();
    for i in 0..100 {
        let entity = world.spawn_entity();
        world.add_component(entity, Counter { value: i }).unwrap();
        entities.push(entity);
    }

    // Perform mixed operations
    for (i, &entity) in entities.iter().enumerate() {
        match i % 4 {
            0 => {
                // Update component
                world
                    .update_component::<Counter, _>(entity, |mut counter| {
                        counter.value *= 2;
                        counter
                    })
                    .unwrap();
            }
            1 => {
                // Add another component
                world
                    .add_component(
                        entity,
                        Position {
                            x: i as f32,
                            y: i as f32,
                        },
                    )
                    .unwrap();
            }
            2 => {
                // Replace component
                world.replace_component(entity, Counter { value: -(i as i64) });
            }
            3 => {
                // Remove and re-add component
                let old = world.remove_component::<Counter>(entity).unwrap();
                world
                    .add_component(
                        entity,
                        Counter {
                            value: old.value + 1000,
                        },
                    )
                    .unwrap();
            }
            _ => unreachable!(),
        }
    }

    // Verify final state
    assert_eq!(world.entities().count(), 100);

    for (i, &entity) in entities.iter().enumerate() {
        assert!(world.has_component::<Counter>(entity));

        let counter = world.get_component::<Counter>(entity).unwrap();
        let expected = match i % 4 {
            0 => i as i64 * 2,    // Doubled
            1 => i as i64,        // Unchanged
            2 => -(i as i64),     // Negated
            3 => i as i64 + 1000, // Added 1000
            _ => unreachable!(),
        };
        assert_eq!(counter.value, expected);

        // Every 4th entity (i%4==1) should have Position
        if i % 4 == 1 {
            assert!(world.has_component::<Position>(entity));
            let pos = world.get_component::<Position>(entity).unwrap();
            assert_eq!(pos.x, i as f32);
            assert_eq!(pos.y, i as f32);
        } else {
            assert!(!world.has_component::<Position>(entity));
        }
    }
}

#[test]
fn test_world_large_scale_operations() {
    let mut world = World::new();

    const ENTITY_COUNT: usize = 10_000;
    let mut entities = Vec::new();

    // Mass entity creation
    for i in 0..ENTITY_COUNT {
        let entity = world.spawn_entity();
        world
            .add_component(entity, Counter { value: i as i64 })
            .unwrap();

        if i % 2 == 0 {
            world
                .add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: 0.0,
                    },
                )
                .unwrap();
        }

        if i % 3 == 0 {
            world
                .add_component(
                    entity,
                    Health {
                        current: (i % 100) as u32,
                        max: 100,
                    },
                )
                .unwrap();
        }

        entities.push(entity);
    }

    assert_eq!(world.entities().count(), ENTITY_COUNT);

    // Verify component distribution
    let mut pos_count = 0;
    let mut health_count = 0;

    for &entity in &entities {
        assert!(world.has_component::<Counter>(entity));

        if world.has_component::<Position>(entity) {
            pos_count += 1;
        }

        if world.has_component::<Health>(entity) {
            health_count += 1;
        }
    }

    assert_eq!(pos_count, ENTITY_COUNT / 2);
    assert_eq!(health_count, ENTITY_COUNT.div_ceil(3)); // Ceiling division

    // Mass deletion (every 5th entity)
    for i in (0..ENTITY_COUNT).step_by(5) {
        world.delete_entity(entities[i]);
    }

    let expected_remaining = ENTITY_COUNT - (ENTITY_COUNT / 5);
    assert_eq!(world.entities().count(), expected_remaining);

    // Cleanup
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), expected_remaining);

    // Verify remaining entities
    for (i, &entity) in entities.iter().enumerate() {
        if i % 5 == 0 {
            assert!(!world.has_component::<Counter>(entity));
        } else {
            assert!(world.has_component::<Counter>(entity));
        }
    }
}

#[test]
fn test_world_state_consistency() {
    let mut world = World::new();

    // Create complex entity relationships
    let mut entities = Vec::new();
    for _i in 0..50 {
        let entity = world.spawn_entity();
        entities.push(entity);
    }

    // Add components in patterns
    for (i, &entity) in entities.iter().enumerate() {
        // All entities get a counter
        world
            .add_component(entity, Counter { value: i as i64 })
            .unwrap();

        // Create chains of components
        if i > 0 {
            world
                .add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: (i - 1) as f32,
                    },
                )
                .unwrap();
        }

        if i > 1 {
            world
                .add_component(entity, Velocity { x: 1.0, y: 0.0 })
                .unwrap();
        }

        if i > 2 {
            world
                .add_component(
                    entity,
                    Health {
                        current: 100,
                        max: 100,
                    },
                )
                .unwrap();
        }
    }

    // Perform complex operations
    for (i, &entity) in entities.iter().enumerate() {
        if i % 7 == 0 && i > 0 {
            // Remove position from some entities
            world.remove_component::<Position>(entity);
        }

        if i % 11 == 0 && i > 2 {
            // Update health for some entities
            world
                .update_component::<Health, _>(entity, |mut health| {
                    health.current = health.current.saturating_sub(25);
                    health
                })
                .ok();
        }

        if i % 13 == 0 {
            // Replace counter for some entities
            world.replace_component(entity, Counter { value: -(i as i64) });
        }
    }

    // Verify consistency
    for (i, &entity) in entities.iter().enumerate() {
        // All entities should still have a counter
        assert!(world.has_component::<Counter>(entity));

        let counter = world.get_component::<Counter>(entity).unwrap();
        if i % 13 == 0 {
            assert_eq!(counter.value, -(i as i64));
        } else {
            assert_eq!(counter.value, i as i64);
        }

        // Position should exist unless removed
        if i > 0 && i % 7 != 0 {
            assert!(world.has_component::<Position>(entity));
        } else if i % 7 == 0 && i > 0 {
            assert!(!world.has_component::<Position>(entity));
        }

        // Velocity patterns
        if i > 1 {
            assert!(world.has_component::<Velocity>(entity));
        }

        // Health patterns with updates
        if i > 2 {
            assert!(world.has_component::<Health>(entity));
            let health = world.get_component::<Health>(entity).unwrap();
            if i % 11 == 0 {
                assert_eq!(health.current, 75); // 100 - 25
            } else {
                assert_eq!(health.current, 100);
            }
        }
    }

    assert_eq!(world.entities().count(), 50);
}
