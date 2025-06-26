//! Component Lifecycle Integration Tests
//!
//! Tests focused on component creation, update, removal, and replacement
//! operations across the component lifecycle.

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
struct Level {
    value: u32,
}
impl Component for Level {}

#[test]
fn test_component_add_remove_cycle() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add component
    world
        .add_component(entity, Position { x: 1.0, y: 2.0 })
        .unwrap();
    assert!(world.has_component::<Position>(entity));

    // Remove component
    let removed = world.remove_component::<Position>(entity);
    assert_eq!(removed, Some(Position { x: 1.0, y: 2.0 }));
    assert!(!world.has_component::<Position>(entity));

    // Add again
    world
        .add_component(entity, Position { x: 3.0, y: 4.0 })
        .unwrap();
    assert!(world.has_component::<Position>(entity));

    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 3.0);
    assert_eq!(pos.y, 4.0);
}

#[test]
fn test_component_replace_cycle() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add initial component
    world
        .add_component(
            entity,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();

    // Replace multiple times
    for i in 1..=5 {
        let old = world.replace_component(
            entity,
            Health {
                current: i * 20,
                max: 100,
            },
        );

        if i == 1 {
            assert_eq!(
                old,
                Some(Health {
                    current: 100,
                    max: 100
                })
            );
        } else {
            assert_eq!(
                old,
                Some(Health {
                    current: (i - 1) * 20,
                    max: 100
                })
            );
        }
    }

    let final_health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(final_health.current, 100); // 5 * 20
}

#[test]
fn test_component_update_cycle() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    world.add_component(entity, Level { value: 1 }).unwrap();

    // Update component multiple times
    for i in 1..=10 {
        let result = world.update_component::<Level, _>(entity, |mut level| {
            level.value += 1;
            level
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, i + 1);
    }

    let final_level = world.get_component::<Level>(entity).unwrap();
    assert_eq!(final_level.value, 11);
}

#[test]
fn test_multiple_components_lifecycle() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add multiple components
    world
        .add_component(entity, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(entity, Velocity { x: 1.0, y: 1.0 })
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

    assert!(world.has_component::<Position>(entity));
    assert!(world.has_component::<Velocity>(entity));
    assert!(world.has_component::<Health>(entity));
    assert!(world.has_component::<Name>(entity));

    // Update some components
    world
        .update_component::<Position, _>(entity, |mut pos| {
            pos.x += 5.0;
            pos.y += 5.0;
            pos
        })
        .unwrap();

    world
        .update_component::<Health, _>(entity, |mut health| {
            health.current -= 25;
            health
        })
        .unwrap();

    // Remove one component
    world.remove_component::<Velocity>(entity);
    assert!(!world.has_component::<Velocity>(entity));

    // Replace another
    world.replace_component(
        entity,
        Name {
            value: "Updated".to_string(),
        },
    );

    // Verify final state
    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 5.0);
    assert_eq!(pos.y, 5.0);

    let health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(health.current, 75);

    let name = world.get_component::<Name>(entity).unwrap();
    assert_eq!(name.value, "Updated");

    assert!(world.has_component::<Position>(entity));
    assert!(!world.has_component::<Velocity>(entity));
    assert!(world.has_component::<Health>(entity));
    assert!(world.has_component::<Name>(entity));
}

#[test]
fn test_component_lifecycle_error_conditions() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Try to add duplicate component
    world
        .add_component(entity, Position { x: 1.0, y: 1.0 })
        .unwrap();

    let result = world.add_component(entity, Position { x: 2.0, y: 2.0 });
    assert!(matches!(
        result,
        Err(ComponentError::ComponentAlreadyExists)
    ));

    // Try to update non-existent component
    let result = world.update_component::<Velocity, _>(entity, |vel| vel);
    assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

    // Try to remove non-existent component
    let result = world.remove_component::<Health>(entity);
    assert_eq!(result, None);

    // Operations on deleted entity
    world.delete_entity(entity);

    let result = world.add_component(entity, Velocity { x: 1.0, y: 1.0 });
    assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

    let result = world.update_component::<Position, _>(entity, |pos| pos);
    assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

    assert!(world.get_component::<Position>(entity).is_none());
    assert!(!world.has_component::<Position>(entity));
}

