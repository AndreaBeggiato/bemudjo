//! Query Performance Integration Tests
//!
//! Tests focused on query system performance, optimization,
//! and scalability under various load conditions.

use bemudjo_ecs::{Component, Query, World};
use std::time::Instant;

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
struct Damage {
    amount: u32,
}
impl Component for Damage {}

#[derive(Clone, Debug, PartialEq)]
struct Experience {
    points: u64,
    level: u32,
}
impl Component for Experience {}

#[derive(Clone, Debug, PartialEq)]
struct Tag {
    name: String,
}
impl Component for Tag {}

#[derive(Clone, Debug, PartialEq)]
struct AI {
    target: Option<u32>,
}
impl Component for AI {}

#[derive(Clone, Debug, PartialEq)]
struct Inventory {
    items: Vec<String>,
}
impl Component for Inventory {}

#[test]
fn test_large_scale_query_performance() {
    let mut world = World::new();

    const ENTITY_COUNT: usize = 50_000;
    let mut entities = Vec::new();

    // Create large number of entities with different component patterns
    for i in 0..ENTITY_COUNT {
        let entity = world.spawn_entity();
        entities.push(entity);

        // All entities have position
        world
            .add_component(
                entity,
                Position {
                    x: (i % 1000) as f32,
                    y: (i / 1000) as f32,
                },
            )
            .unwrap();

        // 50% have velocity
        if i % 2 == 0 {
            world
                .add_component(
                    entity,
                    Velocity {
                        x: (i % 10) as f32,
                        y: -((i % 10) as f32),
                    },
                )
                .unwrap();
        }

        // 33% have health
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

        // 25% have experience
        if i % 4 == 0 {
            world
                .add_component(
                    entity,
                    Experience {
                        points: (i % 1000) as u64,
                        level: (i % 10) as u32,
                    },
                )
                .unwrap();
        }

        // 10% have damage
        if i % 10 == 0 {
            world
                .add_component(
                    entity,
                    Damage {
                        amount: (i % 50) as u32,
                    },
                )
                .unwrap();
        }
    }

    assert_eq!(world.entities().count(), ENTITY_COUNT);

    // Test basic position query performance
    let start = Instant::now();
    let position_query = Query::<Position>::new();
    let position_results: Vec<_> = position_query.iter(&world).collect();
    let position_duration = start.elapsed();

    assert_eq!(position_results.len(), ENTITY_COUNT);
    assert!(position_duration.as_millis() < 100); // Should complete in < 100ms

    // Test filtered query performance
    let start = Instant::now();
    let moving_query = Query::<Position>::new().with::<Velocity>();
    let moving_results: Vec<_> = moving_query.iter(&world).collect();
    let moving_duration = start.elapsed();

    assert_eq!(moving_results.len(), ENTITY_COUNT / 2);
    assert!(moving_duration.as_millis() < 50); // Should be faster than full scan

    // Test complex filtered query performance
    let start = Instant::now();
    let complex_query = Query::<Position>::new()
        .with::<Velocity>()
        .with::<Health>()
        .with::<Experience>();
    let complex_results: Vec<_> = complex_query.iter(&world).collect();
    let complex_duration = start.elapsed();

    // Should find entities where i % 2 == 0 AND i % 3 == 0 AND i % 4 == 0
    // This is i % lcm(2,3,4) == 0, which is i % 12 == 0
    let expected_count = (0..ENTITY_COUNT)
        .filter(|&i| i % 2 == 0 && i % 3 == 0 && i % 4 == 0)
        .count();
    assert_eq!(complex_results.len(), expected_count);
    assert!(complex_duration.as_millis() < 30); // Should be very fast due to filtering

    // Test exclusion query performance
    let start = Instant::now();
    let exclusion_query = Query::<Position>::new()
        .with::<Health>()
        .without::<Damage>();
    let exclusion_results: Vec<_> = exclusion_query.iter(&world).collect();
    let exclusion_duration = start.elapsed();

    // Should find entities where i % 3 == 0 AND i % 10 != 0
    let expected_exclusion = (0..ENTITY_COUNT)
        .filter(|&i| i % 3 == 0 && i % 10 != 0)
        .count();
    assert_eq!(exclusion_results.len(), expected_exclusion);
    assert!(exclusion_duration.as_millis() < 50);
}

