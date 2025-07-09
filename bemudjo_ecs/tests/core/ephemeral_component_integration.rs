//! Ephemeral Component Integration Tests
//!
//! Tests focused on ephemeral component behavior across the entire ECS system,
//! including interaction with regular components, entities, and world operations.

use bemudjo_ecs::{Component, World};

// Test Components
#[derive(Clone, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Clone, Debug, PartialEq)]
struct DamageEvent {
    amount: u32,
    source: String,
}
impl Component for DamageEvent {}

#[derive(Clone, Debug, PartialEq)]
struct HealEvent {
    amount: u32,
}
impl Component for HealEvent {}

#[derive(Clone, Debug, PartialEq)]
struct CollisionEvent {
    other_entity: u32, // Using u32 for simplicity
}
impl Component for CollisionEvent {}

#[test]
fn test_ephemeral_components_independent_of_regular_components() {
    let mut world = World::new();

    // Create entities with regular components
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 10.0, y: 20.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: 30.0, y: 40.0 })
        .unwrap();

    // Add ephemeral components
    world
        .add_ephemeral_component(
            entity1,
            DamageEvent {
                amount: 50,
                source: "sword".to_string(),
            },
        )
        .unwrap();
    world
        .add_ephemeral_component(entity2, HealEvent { amount: 25 })
        .unwrap();

    // Verify regular components are unaffected
    assert_eq!(world.get_component::<Position>(entity1).unwrap().x, 10.0);
    assert_eq!(world.get_component::<Position>(entity2).unwrap().x, 30.0);

    // Verify ephemeral components exist
    assert!(world.has_ephemeral_component::<DamageEvent>(entity1));
    assert!(world.has_ephemeral_component::<HealEvent>(entity2));
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity2));
    assert!(!world.has_ephemeral_component::<HealEvent>(entity1));

    // Clean ephemeral storage
    world.clean_ephemeral_storage();

    // Regular components still exist
    assert!(world.has_component::<Position>(entity1));
    assert!(world.has_component::<Position>(entity2));

    // Ephemeral components are gone
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity1));
    assert!(!world.has_ephemeral_component::<HealEvent>(entity2));
}

#[test]
fn test_ephemeral_components_with_entity_lifecycle() {
    let mut world = World::new();

    // Create entities
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    // Add regular and ephemeral components
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world
        .add_component(entity3, Position { x: 3.0, y: 3.0 })
        .unwrap();

    world
        .add_ephemeral_component(
            entity1,
            DamageEvent {
                amount: 10,
                source: "fire".to_string(),
            },
        )
        .unwrap();
    world
        .add_ephemeral_component(
            entity2,
            DamageEvent {
                amount: 20,
                source: "ice".to_string(),
            },
        )
        .unwrap();
    world
        .add_ephemeral_component(entity3, HealEvent { amount: 15 })
        .unwrap();

    // Delete entity2
    world.delete_entity(entity2);

    // After deletion - ephemeral components for deleted entities should be gone immediately
    assert!(world.has_ephemeral_component::<DamageEvent>(entity1));
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity2)); // Gone immediately after deletion
    assert!(world.has_ephemeral_component::<HealEvent>(entity3));

    // Cleanup deleted entities
    world.cleanup_deleted_entities();

    // Ephemeral components for deleted entities should still be gone
    assert!(world.has_ephemeral_component::<DamageEvent>(entity1));
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity2)); // Gone after cleanup
    assert!(world.has_ephemeral_component::<HealEvent>(entity3));

    // Clean ephemeral storage
    world.clean_ephemeral_storage();

    // All ephemeral components gone
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity1));
    assert!(!world.has_ephemeral_component::<HealEvent>(entity3));

    // Regular components still exist for living entities
    assert!(world.has_component::<Position>(entity1));
    assert!(!world.has_component::<Position>(entity2)); // Deleted
    assert!(world.has_component::<Position>(entity3));
}

#[test]
fn test_multiple_ephemeral_component_types_per_entity() {
    let mut world = World::new();

    let entity = world.spawn_entity();

    // Add multiple types of ephemeral components to the same entity
    world
        .add_ephemeral_component(
            entity,
            DamageEvent {
                amount: 30,
                source: "lightning".to_string(),
            },
        )
        .unwrap();
    world
        .add_ephemeral_component(entity, HealEvent { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity, CollisionEvent { other_entity: 999 })
        .unwrap();

    // All should exist
    assert!(world.has_ephemeral_component::<DamageEvent>(entity));
    assert!(world.has_ephemeral_component::<HealEvent>(entity));
    assert!(world.has_ephemeral_component::<CollisionEvent>(entity));

    // Verify we can get them
    let damage = world
        .get_ephemeral_component::<DamageEvent>(entity)
        .unwrap();
    assert_eq!(damage.amount, 30);
    assert_eq!(damage.source, "lightning");

    let heal = world.get_ephemeral_component::<HealEvent>(entity).unwrap();
    assert_eq!(heal.amount, 10);

    let collision = world
        .get_ephemeral_component::<CollisionEvent>(entity)
        .unwrap();
    assert_eq!(collision.other_entity, 999);

    // Clean storage
    world.clean_ephemeral_storage();

    // All should be gone
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity));
    assert!(!world.has_ephemeral_component::<HealEvent>(entity));
    assert!(!world.has_ephemeral_component::<CollisionEvent>(entity));
}

