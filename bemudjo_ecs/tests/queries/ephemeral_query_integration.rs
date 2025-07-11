//! Integration tests for ephemeral query functionality
//!
//! These tests validate the ephemeral query system's integration with the World,
//! entity lifecycle, and real-world usage patterns including mixed regular and
//! ephemeral component queries.

use bemudjo_ecs::{Component, Query, World};

// Test Components (Regular)
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

// Test Components (Ephemeral)
#[derive(Debug, Clone, PartialEq)]
struct Damage {
    amount: u32,
}
impl Component for Damage {}

#[derive(Debug, Clone, PartialEq)]
struct Explosion {
    radius: f32,
}
impl Component for Explosion {}

#[derive(Debug, Clone, PartialEq)]
struct StatusEffect {
    effect_type: String,
    duration: f32,
}
impl Component for StatusEffect {}

#[derive(Debug, Clone, PartialEq)]
struct TempBuff {
    multiplier: f32,
}
impl Component for TempBuff {}

#[test]
fn test_basic_ephemeral_query_integration() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    // Add regular components
    world
        .add_component(entity1, Position { x: 1.0, y: 2.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: 3.0, y: 4.0 })
        .unwrap();
    world
        .add_component(entity3, Position { x: 5.0, y: 6.0 })
        .unwrap();

    // Add ephemeral components
    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity2, Explosion { radius: 5.0 })
        .unwrap();
    world
        .add_ephemeral_component(
            entity3,
            StatusEffect {
                effect_type: "poison".to_string(),
                duration: 3.0,
            },
        )
        .unwrap();

    // Test basic ephemeral query
    let damage_query = Query::<Damage>::new();
    let damage_results: Vec<_> = damage_query.iter_ephemeral(&world).collect();
    assert_eq!(damage_results.len(), 1);
    assert_eq!(damage_results[0].0, entity1);
    assert_eq!(damage_results[0].1.amount, 10);

    // Test ephemeral query with filtering by regular components
    let positioned_damage_query = Query::<Damage>::new().with::<Position>();
    let positioned_damage_results: Vec<_> =
        positioned_damage_query.iter_ephemeral(&world).collect();
    assert_eq!(positioned_damage_results.len(), 1);
    assert_eq!(positioned_damage_results[0].0, entity1);

    // Test ephemeral query with filtering by ephemeral components
    let explosion_query = Query::<Explosion>::new().with_ephemeral::<StatusEffect>();
    let explosion_results: Vec<_> = explosion_query.iter_ephemeral(&world).collect();
    assert_eq!(explosion_results.len(), 0); // No entity has both Explosion and StatusEffect

    // Test count
    assert_eq!(damage_query.iter_ephemeral(&world).count(), 1);

    let explosion_query = Query::<Explosion>::new();
    assert_eq!(explosion_query.iter_ephemeral(&world).count(), 1);

    // Test first method
    let first_damage = damage_query.iter_ephemeral(&world).next();
    assert!(first_damage.is_some());
    assert_eq!(first_damage.unwrap().0, entity1);
}

