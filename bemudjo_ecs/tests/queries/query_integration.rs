//! Query system integration tests
//!
//! These tests validate the query system's integration with the World,
//! entity lifecycle, and real-world usage patterns.

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
struct Dead;
impl Component for Dead {}

#[test]
fn test_query_iterator_combinators_integration() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 1.0, y: 2.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: -5.0, y: 10.0 })
        .unwrap();
    world
        .add_component(entity3, Position { x: 15.0, y: -3.0 })
        .unwrap();

    let query = Query::<Position>::new();

    // Test filter: only positive x coordinates
    let positive_x: Vec<_> = query.iter(&world).filter(|(_, pos)| pos.x > 0.0).collect();
    assert_eq!(positive_x.len(), 2);

    // Test map: extract just the x coordinates
    let x_coords: Vec<f32> = query.iter(&world).map(|(_, pos)| pos.x).collect();
    assert_eq!(x_coords.len(), 3);
    assert!(x_coords.contains(&1.0));
    assert!(x_coords.contains(&-5.0));
    assert!(x_coords.contains(&15.0));

    // Test find: first entity with y > 5
    let high_y = query.iter(&world).find(|(_, pos)| pos.y > 5.0);
    assert!(high_y.is_some());
    assert_eq!(high_y.unwrap().1.y, 10.0);

    // Test filter_map for coordinate transformation
    let doubled_x: Vec<f32> = query
        .iter(&world)
        .filter_map(|(_, pos)| if pos.x > 0.0 { Some(pos.x * 2.0) } else { None })
        .collect();
    assert_eq!(doubled_x.len(), 2);
    assert!(doubled_x.contains(&2.0)); // 1.0 * 2
    assert!(doubled_x.contains(&30.0)); // 15.0 * 2

    // Test fold for aggregate calculations
    let total_distance_from_origin: f32 = query.iter(&world).fold(0.0, |acc, (_, pos)| {
        acc + (pos.x * pos.x + pos.y * pos.y).sqrt()
    });
    assert!(total_distance_from_origin > 0.0);
}

#[test]
fn test_complex_filtering_scenarios() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    // entity1: Position + Velocity
    world
        .add_component(entity1, Position { x: 1.0, y: 2.0 })
        .unwrap();
    world
        .add_component(entity1, Velocity { x: 0.5, y: 1.0 })
        .unwrap();

    // entity2: Position only
    world
        .add_component(entity2, Position { x: 3.0, y: 4.0 })
        .unwrap();

    // entity3: Velocity only
    world
        .add_component(entity3, Velocity { x: 2.0, y: 0.0 })
        .unwrap();

    // Test basic position query
    let position_query = Query::<Position>::new();
    let position_results: Vec<_> = position_query.iter(&world).collect();
    assert_eq!(position_results.len(), 2); // entity1 and entity2 have Position

    // Test query with filtering - should only return entities with both Position and Velocity
    let filtered_query = Query::<Position>::new().with::<Velocity>();
    let filtered_results: Vec<_> = filtered_query.iter(&world).collect();
    // Should only return entity1 which has both Position and Velocity
    assert_eq!(filtered_results.len(), 1);
    assert_eq!(filtered_results[0].0, entity1);

    // Test query without filtering - ensure exclusion works
    let health_entity = world.spawn_entity();
    world
        .add_component(health_entity, Position { x: 10.0, y: 10.0 })
        .unwrap();
    world
        .add_component(health_entity, Health { value: 100 })
        .unwrap();

    let non_healthy_query = Query::<Position>::new().without::<Health>();
    let non_healthy_results: Vec<_> = non_healthy_query.iter(&world).collect();
    assert_eq!(non_healthy_results.len(), 2); // entity1 and entity2, not health_entity
}