#[test]
fn test_component_lifecycle_across_entity_deletion() {
    let mut world = World::new();

    // Create multiple entities with same component types
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world
        .add_component(entity3, Position { x: 3.0, y: 3.0 })
        .unwrap();

    // Delete middle entity
    world.delete_entity(entity2);

    // Operations on remaining entities should work normally
    world
        .update_component::<Position, _>(entity1, |mut pos| {
            pos.x += 10.0;
            pos
        })
        .unwrap();

    world.replace_component(entity3, Position { x: 30.0, y: 30.0 });

    // Verify state
    let pos1 = world.get_component::<Position>(entity1).unwrap();
    assert_eq!(pos1.x, 11.0);

    let pos3 = world.get_component::<Position>(entity3).unwrap();
    assert_eq!(pos3.x, 30.0);

    assert!(!world.has_component::<Position>(entity2));
}

#[test]
fn test_component_lifecycle_with_cleanup() {
    let mut world = World::new();

    // Create and delete entities in cycles
    for cycle in 0..5 {
        let mut entities = Vec::new();

        // Create entities
        for i in 0..10 {
            let entity = world.spawn_entity();
            world
                .add_component(
                    entity,
                    Level {
                        value: cycle * 10 + i,
                    },
                )
                .unwrap();
            entities.push(entity);
        }

        assert_eq!(world.entities().count(), 10);

        // Delete half the entities
        for i in (0..10).step_by(2) {
            world.delete_entity(entities[i]);
        }

        assert_eq!(world.entities().count(), 5);

        // Update remaining entities
        for &entity in &entities {
            if world.has_component::<Level>(entity) {
                world
                    .update_component::<Level, _>(entity, |mut level| {
                        level.value += 100;
                        level
                    })
                    .ok();
            }
        }

        // Cleanup and verify
        world.cleanup_deleted_entities();
        assert_eq!(world.entities().count(), 5);

        // Remove all remaining entities
        for &entity in &entities {
            if world.has_component::<Level>(entity) {
                world.delete_entity(entity);
            }
        }

        world.cleanup_deleted_entities();
        assert_eq!(world.entities().count(), 0);
    }
}

#[test]
fn test_component_state_consistency() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Complex sequence of operations
    world
        .add_component(entity, Position { x: 0.0, y: 0.0 })
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

    // Interleaved updates
    for i in 1..=20 {
        // Update position
        world
            .update_component::<Position, _>(entity, |mut pos| {
                pos.x += 1.0;
                pos.y += 0.5;
                pos
            })
            .unwrap();

        // Every 3rd iteration, update health
        if i % 3 == 0 {
            world
                .update_component::<Health, _>(entity, |mut health| {
                    health.current = health.current.saturating_sub(5);
                    health
                })
                .unwrap();
        }

        // Every 5th iteration, replace position
        if i % 5 == 0 {
            world.replace_component(
                entity,
                Position {
                    x: i as f32 * 10.0,
                    y: 0.0,
                },
            );
        }

        // Every 7th iteration, add/remove velocity
        if i % 7 == 0 {
            if world.has_component::<Velocity>(entity) {
                world.remove_component::<Velocity>(entity);
            } else {
                world
                    .add_component(entity, Velocity { x: 1.0, y: 1.0 })
                    .unwrap();
            }
        }
    }

    // Verify final state is consistent
    assert!(world.has_component::<Position>(entity));
    assert!(world.has_component::<Health>(entity));

    let _pos = world.get_component::<Position>(entity).unwrap();
    let health = world.get_component::<Health>(entity).unwrap();

    // Position should be from last replacement (i=20, 200.0, 0.0) or last update
    // Health should have been decremented 6 times (at i=3,6,9,12,15,18): 100 - 30 = 70
    assert_eq!(health.current, 70);

    // Velocity was toggled at i=7 (added) and i=14 (removed): 2 times, so should NOT exist
    assert!(!world.has_component::<Velocity>(entity));
}
