//! Edge cases and advanced scenarios for query system integration tests
//!
//! These tests cover boundary conditions, error cases, and complex scenarios
//! that might not be covered in basic integration tests.

use bemudjo_ecs::{Component, Query, World};

// Test Components
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
    value: u32,
}
impl Component for Health {}

#[derive(Debug, Clone, PartialEq)]
struct Damage {
    amount: u32,
}
impl Component for Damage {}

#[derive(Debug, Clone, PartialEq)]
struct TempBuff {
    multiplier: f32,
}
impl Component for TempBuff {}

#[derive(Debug, Clone, PartialEq)]
struct StatusEffect {
    effect_type: String,
    duration: f32,
}
impl Component for StatusEffect {}

// Marker components for testing
#[derive(Debug, Clone, PartialEq)]
struct Player;
impl Component for Player {}

#[derive(Debug, Clone, PartialEq)]
struct Enemy;
impl Component for Enemy {}

#[derive(Debug, Clone, PartialEq)]
struct Dead;
impl Component for Dead {}

#[test]
fn test_empty_world_queries() {
    let world = World::new();

    // Test regular queries on empty world
    let position_query = Query::<Position>::new();
    let results: Vec<_> = position_query.iter(&world).collect();
    assert_eq!(results.len(), 0);

    // Test ephemeral queries on empty world
    let damage_query = Query::<Damage>::new();
    let ephemeral_results: Vec<_> = damage_query.iter_ephemeral(&world).collect();
    assert_eq!(ephemeral_results.len(), 0);

    // Test complex queries on empty world
    let complex_query = Query::<Position>::new()
        .with::<Health>()
        .without::<Dead>()
        .with_ephemeral::<Damage>()
        .without_ephemeral::<TempBuff>();
    let complex_results: Vec<_> = complex_query.iter(&world).collect();
    assert_eq!(complex_results.len(), 0);

    let complex_ephemeral_results: Vec<_> = complex_query.iter_ephemeral(&world).collect();
    assert_eq!(complex_ephemeral_results.len(), 0);
}

#[test]
fn test_query_with_no_matching_entities() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();

    // Add components that don't match the query
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(entity2, Velocity { x: 1.0, y: 0.0 })
        .unwrap();

    // Query for Health component (which no entity has)
    let health_query = Query::<Health>::new();
    let results: Vec<_> = health_query.iter(&world).collect();
    assert_eq!(results.len(), 0);

    // Query for ephemeral components (which no entity has)
    let damage_query = Query::<Damage>::new();
    let ephemeral_results: Vec<_> = damage_query.iter_ephemeral(&world).collect();
    assert_eq!(ephemeral_results.len(), 0);
}

#[test]
fn test_query_with_contradictory_filters() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world.add_component(entity1, Health { value: 100 }).unwrap();
    world.add_component(entity1, Dead).unwrap();

    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world.add_component(entity2, Health { value: 50 }).unwrap();

    // Contradictory regular component filters (with and without the same component)
    let contradictory_query = Query::<Position>::new()
        .with::<Health>()
        .without::<Health>();
    let results: Vec<_> = contradictory_query.iter(&world).collect();
    assert_eq!(results.len(), 0); // No entity can have and not have Health at the same time

    // Add ephemeral components
    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity2, TempBuff { multiplier: 1.5 })
        .unwrap();

    // Contradictory ephemeral component filters
    let contradictory_ephemeral_query = Query::<Damage>::new()
        .with_ephemeral::<TempBuff>()
        .without_ephemeral::<TempBuff>();
    let ephemeral_results: Vec<_> = contradictory_ephemeral_query
        .iter_ephemeral(&world)
        .collect();
    assert_eq!(ephemeral_results.len(), 0); // No entity can have and not have TempBuff at the same time
}