#[test]
fn test_living_vs_dead_entities_filtering() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    // entity1: Health (alive)
    world.add_component(entity1, Health { value: 100 }).unwrap();

    // entity2: Health + Dead
    world.add_component(entity2, Health { value: 0 }).unwrap();
    world.add_component(entity2, Dead).unwrap();

    // entity3: Dead only (no Health)
    world.add_component(entity3, Dead).unwrap();

    // Test basic health query
    let health_query = Query::<Health>::new();
    let health_results: Vec<_> = health_query.iter(&world).collect();
    assert_eq!(health_results.len(), 2); // entity1 and entity2 have Health

    // Test query without filtering - should only return living entities (without Dead component)
    let living_query = Query::<Health>::new().without::<Dead>();
    let living_results: Vec<_> = living_query.iter(&world).collect();
    // Should only return entity1 which has Health but not Dead
    assert_eq!(living_results.len(), 1);
    assert_eq!(living_results[0].0, entity1);
    assert_eq!(living_results[0].1.value, 100);

    // Test counting living entities
    assert_eq!(living_query.count(&world), 1);
    assert!(living_query.any(&world));

    // Test finding first living entity
    let first_living = living_query.first(&world);
    assert!(first_living.is_some());
    assert_eq!(first_living.unwrap().0, entity1);
}

#[test]
fn test_comprehensive_multi_component_filtering() {
    let mut world = World::new();

    // Create entities with different component combinations
    let entity1 = world.spawn_entity(); // Position + Velocity
    let entity2 = world.spawn_entity(); // Position only
    let entity3 = world.spawn_entity(); // Velocity only
    let entity4 = world.spawn_entity(); // Position + Health
    let entity5 = world.spawn_entity(); // Position + Velocity + Dead

    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(entity1, Velocity { x: 1.0, y: 0.0 })
        .unwrap();

    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();

    world
        .add_component(entity3, Velocity { x: 0.0, y: 1.0 })
        .unwrap();

    world
        .add_component(entity4, Position { x: 4.0, y: 4.0 })
        .unwrap();
    world.add_component(entity4, Health { value: 100 }).unwrap();

    world
        .add_component(entity5, Position { x: 5.0, y: 5.0 })
        .unwrap();
    world
        .add_component(entity5, Velocity { x: -1.0, y: -1.0 })
        .unwrap();
    world.add_component(entity5, Dead).unwrap();

    // Test .with() filtering
    let moving_query = Query::<Position>::new().with::<Velocity>();
    let moving_entities = moving_query.iter(&world).collect::<Vec<_>>();
    assert_eq!(moving_entities.len(), 2); // entity1 and entity5
    let moving_ids: Vec<_> = moving_entities.iter().map(|(e, _)| *e).collect();
    assert!(moving_ids.contains(&entity1));
    assert!(moving_ids.contains(&entity5));

    // Test .without() filtering
    let living_query = Query::<Position>::new().without::<Dead>();
    let living_entities = living_query.iter(&world).collect::<Vec<_>>();
    assert_eq!(living_entities.len(), 3); // entity1, entity2, entity4
    let living_ids: Vec<_> = living_entities.iter().map(|(e, _)| *e).collect();
    assert!(living_ids.contains(&entity1));
    assert!(living_ids.contains(&entity2));
    assert!(living_ids.contains(&entity4));
    assert!(!living_ids.contains(&entity5)); // Dead entity excluded

    // Test combined filtering
    let living_moving_query = Query::<Position>::new()
        .with::<Velocity>()
        .without::<Dead>();
    let living_moving_entities = living_moving_query.iter(&world).collect::<Vec<_>>();
    assert_eq!(living_moving_entities.len(), 1); // Only entity1
    assert_eq!(living_moving_entities[0].0, entity1);

    // Test multiple .with() conditions
    let healthy_query = Query::<Position>::new().with::<Health>();
    let healthy_positioned_entities = healthy_query.iter(&world).collect::<Vec<_>>();
    assert_eq!(healthy_positioned_entities.len(), 1); // Only entity4
    assert_eq!(healthy_positioned_entities[0].0, entity4);

    // Test chaining query operations with iterator combinators
    let living_positions: Vec<(f32, f32)> = Query::<Position>::new()
        .without::<Dead>()
        .iter(&world)
        .map(|(_, pos)| (pos.x, pos.y))
        .collect();
    assert_eq!(living_positions.len(), 3);
    assert!(living_positions.contains(&(1.0, 1.0))); // entity1
    assert!(living_positions.contains(&(2.0, 2.0))); // entity2
    assert!(living_positions.contains(&(4.0, 4.0))); // entity4
}