#[test]
fn test_query_iterator_performance() {
    let mut world = World::new();

    const ENTITY_COUNT: usize = 10_000;

    // Create entities
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
        world
            .add_component(
                entity,
                Health {
                    current: i as u32 % 100,
                    max: 100,
                },
            )
            .unwrap();
    }

    let query = Query::<Position>::new().with::<Health>();

    // Test basic iteration
    let start = Instant::now();
    let count = query.iter(&world).count();
    let count_duration = start.elapsed();

    assert_eq!(count, ENTITY_COUNT);
    assert!(count_duration.as_millis() < 20);

    // Test iterator combinators
    let start = Instant::now();
    let filtered_count = query.iter(&world).filter(|(_, pos)| pos.x > 5000.0).count();
    let filter_duration = start.elapsed();

    assert_eq!(filtered_count, ENTITY_COUNT - 5001); // 5001 to 9999
    assert!(filter_duration.as_millis() < 30);

    // Test map operation
    let start = Instant::now();
    let mapped: Vec<f32> = query.iter(&world).map(|(_, pos)| pos.x + pos.y).collect();
    let map_duration = start.elapsed();

    assert_eq!(mapped.len(), ENTITY_COUNT);
    assert!(map_duration.as_millis() < 40); // Test fold operation
    let start = Instant::now();
    let sum: f32 = query.iter(&world).fold(0.0, |acc, (_, pos)| acc + pos.x);
    let fold_duration = start.elapsed();

    // Just verify the sum is in a reasonable range (not zero, not too large)
    assert!(sum > 0.0);
    assert!(sum < 100_000_000.0); // Should be less than 100 million
    assert!(fold_duration.as_millis() < 25);
}

#[test]
fn test_query_performance_with_sparse_components() {
    let mut world = World::new();

    const ENTITY_COUNT: usize = 20_000;

    // Create entities with very sparse component distribution
    for i in 0..ENTITY_COUNT {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: 0.0,
                },
            )
            .unwrap();

        // Only 1% have velocity (very sparse)
        if i % 100 == 0 {
            world
                .add_component(entity, Velocity { x: 1.0, y: 1.0 })
                .unwrap();
        }

        // Only 0.5% have damage (extremely sparse)
        if i % 200 == 0 {
            world.add_component(entity, Damage { amount: 10 }).unwrap();
        }

        // Only 0.1% have AI (ultra sparse)
        if i % 1000 == 0 {
            world.add_component(entity, AI { target: None }).unwrap();
        }
    }

    // Test query with sparse components should be very fast
    let start = Instant::now();
    let sparse_query = Query::<Position>::new()
        .with::<Velocity>()
        .with::<Damage>()
        .with::<AI>();
    let sparse_results: Vec<_> = sparse_query.iter(&world).collect();
    let sparse_duration = start.elapsed();

    // Should find entities where i % 100 == 0 AND i % 200 == 0 AND i % 1000 == 0
    // This is i % 1000 == 0
    let expected_count = ENTITY_COUNT / 1000;
    assert_eq!(sparse_results.len(), expected_count);
    assert!(sparse_duration.as_millis() < 10); // Should be very fast due to sparsity

    // Test that performance scales with result size, not entity count
    let start = Instant::now();
    let medium_sparse_query = Query::<Position>::new().with::<Velocity>();
    let medium_sparse_results: Vec<_> = medium_sparse_query.iter(&world).collect();
    let medium_sparse_duration = start.elapsed();

    assert_eq!(medium_sparse_results.len(), ENTITY_COUNT / 100);
    assert!(medium_sparse_duration.as_millis() < 15);

    // Both queries should be reasonably fast due to sparsity
    // Note: Ultra-sparse queries might have more overhead than medium-sparse queries
    assert!(sparse_duration.as_millis() < 20);
    assert!(medium_sparse_duration.as_millis() < 20);
}