#[test]
fn test_query_with_self_referential_filters() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world.add_component(entity2, Health { value: 100 }).unwrap();

    // Query for Position with Position (redundant but valid)
    let redundant_query = Query::<Position>::new().with::<Position>();
    let results: Vec<_> = redundant_query.iter(&world).collect();
    assert_eq!(results.len(), 2); // Both entities have Position

    // Query for Position without Position (contradictory)
    let contradictory_query = Query::<Position>::new().without::<Position>();
    let contradictory_results: Vec<_> = contradictory_query.iter(&world).collect();
    assert_eq!(contradictory_results.len(), 0); // No entity can have Position and not have Position

    // Add ephemeral components
    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity2, Damage { amount: 20 })
        .unwrap();

    // Query for Damage with Damage (redundant but valid)
    let redundant_ephemeral_query = Query::<Damage>::new().with_ephemeral::<Damage>();
    let ephemeral_results: Vec<_> = redundant_ephemeral_query.iter_ephemeral(&world).collect();
    assert_eq!(ephemeral_results.len(), 2); // Both entities have Damage

    // Query for Damage without Damage (contradictory)
    let contradictory_ephemeral_query = Query::<Damage>::new().without_ephemeral::<Damage>();
    let contradictory_ephemeral_results: Vec<_> = contradictory_ephemeral_query
        .iter_ephemeral(&world)
        .collect();
    assert_eq!(contradictory_ephemeral_results.len(), 0); // No entity can have Damage and not have Damage
}

#[test]
fn test_query_with_many_filters() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    // Setup entity1 with many components
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world.add_component(entity1, Health { value: 100 }).unwrap();
    world
        .add_component(entity1, Velocity { x: 1.0, y: 0.0 })
        .unwrap();
    world.add_component(entity1, Player).unwrap();
    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity1, TempBuff { multiplier: 1.5 })
        .unwrap();
    world
        .add_ephemeral_component(
            entity1,
            StatusEffect {
                effect_type: "speed".to_string(),
                duration: 3.0,
            },
        )
        .unwrap();

    // Setup entity2 with some components
    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world.add_component(entity2, Health { value: 50 }).unwrap();
    world.add_component(entity2, Enemy).unwrap();
    world
        .add_ephemeral_component(entity2, Damage { amount: 5 })
        .unwrap();

    // Setup entity3 with minimal components
    world
        .add_component(entity3, Position { x: 3.0, y: 3.0 })
        .unwrap();
    world.add_component(entity3, Dead).unwrap();

    // Complex query with many filters
    let complex_query = Query::<Position>::new()
        .with::<Health>()
        .with::<Velocity>()
        .with::<Player>()
        .without::<Enemy>()
        .without::<Dead>()
        .with_ephemeral::<Damage>()
        .with_ephemeral::<TempBuff>()
        .with_ephemeral::<StatusEffect>()
        .without_ephemeral::<StatusEffect>(); // Contradictory filter

    let results: Vec<_> = complex_query.iter(&world).collect();
    assert_eq!(results.len(), 0); // Contradictory ephemeral filter makes this impossible

    // Non-contradictory complex query
    let valid_complex_query = Query::<Position>::new()
        .with::<Health>()
        .with::<Velocity>()
        .with::<Player>()
        .without::<Enemy>()
        .without::<Dead>()
        .with_ephemeral::<Damage>()
        .with_ephemeral::<TempBuff>()
        .with_ephemeral::<StatusEffect>();

    let valid_results: Vec<_> = valid_complex_query.iter(&world).collect();
    assert_eq!(valid_results.len(), 1); // Only entity1 matches all criteria
    assert_eq!(valid_results[0].0, entity1);
}

#[test]
fn test_query_with_deleted_entities_edge_cases() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    // Add components
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
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity2, Damage { amount: 20 })
        .unwrap();
    world
        .add_ephemeral_component(entity3, Damage { amount: 30 })
        .unwrap();

    let position_query = Query::<Position>::new();
    let damage_query = Query::<Damage>::new();

    // Initial state
    assert_eq!(position_query.iter(&world).count(), 3);
    assert_eq!(damage_query.iter_ephemeral(&world).count(), 3);

    // Delete entity in the middle
    world.delete_entity(entity2);
    assert_eq!(position_query.iter(&world).count(), 2);
    assert_eq!(damage_query.iter_ephemeral(&world).count(), 2);

    // Delete first entity
    world.delete_entity(entity1);
    assert_eq!(position_query.iter(&world).count(), 1);
    assert_eq!(damage_query.iter_ephemeral(&world).count(), 1);

    // Delete last entity
    world.delete_entity(entity3);
    assert_eq!(position_query.iter(&world).count(), 0);
    assert_eq!(damage_query.iter_ephemeral(&world).count(), 0);

    // Queries should return empty results
    let position_results: Vec<_> = position_query.iter(&world).collect();
    assert_eq!(position_results.len(), 0);

    let damage_results: Vec<_> = damage_query.iter_ephemeral(&world).collect();
    assert_eq!(damage_results.len(), 0);
}