#[test]
fn test_query_integration_with_entity_lifecycle() {
    let mut world = World::new();
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    world
        .add_component(entity1, Position { x: 1.0, y: 2.0 })
        .unwrap();
    world
        .add_component(entity2, Position { x: 3.0, y: 4.0 })
        .unwrap();
    world
        .add_component(entity3, Position { x: 5.0, y: 6.0 })
        .unwrap();

    let query = Query::<Position>::new();

    // Initially, all 3 entities should be found
    let initial_results: Vec<_> = query.iter(&world).collect();
    assert_eq!(initial_results.len(), 3);

    // Delete one entity
    world.delete_entity(entity2);

    // Query should now only find 2 entities (deleted entities are automatically excluded)
    let after_delete_results: Vec<_> = query.iter(&world).collect();
    assert_eq!(after_delete_results.len(), 2);
    let found_entities: Vec<_> = after_delete_results.iter().map(|(e, _)| *e).collect();
    assert!(found_entities.contains(&entity1));
    assert!(!found_entities.contains(&entity2)); // Deleted entity not found
    assert!(found_entities.contains(&entity3));

    // Remove a component from an entity
    world.remove_component::<Position>(entity3);

    // Query should now only find 1 entity
    let after_component_removal: Vec<_> = query.iter(&world).collect();
    assert_eq!(after_component_removal.len(), 1);
    assert_eq!(after_component_removal[0].0, entity1);

    // Add component back to entity3
    world
        .add_component(entity3, Position { x: 7.0, y: 8.0 })
        .unwrap();

    // Query should find 2 entities again
    let after_re_add: Vec<_> = query.iter(&world).collect();
    assert_eq!(after_re_add.len(), 2);
    let final_entities: Vec<_> = after_re_add.iter().map(|(e, _)| *e).collect();
    assert!(final_entities.contains(&entity1));
    assert!(final_entities.contains(&entity3));

    // Cleanup deleted entities
    world.cleanup_deleted_entities();

    // Query results should remain the same after cleanup
    let after_cleanup: Vec<_> = query.iter(&world).collect();
    assert_eq!(after_cleanup.len(), 2);
}

#[test]
fn test_large_scale_query_performance_integration() {
    let mut world = World::new();

    // Create many entities with different component combinations
    let mut expected_position_count = 0;
    let mut expected_moving_count = 0;
    let mut expected_living_count = 0;

    for i in 0..1000 {
        let entity = world.spawn_entity();

        // Every entity gets a unique pattern based on index
        if i % 3 == 0 {
            // Every third entity has Position
            world
                .add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: (i * 2) as f32,
                    },
                )
                .unwrap();
            expected_position_count += 1;

            if i % 6 == 0 {
                // Every sixth entity also has Velocity
                world
                    .add_component(entity, Velocity { x: 1.0, y: 0.0 })
                    .unwrap();
                expected_moving_count += 1;
            }

            if i % 9 != 0 {
                // Most entities are living (not every ninth)
                expected_living_count += 1;
            } else {
                // Every ninth entity is dead
                world.add_component(entity, Dead).unwrap();
            }
        }
    }

    // Test basic position query
    let position_query = Query::<Position>::new();
    let position_results: Vec<_> = position_query.iter(&world).collect();
    assert_eq!(position_results.len(), expected_position_count);

    // Test query with Velocity filtering
    let moving_query = Query::<Position>::new().with::<Velocity>();
    let moving_results: Vec<_> = moving_query.iter(&world).collect();
    assert_eq!(moving_results.len(), expected_moving_count);

    // Test query without Dead filtering
    let living_query = Query::<Position>::new().without::<Dead>();
    let living_results: Vec<_> = living_query.iter(&world).collect();
    assert_eq!(living_results.len(), expected_living_count);

    // Test combined filtering
    let complex_query = Query::<Position>::new()
        .with::<Velocity>()
        .without::<Dead>();
    let complex_results: Vec<_> = complex_query.iter(&world).collect();

    // Should find entities that have Position, Velocity, but not Dead
    // This would be entities where i % 6 == 0 but i % 9 != 0
    let expected_complex_count = (0..1000)
        .filter(|&i| i % 3 == 0) // Has Position
        .filter(|&i| i % 6 == 0) // Has Velocity
        .filter(|&i| i % 9 != 0) // Not Dead
        .count();

    assert_eq!(complex_results.len(), expected_complex_count);

    // Verify size hint optimization works for large collections
    let iter = position_query.iter(&world);
    let (lower_hint, upper_hint) = iter.size_hint();
    assert_eq!(upper_hint, Some(expected_position_count)); // Only entities with Position component
                                                           // Lower hint should be reasonable approximation (default 10% * 1.5 = 15%)
    assert!(lower_hint <= position_results.len() * 2); // Within 2x of actual

    // Test performance characteristics by measuring multiple iterations
    let start_time = std::time::Instant::now();
    for _ in 0..10 {
        let _: Vec<_> = complex_query.iter(&world).collect();
    }
    let duration = start_time.elapsed();

    // Should complete 10 iterations quickly (this is more of a smoke test)
    assert!(duration.as_millis() < 1000); // Less than 1 second for 10 iterations
}

