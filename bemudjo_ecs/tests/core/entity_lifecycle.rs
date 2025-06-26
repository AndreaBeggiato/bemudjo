//! Entity Lifecycle Integration Tests
//!
//! Tests focused on entity creation, deletion, and management
//! operations across the entity lifecycle.

use bemudjo_ecs::{Component, World};

// Test Components
#[derive(Clone, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Clone, Debug, PartialEq)]
struct Health {
    value: u32,
}
impl Component for Health {}

#[derive(Clone, Debug, PartialEq)]
struct Tag {
    name: String,
}
impl Component for Tag {}

#[test]
fn test_entity_spawn_delete_cycle() {
    let mut world = World::new();

    // Spawn entities
    let mut entities = Vec::new();
    for i in 0..10 {
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
        entities.push(entity);
    }

    assert_eq!(world.entities().count(), 10);

    // Delete every other entity
    for i in (0..10).step_by(2) {
        world.delete_entity(entities[i]);
    }

    // Before cleanup, count includes deleted entities
    assert_eq!(world.entities().count(), 5);

    // Verify deleted entities don't have components
    for i in (0..10).step_by(2) {
        assert!(!world.has_component::<Position>(entities[i]));
    }

    // Verify remaining entities still have components
    for i in (1..10).step_by(2) {
        assert!(world.has_component::<Position>(entities[i]));
    }

    // Cleanup
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 5);
}

#[test]
fn test_entity_reuse_after_deletion() {
    let mut world = World::new();

    // Create and track entity IDs
    let mut created_entities = Vec::new();

    for cycle in 0..5 {
        // Create entities
        let mut cycle_entities = Vec::new();
        for i in 0..10 {
            let entity = world.spawn_entity();
            world
                .add_component(
                    entity,
                    Tag {
                        name: format!("Cycle{}_{}", cycle, i),
                    },
                )
                .unwrap();
            cycle_entities.push(entity);
            created_entities.push(entity);
        }

        // Delete all entities from this cycle
        for entity in cycle_entities {
            world.delete_entity(entity);
        }

        world.cleanup_deleted_entities();
        assert_eq!(world.entities().count(), 0);
    }

    // Verify all entities are properly cleaned up
    for &entity in &created_entities {
        assert!(!world.has_component::<Tag>(entity));
        assert!(world.get_component::<Tag>(entity).is_none());
    }
}

#[test]
fn test_entity_lifecycle_with_multiple_components() {
    let mut world = World::new();

    let entity = world.spawn_entity();

    // Add components over time
    world
        .add_component(entity, Position { x: 1.0, y: 1.0 })
        .unwrap();
    assert_eq!(world.entities().count(), 1);

    world.add_component(entity, Health { value: 100 }).unwrap();
    assert_eq!(world.entities().count(), 1);

    world
        .add_component(
            entity,
            Tag {
                name: "Player".to_string(),
            },
        )
        .unwrap();
    assert_eq!(world.entities().count(), 1);

    // Verify all components exist
    assert!(world.has_component::<Position>(entity));
    assert!(world.has_component::<Health>(entity));
    assert!(world.has_component::<Tag>(entity));

    // Remove components one by one
    world.remove_component::<Health>(entity);
    assert!(!world.has_component::<Health>(entity));
    assert!(world.has_component::<Position>(entity));
    assert!(world.has_component::<Tag>(entity));
    assert_eq!(world.entities().count(), 1);

    world.remove_component::<Position>(entity);
    assert!(!world.has_component::<Position>(entity));
    assert!(world.has_component::<Tag>(entity));
    assert_eq!(world.entities().count(), 1);

    world.remove_component::<Tag>(entity);
    assert!(!world.has_component::<Tag>(entity));
    assert_eq!(world.entities().count(), 1); // Entity still exists, just no components

    // Delete entity
    world.delete_entity(entity);
    assert_eq!(world.entities().count(), 0);
}

#[test]
fn test_mass_entity_creation_deletion() {
    let mut world = World::new();

    const ENTITY_COUNT: usize = 1000;
    let mut entities = Vec::new();

    // Mass creation
    for i in 0..ENTITY_COUNT {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: (i * 2) as f32,
                },
            )
            .unwrap();
        world
            .add_component(
                entity,
                Health {
                    value: (i % 100) as u32,
                },
            )
            .unwrap();
        entities.push(entity);
    }

    assert_eq!(world.entities().count(), ENTITY_COUNT);

    // Verify all entities exist with correct components
    for (i, &entity) in entities.iter().enumerate() {
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, i as f32);
        assert_eq!(pos.y, (i * 2) as f32);

        let health = world.get_component::<Health>(entity).unwrap();
        assert_eq!(health.value, (i % 100) as u32);
    }

    // Mass deletion in chunks
    for chunk in entities.chunks(100) {
        for &entity in chunk {
            world.delete_entity(entity);
        }

        // Periodic cleanup
        if chunk.len() > 0 {
            world.cleanup_deleted_entities();
        }
    }

    // Final cleanup
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 0);

    // Verify all entities are gone
    for &entity in &entities {
        assert!(!world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));
    }
}