#[test]
fn test_query_with_component_removal_edge_cases() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();

    // Add components
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world.add_component(entity1, Health { value: 100 }).unwrap();
    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world.add_component(entity2, Health { value: 50 }).unwrap();

    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity2, Damage { amount: 20 })
        .unwrap();

    let healthy_query = Query::<Health>::new().with::<Position>();
    let damaged_query = Query::<Damage>::new().with::<Position>();

    // Initial state
    assert_eq!(healthy_query.iter(&world).count(), 2);
    assert_eq!(damaged_query.iter_ephemeral(&world).count(), 2);

    // Remove Position from entity1
    world.remove_component::<Position>(entity1);
    assert_eq!(healthy_query.iter(&world).count(), 1); // entity1 no longer matches
    assert_eq!(damaged_query.iter_ephemeral(&world).count(), 1); // entity1 no longer matches

    // Remove Health from entity2
    world.remove_component::<Health>(entity2);
    assert_eq!(healthy_query.iter(&world).count(), 0); // entity2 no longer matches
    assert_eq!(damaged_query.iter_ephemeral(&world).count(), 1); // entity2 still matches (has Position and Damage)

    // Clean ephemeral storage (simulating end of frame cleanup)
    world.clean_ephemeral_storage();
    assert_eq!(damaged_query.iter_ephemeral(&world).count(), 0); // No entities match after cleanup

    // Add components back
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world.add_component(entity2, Health { value: 25 }).unwrap();
    world
        .add_ephemeral_component(entity1, Damage { amount: 5 })
        .unwrap();

    assert_eq!(healthy_query.iter(&world).count(), 2); // Both entities match again
    assert_eq!(damaged_query.iter_ephemeral(&world).count(), 1); // Only entity1 matches
}

#[test]
fn test_query_deduplication_edge_cases() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world.add_component(entity1, Health { value: 100 }).unwrap();
    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();

    // Test multiple .with() calls for the same component (should deduplicate)
    let redundant_query = Query::<Position>::new()
        .with::<Health>()
        .with::<Health>()
        .with::<Health>();
    let results: Vec<_> = redundant_query.iter(&world).collect();
    assert_eq!(results.len(), 1); // Should still find entity1

    // Test multiple .without() calls for the same component (should deduplicate)
    let redundant_without_query = Query::<Position>::new()
        .without::<Enemy>()
        .without::<Enemy>()
        .without::<Enemy>();
    let without_results: Vec<_> = redundant_without_query.iter(&world).collect();
    assert_eq!(without_results.len(), 1); // Should still find entity1

    // Test multiple ephemeral .with_ephemeral() calls
    let redundant_ephemeral_query = Query::<Damage>::new()
        .with_ephemeral::<Damage>()
        .with_ephemeral::<Damage>();
    let ephemeral_results: Vec<_> = redundant_ephemeral_query.iter_ephemeral(&world).collect();
    assert_eq!(ephemeral_results.len(), 1); // Should still find entity1

    // Test multiple ephemeral .without_ephemeral() calls
    let redundant_ephemeral_without_query = Query::<Damage>::new()
        .without_ephemeral::<TempBuff>()
        .without_ephemeral::<TempBuff>();
    let ephemeral_without_results: Vec<_> = redundant_ephemeral_without_query
        .iter_ephemeral(&world)
        .collect();
    assert_eq!(ephemeral_without_results.len(), 1); // Should still find entity1
}