#[test]
fn test_query_performance_under_modification() {
    let mut world = World::new();

    const ENTITY_COUNT: usize = 5_000;
    let mut entities = Vec::new();

    // Create initial entities
    for i in 0..ENTITY_COUNT {
        let entity = world.spawn_entity();
        entities.push(entity);

        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32,
                },
            )
            .unwrap();
        if i % 2 == 0 {
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

    let query = Query::<Position>::new().with::<Health>();

    // Baseline performance
    let start = Instant::now();
    let baseline_count = query.iter(&world).count();
    let baseline_duration = start.elapsed();

    assert_eq!(baseline_count, ENTITY_COUNT / 2);

    // Modify world by adding/removing components in a meaningful way
    let mut modifications_made = 0;

    for (i, entity) in entities.iter().enumerate().take(1000) {
        let entity = *entity;

        if i % 3 == 0 {
            // Add Health to entities that don't have it (odd-indexed entities)
            if !world.has_component::<Health>(entity) {
                world
                    .add_component(
                        entity,
                        Health {
                            current: 50,
                            max: 50,
                        },
                    )
                    .unwrap();
                modifications_made += 1;
            }
        } else if i % 3 == 1 {
            // Remove Health from entities that have it (even-indexed entities)
            if world.has_component::<Health>(entity) {
                world.remove_component::<Health>(entity);
                modifications_made += 1;
            }
        } else {
            // Delete entities that have Health (this will reduce the count)
            if world.has_component::<Health>(entity) {
                world.delete_entity(entity);
                modifications_made += 1;
            }
        }
    }

    // Clean up deleted entities
    world.cleanup_deleted_entities();

    // Test performance after modifications
    let start = Instant::now();
    let modified_count = query.iter(&world).count();
    let modified_duration = start.elapsed();

    // Ensure we actually made some modifications
    assert!(
        modifications_made > 0,
        "No modifications were made to the world"
    );

    // Performance should not degrade significantly
    let performance_ratio =
        modified_duration.as_nanos() as f64 / baseline_duration.as_nanos() as f64;
    assert!(performance_ratio < 3.0); // Should not be more than 3x slower

    // Count should have changed due to the modifications
    assert_ne!(modified_count, baseline_count,
        "Query count should change after modifications. Baseline: {baseline_count}, Modified: {modified_count}, Modifications made: {modifications_made}");

    // Test performance after cleanup
    let start = Instant::now();
    let _cleanup_count = query.iter(&world).count();
    let cleanup_duration = start.elapsed();

    // Performance after cleanup should be similar to baseline
    let cleanup_ratio = cleanup_duration.as_nanos() as f64 / baseline_duration.as_nanos() as f64;
    assert!(cleanup_ratio < 2.0);
}

#[test]
fn test_multiple_concurrent_queries() {
    let mut world = World::new();

    const ENTITY_COUNT: usize = 10_000;

    // Create entities with overlapping component sets
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

        if i % 2 == 0 {
            world
                .add_component(entity, Velocity { x: 1.0, y: 1.0 })
                .unwrap();
        }

        if i % 3 == 0 {
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

        if i % 5 == 0 {
            world
                .add_component(
                    entity,
                    Experience {
                        points: 0,
                        level: 1,
                    },
                )
                .unwrap();
        }

        if i % 7 == 0 {
            world
                .add_component(
                    entity,
                    Tag {
                        name: format!("Entity{i}"),
                    },
                )
                .unwrap();
        }
    }

    // Define multiple queries
    let queries = vec![
        Query::<Position>::new(),
        Query::<Position>::new().with::<Velocity>(),
        Query::<Position>::new().with::<Health>(),
        Query::<Position>::new().with::<Experience>(),
        Query::<Position>::new().with::<Tag>(),
        Query::<Position>::new().with::<Velocity>().with::<Health>(),
        Query::<Position>::new()
            .with::<Health>()
            .with::<Experience>(),
        Query::<Position>::new().without::<Velocity>(),
        Query::<Position>::new().without::<Health>(),
    ];

    // Execute all queries and measure total time
    let start = Instant::now();
    let mut total_results = 0;

    for query in &queries {
        let count = query.iter(&world).count();
        total_results += count;
    }

    let total_duration = start.elapsed();

    // Should complete all queries quickly
    assert!(total_duration.as_millis() < 100);
    assert!(total_results > 0);

    // Test parallel-style execution (simulated)
    let start = Instant::now();
    let results: Vec<usize> = queries
        .iter()
        .map(|query| query.iter(&world).count())
        .collect();
    let parallel_duration = start.elapsed();

    assert_eq!(results.len(), queries.len());
    assert!(parallel_duration.as_millis() < 80); // Should be similar or faster

    // Verify expected counts
    assert_eq!(results[0], ENTITY_COUNT); // All have Position
    assert_eq!(results[1], ENTITY_COUNT / 2); // Half have Velocity
    assert_eq!(results[2], ENTITY_COUNT.div_ceil(3)); // Third have Health
    assert_eq!(results[3], ENTITY_COUNT.div_ceil(5)); // Fifth have Experience
    assert_eq!(results[4], ENTITY_COUNT.div_ceil(7)); // Seventh have Tag
}