#[test]
fn test_realistic_game_scenario_integration() {
    let mut world = World::new();

    // Create a realistic game scenario with different entity types

    // Player entity
    let player = world.spawn_entity();
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(player, Velocity { x: 0.0, y: 0.0 })
        .unwrap();
    world.add_component(player, Health { value: 100 }).unwrap();

    // Enemy entities
    let mut enemies = Vec::new();
    for i in 0..5 {
        let enemy = world.spawn_entity();
        world
            .add_component(
                enemy,
                Position {
                    x: (i * 10) as f32,
                    y: (i * 10) as f32,
                },
            )
            .unwrap();
        world
            .add_component(enemy, Velocity { x: -1.0, y: 0.0 })
            .unwrap();
        world.add_component(enemy, Health { value: 50 }).unwrap();
        enemies.push(enemy);
    }

    // Static environment entities (no velocity)
    for i in 0..3 {
        let static_entity = world.spawn_entity();
        world
            .add_component(
                static_entity,
                Position {
                    x: (i * 20) as f32,
                    y: 100.0,
                },
            )
            .unwrap();
    }

    // Query all entities that can move (have velocity)
    let moving_entities = Query::<Position>::new().with::<Velocity>();
    let moving_count = moving_entities.count(&world);
    assert_eq!(moving_count, 6); // Player + 5 enemies

    // Query all entities with health (living entities)
    let living_entities = Query::<Health>::new();
    let living_count = living_entities.count(&world);
    assert_eq!(living_count, 6); // Player + 5 enemies

    // Query moving entities with health (combat-capable entities)
    let combat_entities = Query::<Position>::new().with::<Velocity>().with::<Health>();
    let combat_count = combat_entities.count(&world);
    assert_eq!(combat_count, 6); // Player + 5 enemies

    // Simulate killing an enemy
    world.remove_component::<Health>(enemies[0]);
    world.add_component(enemies[0], Dead).unwrap();

    // Query living combat entities
    let living_combat = Query::<Position>::new()
        .with::<Velocity>()
        .with::<Health>()
        .without::<Dead>();
    let living_combat_count = living_combat.count(&world);
    assert_eq!(living_combat_count, 5); // Player + 4 remaining enemies

    // Query all positioned entities (should include static environment)
    let all_positioned = Query::<Position>::new();
    let positioned_count = all_positioned.count(&world);
    assert_eq!(positioned_count, 9); // Player + 5 enemies + 3 static

    // Test iterator usage in a realistic scenario
    let enemy_positions: Vec<(f32, f32)> = Query::<Position>::new()
        .with::<Velocity>()
        .with::<Health>()
        .without::<Dead>()
        .iter(&world)
        .filter(|(entity, _)| *entity != player) // Exclude player
        .map(|(_, pos)| (pos.x, pos.y))
        .collect();

    assert_eq!(enemy_positions.len(), 4); // 4 living enemies

    // Verify enemy positions - should be from the expected set (excluding (0,0) which is dead)
    let expected_positions: Vec<(f32, f32)> =
        vec![(10.0, 10.0), (20.0, 20.0), (30.0, 30.0), (40.0, 40.0)];
    for &(x, y) in &enemy_positions {
        assert!(
            expected_positions.contains(&(x, y)),
            "Found unexpected position ({x}, {y})",
        );
    }

    // Verify we don't have the dead enemy's position (0,0)
    assert!(
        !enemy_positions.contains(&(0.0, 0.0)),
        "Dead enemy position should not be included"
    );
}