#[test]
fn test_query_with_mixed_component_types() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    // Mix of regular and ephemeral components
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world.add_component(entity1, Health { value: 100 }).unwrap();
    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity1, TempBuff { multiplier: 1.5 })
        .unwrap();

    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world
        .add_ephemeral_component(entity2, Damage { amount: 20 })
        .unwrap();

    world.add_component(entity3, Health { value: 50 }).unwrap();
    world
        .add_ephemeral_component(entity3, TempBuff { multiplier: 2.0 })
        .unwrap();

    // Query regular components filtered by ephemeral components
    let regular_with_ephemeral_query = Query::<Position>::new()
        .with_ephemeral::<Damage>()
        .without_ephemeral::<TempBuff>();
    let results: Vec<_> = regular_with_ephemeral_query.iter(&world).collect();
    assert_eq!(results.len(), 1); // Only entity2 has Position and Damage but not TempBuff
    assert_eq!(results[0].0, entity2);

    // Query ephemeral components filtered by regular components
    let ephemeral_with_regular_query = Query::<TempBuff>::new()
        .with::<Health>()
        .without::<Position>();
    let ephemeral_results: Vec<_> = ephemeral_with_regular_query
        .iter_ephemeral(&world)
        .collect();
    assert_eq!(ephemeral_results.len(), 1); // Only entity3 has TempBuff and Health but not Position
    assert_eq!(ephemeral_results[0].0, entity3);

    // Complex mixed query
    let complex_mixed_query = Query::<Health>::new()
        .with::<Position>()
        .with_ephemeral::<Damage>()
        .with_ephemeral::<TempBuff>();
    let complex_results: Vec<_> = complex_mixed_query.iter(&world).collect();
    assert_eq!(complex_results.len(), 1); // Only entity1 matches all criteria
    assert_eq!(complex_results[0].0, entity1);
}

#[test]
fn test_query_iterator_edge_cases() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity2, Damage { amount: 20 })
        .unwrap();

    let position_query = Query::<Position>::new();
    let damage_query = Query::<Damage>::new();

    // Test multiple iterator consumption
    let first_iter_count = position_query.iter(&world).count();
    let second_iter_count = position_query.iter(&world).count();
    assert_eq!(first_iter_count, second_iter_count);
    assert_eq!(first_iter_count, 2);

    // Test ephemeral iterator consumption
    let first_ephemeral_count = damage_query.iter_ephemeral(&world).count();
    let second_ephemeral_count = damage_query.iter_ephemeral(&world).count();
    assert_eq!(first_ephemeral_count, second_ephemeral_count);
    assert_eq!(first_ephemeral_count, 2);

    // Test iterator chaining
    let chained_result: Vec<f32> = position_query
        .iter(&world)
        .map(|(_, pos)| pos.x)
        .filter(|&x| x > 1.5)
        .collect();
    assert_eq!(chained_result.len(), 1);
    assert_eq!(chained_result[0], 2.0);

    // Test empty iterator operations
    let empty_query = Query::<Health>::new();
    let empty_result: Vec<_> = empty_query.iter(&world).collect();
    assert_eq!(empty_result.len(), 0);

    let empty_fold_result = empty_query.iter(&world).fold(0, |acc, _| acc + 1);
    assert_eq!(empty_fold_result, 0);

    let empty_any_result = empty_query.iter(&world).any(|_| true);
    assert!(!empty_any_result);
}

#[test]
fn test_query_with_extreme_entity_counts() {
    let mut world = World::new();

    // Test with zero entities
    let position_query = Query::<Position>::new();
    assert_eq!(position_query.iter(&world).count(), 0);

    // Test with single entity
    let entity = world.spawn_entity();
    world
        .add_component(entity, Position { x: 1.0, y: 1.0 })
        .unwrap();
    assert_eq!(position_query.iter(&world).count(), 1);

    // Test with many entities (but not too many for the test to be slow)
    let mut entities = Vec::new();
    for i in 0..100 {
        let e = world.spawn_entity();
        entities.push(e);
        world
            .add_component(
                e,
                Position {
                    x: i as f32,
                    y: i as f32,
                },
            )
            .unwrap();
    }

    assert_eq!(position_query.iter(&world).count(), 101); // 1 + 100

    // Delete all entities
    for entity in entities {
        world.delete_entity(entity);
    }
    world.delete_entity(entity);

    assert_eq!(position_query.iter(&world).count(), 0);
}