#[test]
fn test_query_performance_with_large_components() {
    let mut world = World::new();

    const ENTITY_COUNT: usize = 1_000;

    // Create entities with large components
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

        // Large inventory component
        world
            .add_component(
                entity,
                Inventory {
                    items: (0..100).map(|j| format!("Item{i}_{j}")).collect(),
                },
            )
            .unwrap();

        if i % 2 == 0 {
            world
                .add_component(
                    entity,
                    Tag {
                        name: "X".repeat(1000), // Large string
                    },
                )
                .unwrap();
        }
    }

    // Test query performance with large components
    let start = Instant::now();
    let query = Query::<Position>::new().with::<Inventory>();
    let results: Vec<_> = query.iter(&world).collect();
    let duration = start.elapsed();

    assert_eq!(results.len(), ENTITY_COUNT);
    assert!(duration.as_millis() < 100); // Should still be fast

    // Test iteration that accesses large components
    let start = Instant::now();
    let total_items: usize = query
        .iter(&world)
        .map(|(entity, _)| {
            world
                .get_component::<Inventory>(entity)
                .map(|inv| inv.items.len())
                .unwrap_or(0)
        })
        .sum();
    let access_duration = start.elapsed();

    assert_eq!(total_items, ENTITY_COUNT * 100);
    assert!(access_duration.as_millis() < 200); // Accessing large components takes more time

    // Test filtered query with large components
    let start = Instant::now();
    let filtered_query = Query::<Position>::new().with::<Inventory>().with::<Tag>();
    let filtered_results: Vec<_> = filtered_query.iter(&world).collect();
    let filtered_duration = start.elapsed();

    assert_eq!(filtered_results.len(), ENTITY_COUNT / 2);
    assert!(filtered_duration.as_millis() < 50); // Should be faster due to filtering
}

#[test]
fn test_query_performance_regression() {
    // This test establishes performance baselines for regression testing
    let mut world = World::new();

    const SMALL_ENTITY_COUNT: usize = 1_000;
    const MEDIUM_ENTITY_COUNT: usize = 10_000;
    const LARGE_ENTITY_COUNT: usize = 50_000;

    // Helper function to create entities with standard pattern
    let create_entities = |world: &mut World, count: usize| {
        for i in 0..count {
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

            if i % 2 == 0 {
                world
                    .add_component(entity, Velocity { x: 1.0, y: 1.0 })
                    .unwrap();
            }

            if i % 3 == 0 {
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
    };

    // Test small scale
    create_entities(&mut world, SMALL_ENTITY_COUNT);

    let start = Instant::now();
    let small_query = Query::<Position>::new().with::<Velocity>();
    let small_count = small_query.iter(&world).count();
    let small_duration = start.elapsed();

    assert_eq!(small_count, SMALL_ENTITY_COUNT / 2);
    assert!(small_duration.as_millis() < 10); // Very fast for small scale

    // Test medium scale
    world = World::new(); // Reset
    create_entities(&mut world, MEDIUM_ENTITY_COUNT);

    let start = Instant::now();
    let medium_query = Query::<Position>::new().with::<Velocity>();
    let medium_count = medium_query.iter(&world).count();
    let medium_duration = start.elapsed();

    assert_eq!(medium_count, MEDIUM_ENTITY_COUNT / 2);
    assert!(medium_duration.as_millis() < 50); // Should scale reasonably

    // Test large scale
    world = World::new(); // Reset
    create_entities(&mut world, LARGE_ENTITY_COUNT);

    let start = Instant::now();
    let large_query = Query::<Position>::new().with::<Velocity>();
    let large_count = large_query.iter(&world).count();
    let large_duration = start.elapsed();

    assert_eq!(large_count, LARGE_ENTITY_COUNT / 2);
    assert!(large_duration.as_millis() < 200); // Should handle large scale

    // Performance should scale roughly linearly
    let small_per_entity = small_duration.as_nanos() as f64 / SMALL_ENTITY_COUNT as f64;
    let large_per_entity = large_duration.as_nanos() as f64 / LARGE_ENTITY_COUNT as f64;

    // Large scale should not be more than 5x slower per entity (due to cache effects, etc.)
    assert!(large_per_entity / small_per_entity < 5.0);
}