#[test]
fn test_ephemeral_component_replacement_behavior() {
    let mut world = World::new();

    let entity = world.spawn_entity();

    // Add an ephemeral component
    world
        .add_ephemeral_component(
            entity,
            DamageEvent {
                amount: 10,
                source: "sword".to_string(),
            },
        )
        .unwrap();

    // Verify it exists
    let damage1 = world
        .get_ephemeral_component::<DamageEvent>(entity)
        .unwrap();
    assert_eq!(damage1.amount, 10);
    assert_eq!(damage1.source, "sword");

    // Replace with a new one
    world
        .add_ephemeral_component(
            entity,
            DamageEvent {
                amount: 25,
                source: "magic".to_string(),
            },
        )
        .unwrap();

    // Should be replaced
    let damage2 = world
        .get_ephemeral_component::<DamageEvent>(entity)
        .unwrap();
    assert_eq!(damage2.amount, 25);
    assert_eq!(damage2.source, "magic");

    // Clean storage
    world.clean_ephemeral_storage();

    // Should be gone
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity));
}

#[test]
fn test_ephemeral_components_large_scale_operations() {
    let mut world = World::new();

    // Create many entities
    let mut entities = Vec::new();
    for i in 0..100 {
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

    // Add ephemeral components to every other entity
    for (i, &entity) in entities.iter().enumerate() {
        if i % 2 == 0 {
            world
                .add_ephemeral_component(
                    entity,
                    DamageEvent {
                        amount: i as u32,
                        source: format!("source_{}", i),
                    },
                )
                .unwrap();
        } else {
            world
                .add_ephemeral_component(entity, HealEvent { amount: i as u32 })
                .unwrap();
        }
    }

    // Verify all ephemeral components exist
    for (i, &entity) in entities.iter().enumerate() {
        if i % 2 == 0 {
            assert!(world.has_ephemeral_component::<DamageEvent>(entity));
            assert!(!world.has_ephemeral_component::<HealEvent>(entity));

            let damage = world
                .get_ephemeral_component::<DamageEvent>(entity)
                .unwrap();
            assert_eq!(damage.amount, i as u32);
        } else {
            assert!(world.has_ephemeral_component::<HealEvent>(entity));
            assert!(!world.has_ephemeral_component::<DamageEvent>(entity));

            let heal = world.get_ephemeral_component::<HealEvent>(entity).unwrap();
            assert_eq!(heal.amount, i as u32);
        }
    }

    // Clean ephemeral storage
    world.clean_ephemeral_storage();

    // All ephemeral components should be gone
    for &entity in &entities {
        assert!(!world.has_ephemeral_component::<DamageEvent>(entity));
        assert!(!world.has_ephemeral_component::<HealEvent>(entity));
    }

    // Regular components should still exist
    for (i, &entity) in entities.iter().enumerate() {
        assert!(world.has_component::<Position>(entity));
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, i as f32);
        assert_eq!(pos.y, i as f32);
    }
}

#[test]
fn test_ephemeral_components_error_conditions() {
    let mut world = World::new();

    let entity = world.spawn_entity();
    let nonexistent_entity = world.spawn_entity();
    world.delete_entity(nonexistent_entity);
    world.cleanup_deleted_entities();

    // Adding to nonexistent entity should fail
    let result = world.add_ephemeral_component(
        nonexistent_entity,
        DamageEvent {
            amount: 10,
            source: "test".to_string(),
        },
    );
    assert!(result.is_err());

    // Getting from nonexistent entity should return None
    assert!(world
        .get_ephemeral_component::<DamageEvent>(nonexistent_entity)
        .is_none());

    // Has ephemeral component on nonexistent entity should return false
    assert!(!world.has_ephemeral_component::<DamageEvent>(nonexistent_entity));

    // Getting nonexistent ephemeral component should return None
    assert!(world
        .get_ephemeral_component::<DamageEvent>(entity)
        .is_none());

    // Has nonexistent ephemeral component should return false
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity));
}