#[test]
fn test_mixed_regular_ephemeral_filtering() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();
    let entity4 = world.spawn_entity();

    // Setup regular components
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world.add_component(entity1, Health { value: 100 }).unwrap();

    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world
        .add_component(entity2, Velocity { x: 1.0, y: 0.0 })
        .unwrap();

    world
        .add_component(entity3, Position { x: 3.0, y: 3.0 })
        .unwrap();
    world.add_component(entity3, Health { value: 50 }).unwrap();
    world
        .add_component(entity3, Velocity { x: 0.0, y: 1.0 })
        .unwrap();

    world.add_component(entity4, Health { value: 25 }).unwrap();

    // Setup ephemeral components
    world
        .add_ephemeral_component(entity1, Damage { amount: 20 })
        .unwrap();
    world
        .add_ephemeral_component(entity1, TempBuff { multiplier: 1.5 })
        .unwrap();

    world
        .add_ephemeral_component(entity2, Explosion { radius: 3.0 })
        .unwrap();

    world
        .add_ephemeral_component(
            entity3,
            StatusEffect {
                effect_type: "slow".to_string(),
                duration: 2.0,
            },
        )
        .unwrap();

    world
        .add_ephemeral_component(entity4, Damage { amount: 15 })
        .unwrap();

    // Test: Query for positioned entities with damage (regular + ephemeral filtering)
    let positioned_damage_query = Query::<Damage>::new().with::<Position>();
    let positioned_damage_results: Vec<_> =
        positioned_damage_query.iter_ephemeral(&world).collect();
    assert_eq!(positioned_damage_results.len(), 1); // Only entity1 has both Position and Damage
    assert_eq!(positioned_damage_results[0].0, entity1);

    // Test: Query for healthy entities with temporary buffs (regular + ephemeral filtering)
    let healthy_buffed_query = Query::<TempBuff>::new().with::<Health>();
    let healthy_buffed_results: Vec<_> = healthy_buffed_query.iter_ephemeral(&world).collect();
    assert_eq!(healthy_buffed_results.len(), 1); // Only entity1 has both Health and TempBuff
    assert_eq!(healthy_buffed_results[0].0, entity1);

    // Test: Query for entities without velocity but with ephemeral effects
    let stationary_affected_query = Query::<StatusEffect>::new().without::<Velocity>();
    let stationary_affected_results: Vec<_> =
        stationary_affected_query.iter_ephemeral(&world).collect();
    assert_eq!(stationary_affected_results.len(), 0); // entity3 has StatusEffect but also has Velocity

    // Test: Query for damage without explosion (ephemeral without ephemeral)
    let damage_no_explosion_query = Query::<Damage>::new().without_ephemeral::<Explosion>();
    let damage_no_explosion_results: Vec<_> =
        damage_no_explosion_query.iter_ephemeral(&world).collect();
    assert_eq!(damage_no_explosion_results.len(), 2); // entity1 and entity4 have Damage but not Explosion

    // Test: Complex filtering - positioned, healthy, damaged, but not exploding
    let complex_query = Query::<Damage>::new()
        .with::<Position>()
        .with::<Health>()
        .without_ephemeral::<Explosion>();
    let complex_results: Vec<_> = complex_query.iter_ephemeral(&world).collect();
    assert_eq!(complex_results.len(), 1); // Only entity1 matches all criteria
    assert_eq!(complex_results[0].0, entity1);
}

#[test]
fn test_ephemeral_query_with_entity_lifecycle() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    // Add ephemeral components
    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity2, Damage { amount: 20 })
        .unwrap();
    world
        .add_ephemeral_component(entity3, Damage { amount: 30 })
        .unwrap();

    let damage_query = Query::<Damage>::new();

    // Initially, all 3 entities should be found
    let initial_results: Vec<_> = damage_query.iter_ephemeral(&world).collect();
    assert_eq!(initial_results.len(), 3);

    // Delete one entity
    world.delete_entity(entity2);

    // Query should now only find 2 entities
    let after_delete_results: Vec<_> = damage_query.iter_ephemeral(&world).collect();
    assert_eq!(after_delete_results.len(), 2);
    let found_entities: Vec<_> = after_delete_results.iter().map(|(e, _)| *e).collect();
    assert!(found_entities.contains(&entity1));
    assert!(!found_entities.contains(&entity2)); // Deleted entity not found
    assert!(found_entities.contains(&entity3));

    // Clean ephemeral storage (simulating end of frame cleanup)
    world.clean_ephemeral_storage();

    // Query should now find no entities since all ephemeral components are gone
    let after_cleanup: Vec<_> = damage_query.iter_ephemeral(&world).collect();
    assert_eq!(after_cleanup.len(), 0);

    // Add ephemeral component back to entity3
    world
        .add_ephemeral_component(entity3, Damage { amount: 40 })
        .unwrap();

    // Query should now find 1 entity (only entity3 has ephemeral component)
    let after_component_add: Vec<_> = damage_query.iter_ephemeral(&world).collect();
    assert_eq!(after_component_add.len(), 1);
    let final_entities: Vec<_> = after_component_add.iter().map(|(e, _)| *e).collect();
    assert!(final_entities.contains(&entity3));
    assert!(!final_entities.contains(&entity1)); // entity1 no longer has ephemeral component
}