#[test]
fn test_entity_deletion_with_partial_cleanup() {
    let mut world = World::new();

    // Create entities
    let mut entities = Vec::new();
    for i in 0..20 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Tag {
                    name: format!("Entity{}", i),
                },
            )
            .unwrap();
        entities.push(entity);
    }

    assert_eq!(world.entities().count(), 20);

    // Delete some entities but don't cleanup
    for i in 0..10 {
        world.delete_entity(entities[i]);
    }

    assert_eq!(world.entities().count(), 10);

    // Add components to remaining entities
    for i in 10..20 {
        world
            .add_component(
                entities[i],
                Position {
                    x: i as f32,
                    y: i as f32,
                },
            )
            .unwrap();
    }

    // Verify state before cleanup
    for i in 0..10 {
        assert!(!world.has_component::<Tag>(entities[i]));
    }
    for i in 10..20 {
        assert!(world.has_component::<Tag>(entities[i]));
        assert!(world.has_component::<Position>(entities[i]));
    }

    // Cleanup and verify final state
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 10);

    for i in 10..20 {
        assert!(world.has_component::<Tag>(entities[i]));
        assert!(world.has_component::<Position>(entities[i]));
    }
}

#[test]
fn test_entity_lifecycle_stress() {
    let mut world = World::new();

    // Stress test with rapid creation/deletion cycles
    for cycle in 0..100 {
        // Create entities
        let mut cycle_entities = Vec::new();
        for i in 0..50 {
            let entity = world.spawn_entity();
            world
                .add_component(
                    entity,
                    Position {
                        x: (cycle * 50 + i) as f32,
                        y: 0.0,
                    },
                )
                .unwrap();
            cycle_entities.push(entity);
        }

        // Delete random entities
        for i in (0..50).step_by(3) {
            if i < cycle_entities.len() {
                world.delete_entity(cycle_entities[i]);
            }
        }

        // Add more components to surviving entities
        for &entity in &cycle_entities {
            if world.has_component::<Position>(entity) {
                world
                    .add_component(
                        entity,
                        Health {
                            value: cycle as u32,
                        },
                    )
                    .ok();
            }
        }

        // Cleanup every 10 cycles
        if cycle % 10 == 9 {
            world.cleanup_deleted_entities();
        }

        // Delete all remaining entities
        for entity in cycle_entities {
            world.delete_entity(entity);
        }
    }

    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 0);
}

#[test]
fn test_entity_uniqueness() {
    let mut world = World::new();

    let mut all_entities = std::collections::HashSet::new();

    // Create many entities and verify they're all unique
    for _ in 0..1000 {
        let entity = world.spawn_entity();
        assert!(
            all_entities.insert(entity),
            "Entity ID was not unique: {:?}",
            entity
        );

        world
            .add_component(entity, Position { x: 0.0, y: 0.0 })
            .unwrap();
    }

    assert_eq!(world.entities().count(), 1000);
    assert_eq!(all_entities.len(), 1000);

    // Delete half and create more
    let entities_vec: Vec<_> = all_entities.iter().cloned().collect();
    for i in (0..1000).step_by(2) {
        world.delete_entity(entities_vec[i]);
    }

    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 500);

    // Create new entities and verify they're still unique
    for _ in 0..500 {
        let entity = world.spawn_entity();
        assert!(
            all_entities.insert(entity),
            "New entity ID was not unique: {:?}",
            entity
        );

        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
    }

    assert_eq!(world.entities().count(), 1000);
    assert_eq!(all_entities.len(), 1500); // 1000 original + 500 new
}

#[test]
fn test_entity_iteration_consistency() {
    let mut world = World::new();

    // Create entities
    let mut entities = Vec::new();
    for i in 0..10 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Tag {
                    name: format!("Entity{}", i),
                },
            )
            .unwrap();
        entities.push(entity);
    }

    // Verify iteration includes all entities
    let iterated_entities: Vec<_> = world.entities().cloned().collect();
    let sorted_entities = entities.clone();

    // Just check length since Entity doesn't implement Ord
    assert_eq!(iterated_entities.len(), sorted_entities.len());

    // Verify all entities are present (can't sort Entity types)
    for entity in &entities {
        assert!(iterated_entities.contains(entity));
    }

    // Delete some entities
    world.delete_entity(entities[2]);
    world.delete_entity(entities[5]);
    world.delete_entity(entities[8]);

    // Verify iteration excludes deleted entities
    let remaining_entities: Vec<_> = world.entities().cloned().collect();
    assert_eq!(remaining_entities.len(), 7);

    for &entity in &remaining_entities {
        assert!(entities.contains(&entity));
        assert!(entity != entities[2] && entity != entities[5] && entity != entities[8]);
    }

    // Cleanup and verify again
    world.cleanup_deleted_entities();
    let final_entities: Vec<_> = world.entities().cloned().collect();
    assert_eq!(final_entities.len(), 7);
    assert_eq!(final_entities, remaining_entities);
}