#[test]
fn test_ephemeral_query_iterator_combinators() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    world
        .add_ephemeral_component(entity1, Damage { amount: 10 })
        .unwrap();
    world
        .add_ephemeral_component(entity2, Damage { amount: 25 })
        .unwrap();
    world
        .add_ephemeral_component(entity3, Damage { amount: 5 })
        .unwrap();

    let damage_query = Query::<Damage>::new();

    // Test filter: only high damage
    let high_damage: Vec<_> = damage_query
        .iter_ephemeral(&world)
        .filter(|(_, damage)| damage.amount > 15)
        .collect();
    assert_eq!(high_damage.len(), 1);
    assert_eq!(high_damage[0].1.amount, 25);

    // Test map: extract damage amounts
    let damage_amounts: Vec<u32> = damage_query
        .iter_ephemeral(&world)
        .map(|(_, damage)| damage.amount)
        .collect();
    assert_eq!(damage_amounts.len(), 3);
    assert!(damage_amounts.contains(&10));
    assert!(damage_amounts.contains(&25));
    assert!(damage_amounts.contains(&5));

    // Test find: first entity with damage > 20
    let lethal_damage = damage_query
        .iter_ephemeral(&world)
        .find(|(_, damage)| damage.amount > 20);
    assert!(lethal_damage.is_some());
    assert_eq!(lethal_damage.unwrap().1.amount, 25);

    // Test fold: total damage
    let total_damage: u32 = damage_query
        .iter_ephemeral(&world)
        .fold(0, |acc, (_, damage)| acc + damage.amount);
    assert_eq!(total_damage, 40); // 10 + 25 + 5

    // Test filter_map: double damage for high damage entities
    let doubled_high_damage: Vec<u32> = damage_query
        .iter_ephemeral(&world)
        .filter_map(|(_, damage)| {
            if damage.amount > 15 {
                Some(damage.amount * 2)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(doubled_high_damage.len(), 1);
    assert_eq!(doubled_high_damage[0], 50); // 25 * 2
}

#[test]
fn test_game_simulation_with_ephemeral_queries() {
    let mut world = World::new();

    // Create player entities
    let player1 = world.spawn_entity();
    let player2 = world.spawn_entity();
    let player3 = world.spawn_entity();

    // Create NPC entities
    let npc1 = world.spawn_entity();
    let npc2 = world.spawn_entity();

    // Setup player components
    world
        .add_component(player1, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world.add_component(player1, Health { value: 100 }).unwrap();
    world
        .add_component(player1, Velocity { x: 1.0, y: 0.0 })
        .unwrap();

    world
        .add_component(player2, Position { x: 10.0, y: 10.0 })
        .unwrap();
    world.add_component(player2, Health { value: 75 }).unwrap();

    world
        .add_component(player3, Position { x: 5.0, y: 5.0 })
        .unwrap();
    world.add_component(player3, Health { value: 50 }).unwrap();
    world
        .add_component(player3, Velocity { x: 0.0, y: 1.0 })
        .unwrap();

    // Setup NPC components
    world
        .add_component(npc1, Position { x: 20.0, y: 20.0 })
        .unwrap();
    world.add_component(npc1, Health { value: 200 }).unwrap();

    world
        .add_component(npc2, Position { x: 15.0, y: 15.0 })
        .unwrap();
    world.add_component(npc2, Health { value: 150 }).unwrap();

    // Simulate game events with ephemeral components
    // Player1 takes damage
    world
        .add_ephemeral_component(player1, Damage { amount: 15 })
        .unwrap();

    // Player2 gets a temporary buff
    world
        .add_ephemeral_component(player2, TempBuff { multiplier: 2.0 })
        .unwrap();

    // Player3 is affected by a status effect
    world
        .add_ephemeral_component(
            player3,
            StatusEffect {
                effect_type: "poison".to_string(),
                duration: 5.0,
            },
        )
        .unwrap();

    // NPC1 explodes
    world
        .add_ephemeral_component(npc1, Explosion { radius: 8.0 })
        .unwrap();

    // NPC2 takes damage
    world
        .add_ephemeral_component(npc2, Damage { amount: 25 })
        .unwrap();

    // Query 1: Find all damaged entities (players and NPCs)
    let damaged_entities_query = Query::<Damage>::new().with::<Health>();
    let damaged_entities: Vec<_> = damaged_entities_query.iter_ephemeral(&world).collect();
    assert_eq!(damaged_entities.len(), 2); // player1 and npc2

    // Query 2: Find moving entities with temporary buffs
    let buffed_moving_query = Query::<TempBuff>::new().with::<Velocity>();
    let buffed_moving_results: Vec<_> = buffed_moving_query.iter_ephemeral(&world).collect();
    assert_eq!(buffed_moving_results.len(), 0); // player2 has TempBuff but no Velocity

    // Query 3: Find entities in explosion range (positioned entities near explosions)
    let explosion_query = Query::<Explosion>::new();
    let positioned_query = Query::<Position>::new();

    let explosions: Vec<_> = explosion_query.iter_ephemeral(&world).collect();
    let positioned_entities: Vec<_> = positioned_query.iter(&world).collect();

    assert_eq!(explosions.len(), 1); // Only npc1 has explosion
    assert_eq!(positioned_entities.len(), 5); // All entities have position

    // Query 4: Find entities with status effects that are not exploding
    let status_no_explosion_query = Query::<StatusEffect>::new().without_ephemeral::<Explosion>();
    let status_no_explosion_results: Vec<_> =
        status_no_explosion_query.iter_ephemeral(&world).collect();
    assert_eq!(status_no_explosion_results.len(), 1); // Only player3 has StatusEffect without Explosion

    // Query 5: Complex scenario - find healthy, positioned entities that are not damaged and not exploding
    let safe_entities_query = Query::<Health>::new()
        .with::<Position>()
        .without_ephemeral::<Damage>()
        .without_ephemeral::<Explosion>();
    let safe_entities: Vec<_> = safe_entities_query.iter(&world).collect();
    assert_eq!(safe_entities.len(), 2); // player2 and player3 are safe

    // Verify the safe entities
    let safe_entity_ids: Vec<_> = safe_entities.iter().map(|(e, _)| *e).collect();
    assert!(safe_entity_ids.contains(&player2));
    assert!(safe_entity_ids.contains(&player3));
    assert!(!safe_entity_ids.contains(&player1)); // Has damage
    assert!(!safe_entity_ids.contains(&npc1)); // Has explosion
    assert!(!safe_entity_ids.contains(&npc2)); // Has damage
}

#[test]
fn test_ephemeral_query_performance_characteristics() {
    let mut world = World::new();

    // Create many entities for performance testing
    let mut entities = Vec::new();
    for i in 0..1000 {
        let entity = world.spawn_entity();
        entities.push(entity);

        // Add regular components to all entities
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32,
                },
            )
            .unwrap();
        world.add_component(entity, Health { value: 100 }).unwrap();

        // Add ephemeral components to some entities
        if i % 3 == 0 {
            world
                .add_ephemeral_component(entity, Damage { amount: 10 })
                .unwrap();
        }
        if i % 5 == 0 {
            world
                .add_ephemeral_component(entity, TempBuff { multiplier: 1.5 })
                .unwrap();
        }
        if i % 7 == 0 {
            world
                .add_ephemeral_component(
                    entity,
                    StatusEffect {
                        effect_type: "test".to_string(),
                        duration: 1.0,
                    },
                )
                .unwrap();
        }
    }

    // Test query performance with large datasets
    let damage_query = Query::<Damage>::new();
    let damage_results: Vec<_> = damage_query.iter_ephemeral(&world).collect();
    assert_eq!(damage_results.len(), 334); // 1000 / 3 = 333.33, rounded up to 334

    let buff_query = Query::<TempBuff>::new();
    let buff_results: Vec<_> = buff_query.iter_ephemeral(&world).collect();
    assert_eq!(buff_results.len(), 200); // 1000 / 5 = 200

    let status_query = Query::<StatusEffect>::new();
    let status_results: Vec<_> = status_query.iter_ephemeral(&world).collect();
    assert_eq!(status_results.len(), 143); // 1000 / 7 = 142.857, rounded up to 143

    // Test complex filtering performance
    let complex_query = Query::<Damage>::new()
        .with::<Health>()
        .with::<Position>()
        .without_ephemeral::<TempBuff>();
    let complex_results: Vec<_> = complex_query.iter_ephemeral(&world).collect();

    // Should find entities with damage but without temp buff
    // Entities with damage: every 3rd (334 entities)
    // Entities with temp buff: every 5th (200 entities)
    // Overlap (every 15th): 1000 / 15 = 66.67, rounded up to 67
    // So entities with damage but not temp buff: 334 - 67 = 267
    assert_eq!(complex_results.len(), 267);

    // Test count operations are efficient
    assert_eq!(damage_query.iter_ephemeral(&world).count(), 334);
    assert_eq!(buff_query.iter_ephemeral(&world).count(), 200);
    assert_eq!(status_query.iter_ephemeral(&world).count(), 143);
}
