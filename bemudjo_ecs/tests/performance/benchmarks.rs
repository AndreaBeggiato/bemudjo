//! Benchmark Integration Tests
//!
//! Tests focused on measuring and validating performance characteristics
//! of ECS operations under various scenarios.

use bemudjo_ecs::{Component, Query, SequentialSystemScheduler, System, World};
use std::time::{Duration, Instant};

// Benchmark Components
#[derive(Clone, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}
impl Component for Position {}

#[derive(Clone, Debug, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}
impl Component for Velocity {}

#[derive(Clone, Debug, PartialEq)]
struct Health {
    current: u32,
    max: u32,
}
impl Component for Health {}

#[derive(Clone, Debug, PartialEq)]
struct Transform {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}
impl Component for Transform {}

#[derive(Clone, Debug, PartialEq)]
struct Renderable {
    mesh_id: u32,
    material_id: u32,
    visible: bool,
}
impl Component for Renderable {}

// Benchmark helper functions
fn benchmark_operation<F>(name: &str, operation: F, expected_max_ms: u64) -> Duration
where
    F: FnOnce(),
{
    let start = Instant::now();
    operation();
    let duration = start.elapsed();

    println!("{}: {:?}", name, duration);
    assert!(
        duration.as_millis() <= expected_max_ms as u128,
        "{} took {}ms, expected <= {}ms",
        name,
        duration.as_millis(),
        expected_max_ms
    );

    duration
}

fn benchmark_repeated_operation<F>(
    name: &str,
    operation: F,
    iterations: usize,
    expected_max_ms: u64,
) -> Duration
where
    F: Fn(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    let duration = start.elapsed();

    let avg_duration = duration / iterations as u32;
    println!(
        "{}: {} iterations in {:?} (avg: {:?})",
        name, iterations, duration, avg_duration
    );

    assert!(
        duration.as_millis() <= expected_max_ms as u128,
        "{} took {}ms for {} iterations, expected <= {}ms",
        name,
        duration.as_millis(),
        iterations,
        expected_max_ms
    );

    duration
}

#[test]
fn benchmark_entity_operations() {
    let mut world = World::new();

    // Benchmark entity spawning
    benchmark_operation(
        "Spawn 10,000 entities",
        || {
            for _ in 0..10_000 {
                world.spawn_entity();
            }
        },
        100, // 100ms max
    );

    assert_eq!(world.entities().count(), 10_000);

    // Benchmark entity iteration
    let entities: Vec<_> = world.entities().cloned().collect();

    benchmark_operation(
        "Iterate 10,000 entities",
        || {
            let mut count = 0;
            for _entity in world.entities() {
                count += 1;
            }
            assert_eq!(count, 10_000);
        },
        10, // 10ms max
    );

    // Benchmark entity deletion
    benchmark_operation(
        "Delete 5,000 entities",
        || {
            for i in (0..10_000).step_by(2) {
                world.delete_entity(entities[i]);
            }
        },
        50, // 50ms max
    );

    assert_eq!(world.entities().count(), 5_000);

    // Benchmark cleanup
    benchmark_operation(
        "Cleanup deleted entities",
        || {
            world.cleanup_deleted_entities();
        },
        20, // 20ms max
    );

    assert_eq!(world.entities().count(), 5_000);
}

#[test]
fn benchmark_component_operations() {
    let mut world = World::new();

    // Create entities
    let mut entities = Vec::new();
    for _ in 0..10_000 {
        entities.push(world.spawn_entity());
    }

    // Benchmark component addition
    benchmark_operation(
        "Add Position to 10,000 entities",
        || {
            for (i, &entity) in entities.iter().enumerate() {
                world
                    .add_component(
                        entity,
                        Position {
                            x: i as f32,
                            y: i as f32,
                            z: 0.0,
                        },
                    )
                    .unwrap();
            }
        },
        200, // 200ms max
    );

    // Benchmark component access
    benchmark_operation(
        "Read Position from 10,000 entities",
        || {
            for &entity in &entities {
                let _pos = world.get_component::<Position>(entity);
            }
        },
        50, // 50ms max
    );

    // Benchmark component updates
    benchmark_operation(
        "Update Position on 10,000 entities",
        || {
            for &entity in &entities {
                world
                    .update_component::<Position, _>(entity, |mut pos| {
                        pos.x += 1.0;
                        pos.y += 1.0;
                        pos
                    })
                    .ok();
            }
        },
        100, // 100ms max
    );

    // Benchmark component replacement
    benchmark_operation(
        "Replace Position on 10,000 entities",
        || {
            for (i, &entity) in entities.iter().enumerate() {
                world.replace_component(
                    entity,
                    Position {
                        x: (i * 2) as f32,
                        y: (i * 2) as f32,
                        z: 1.0,
                    },
                );
            }
        },
        80, // 80ms max
    );

    // Add another component type
    benchmark_operation(
        "Add Velocity to 5,000 entities",
        || {
            for (i, &entity) in entities.iter().enumerate() {
                if i % 2 == 0 {
                    world
                        .add_component(
                            entity,
                            Velocity {
                                x: 1.0,
                                y: 0.5,
                                z: 0.0,
                            },
                        )
                        .unwrap();
                }
            }
        },
        100, // 100ms max
    );

    // Benchmark component removal
    benchmark_operation(
        "Remove Position from 2,500 entities",
        || {
            for (i, &entity) in entities.iter().enumerate() {
                if i % 4 == 0 {
                    world.remove_component::<Position>(entity);
                }
            }
        },
        50, // 50ms max
    );
}

#[test]
#[ignore]
fn benchmark_query_operations() {
    let mut world = World::new();

    // Create entities with various component combinations
    for i in 0..20_000 {
        let entity = world.spawn_entity();

        // All entities have Position
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: (i % 1000) as f32,
                    z: 0.0,
                },
            )
            .unwrap();

        // 50% have Velocity
        if i % 2 == 0 {
            world
                .add_component(
                    entity,
                    Velocity {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0,
                    },
                )
                .unwrap();
        }

        // 33% have Health
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

        // 25% have Transform
        if i % 4 == 0 {
            world
                .add_component(
                    entity,
                    Transform {
                        translation: [i as f32, 0.0, 0.0],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0, 1.0, 1.0],
                    },
                )
                .unwrap();
        }

        // 20% have Renderable
        if i % 5 == 0 {
            world
                .add_component(
                    entity,
                    Renderable {
                        mesh_id: (i % 10) as u32,
                        material_id: (i % 5) as u32,
                        visible: true,
                    },
                )
                .unwrap();
        }
    }

    // Benchmark simple query
    benchmark_operation(
        "Query all Position components (20,000)",
        || {
            let query = Query::<Position>::new();
            let count = query.iter(&world).count();
            assert_eq!(count, 20_000);
        },
        20, // 20ms max
    );

    // Benchmark filtered query
    benchmark_operation(
        "Query Position + Velocity (10,000)",
        || {
            let query = Query::<Position>::new().with::<Velocity>();
            let count = query.iter(&world).count();
            assert_eq!(count, 10_000);
        },
        15, // 15ms max
    );

    // Benchmark complex query
    benchmark_operation(
        "Query Position + Velocity + Health (3,334)",
        || {
            let query = Query::<Position>::new().with::<Velocity>().with::<Health>();
            let count = query.iter(&world).count();
            assert!(count > 3_000 && count < 4_000);
        },
        10, // 10ms max
    );

    // Benchmark query with exclusion
    benchmark_operation(
        "Query Position without Velocity (10,000)",
        || {
            let query = Query::<Position>::new().without::<Velocity>();
            let count = query.iter(&world).count();
            assert_eq!(count, 10_000);
        },
        15, // 15ms max
    );

    // Benchmark query iteration with processing
    benchmark_operation(
        "Query and process Position + Velocity",
        || {
            let query = Query::<Position>::new().with::<Velocity>();
            let total_distance: f32 = query
                .iter(&world)
                .map(|(_, pos)| (pos.x * pos.x + pos.y * pos.y).sqrt())
                .sum();
            assert!(total_distance > 0.0);
        },
        30, // 30ms max
    );

    // Benchmark repeated queries
    benchmark_repeated_operation(
        "Repeated simple query",
        || {
            let query = Query::<Position>::new();
            let _count = query.iter(&world).count();
        },
        1000, // 1000 iterations
        200,  // 200ms max total
    );
}

#[test]
#[ignore]
fn benchmark_system_execution() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Create test entities
    for i in 0..10_000 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32,
                    z: 0.0,
                },
            )
            .unwrap();

        if i % 2 == 0 {
            world
                .add_component(
                    entity,
                    Velocity {
                        x: 1.0,
                        y: 0.5,
                        z: 0.0,
                    },
                )
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

    // Simple movement system
    struct MovementSystem;
    impl System for MovementSystem {
        fn run(&self, world: &mut World) {
            let entities: Vec<_> = world.entities().cloned().collect();
            for entity in entities {
                if let (Some(pos), Some(vel)) = (
                    world.get_component::<Position>(entity),
                    world.get_component::<Velocity>(entity),
                ) {
                    let new_pos = Position {
                        x: pos.x + vel.x,
                        y: pos.y + vel.y,
                        z: pos.z + vel.z,
                    };
                    world.replace_component(entity, new_pos);
                }
            }
        }
    }

    // Health regeneration system
    struct HealthSystem;
    impl System for HealthSystem {
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
                        .ok();
                }
            }
        }
    }

    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(HealthSystem).unwrap();
    scheduler.build().unwrap();

    // Benchmark single system tick
    benchmark_operation(
        "Single system tick (2 systems, 10,000 entities)",
        || {
            scheduler.run_tick(&mut world);
        },
        50, // 50ms max
    );

    // Benchmark multiple system ticks
    benchmark_operation(
        "100 system ticks",
        || {
            for _ in 0..100 {
                scheduler.run_tick(&mut world);
            }
        },
        2000, // 2 seconds max
    );

    // Verify systems actually processed entities
    let moved_entities = Query::<Position>::new()
        .with::<Velocity>()
        .iter(&world)
        .filter(|(_, pos)| pos.x > 100.0) // Should have moved significantly
        .count();
    assert!(moved_entities > 4_000); // Most moving entities should have moved
}

#[test]
fn benchmark_scheduler_operations() {
    // Benchmark scheduler creation and system addition
    benchmark_operation(
        "Create scheduler and add 100 systems",
        || {
            let mut scheduler = SequentialSystemScheduler::new();

            for i in 0..100 {
                struct BenchmarkSystem {
                    id: usize,
                }

                impl System for BenchmarkSystem {
                    fn run(&self, _world: &mut World) {
                        // Minimal work
                        let _result = self.id * 2;
                    }
                }

                scheduler.add_system(BenchmarkSystem { id: i }).unwrap();
            }

            scheduler.build().unwrap();
            assert_eq!(scheduler.system_count(), 100);
        },
        50, // 50ms max
    );

    // Benchmark empty system execution
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    for _ in 0..50 {
        struct EmptySystem;

        impl System for EmptySystem {
            fn run(&self, _world: &mut World) {
                // Do nothing
            }
        }

        scheduler.add_system(EmptySystem).unwrap();
    }

    scheduler.build().unwrap();

    benchmark_operation(
        "50 empty systems execution",
        || {
            scheduler.run_tick(&mut world);
        },
        5, // 5ms max
    );

    // Benchmark scheduler with entities but no component access
    for _ in 0..1000 {
        world.spawn_entity();
    }

    benchmark_operation(
        "50 empty systems with 1,000 entities",
        || {
            scheduler.run_tick(&mut world);
        },
        10, // 10ms max
    );
}

#[test]
fn benchmark_scaling_characteristics() {
    // Test how operations scale with entity count
    let entity_counts = vec![100, 1_000, 10_000, 50_000];
    let mut results = Vec::new();

    for &count in &entity_counts {
        let mut world = World::new();

        // Create entities
        let creation_start = Instant::now();
        for i in 0..count {
            let entity = world.spawn_entity();
            world
                .add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                )
                .unwrap();

            if i % 2 == 0 {
                world
                    .add_component(
                        entity,
                        Velocity {
                            x: 1.0,
                            y: 1.0,
                            z: 0.0,
                        },
                    )
                    .unwrap();
            }
        }
        let creation_duration = creation_start.elapsed();

        // Query performance
        let query_start = Instant::now();
        let query = Query::<Position>::new().with::<Velocity>();
        let query_count = query.iter(&world).count();
        let query_duration = query_start.elapsed();

        assert_eq!(query_count, count / 2);

        results.push((count, creation_duration, query_duration));

        println!(
            "Entities: {}, Creation: {:?}, Query: {:?}",
            count, creation_duration, query_duration
        );
    }

    // Verify scaling characteristics
    for i in 1..results.len() {
        let (prev_count, prev_creation, prev_query) = results[i - 1];
        let (curr_count, curr_creation, curr_query) = results[i];

        let count_ratio = curr_count as f64 / prev_count as f64;
        let creation_ratio = curr_creation.as_nanos() as f64 / prev_creation.as_nanos() as f64;
        let query_ratio = curr_query.as_nanos() as f64 / prev_query.as_nanos() as f64;

        // Creation should scale roughly linearly (within factor of 3 due to overhead)
        assert!(
            creation_ratio <= count_ratio * 3.0,
            "Creation scaling too poor: {}x entities took {}x time",
            count_ratio,
            creation_ratio
        );

        // Query should scale roughly linearly (within factor of 2)
        assert!(
            query_ratio <= count_ratio * 2.0,
            "Query scaling too poor: {}x entities took {}x time",
            count_ratio,
            query_ratio
        );
    }
}

#[test]
fn benchmark_memory_efficiency() {
    // This is a basic memory efficiency test
    // In practice, you'd use proper memory profiling tools

    let mut world = World::new();

    // Measure baseline
    let initial_entities = world.entities().count();

    // Add many entities
    for i in 0..10_000 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32,
                    z: i as f32,
                },
            )
            .unwrap();
    }

    assert_eq!(world.entities().count(), initial_entities + 10_000);

    // Delete half
    let entities: Vec<_> = world.entities().cloned().collect();
    for (i, &entity) in entities.iter().enumerate() {
        if i % 2 == 0 {
            world.delete_entity(entity);
        }
    }

    assert_eq!(world.entities().count(), 5_000);

    // Cleanup
    benchmark_operation(
        "Cleanup 5,000 deleted entities",
        || {
            world.cleanup_deleted_entities();
        },
        50, // 50ms max
    );

    assert_eq!(world.entities().count(), 5_000);

    // Add entities again to test memory reuse
    for i in 0..5_000 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32,
                    z: i as f32,
                },
            )
            .unwrap();
    }

    assert_eq!(world.entities().count(), 10_000);
}

#[test]
#[ignore]
fn benchmark_regression_prevention() {
    // This test establishes performance baselines to prevent regressions

    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Standard benchmark setup
    const ENTITY_COUNT: usize = 10_000;

    for i in 0..ENTITY_COUNT {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
            )
            .unwrap();

        if i % 2 == 0 {
            world
                .add_component(
                    entity,
                    Velocity {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0,
                    },
                )
                .unwrap();
        }
    }

    struct StandardSystem;
    impl System for StandardSystem {
        fn run(&self, world: &mut World) {
            let entities: Vec<_> = world.entities().cloned().collect();
            for entity in entities {
                if let (Some(pos), Some(vel)) = (
                    world.get_component::<Position>(entity),
                    world.get_component::<Velocity>(entity),
                ) {
                    world.replace_component(
                        entity,
                        Position {
                            x: pos.x + vel.x,
                            y: pos.y + vel.y,
                            z: pos.z + vel.z,
                        },
                    );
                }
            }
        }
    }

    scheduler.add_system(StandardSystem).unwrap();
    scheduler.build().unwrap();

    // Establish baseline performance requirements
    let tick_duration = benchmark_operation(
        "Standard benchmark tick",
        || {
            scheduler.run_tick(&mut world);
        },
        30, // 30ms max - this is our regression prevention threshold
    );

    let query_duration = benchmark_operation(
        "Standard benchmark query",
        || {
            let query = Query::<Position>::new().with::<Velocity>();
            let count = query.iter(&world).count();
            assert_eq!(count, ENTITY_COUNT / 2);
        },
        10, // 10ms max
    );

    // Log results for tracking
    println!("Regression prevention baselines:");
    println!("  Tick duration: {:?}", tick_duration);
    println!("  Query duration: {:?}", query_duration);

    // These assertions ensure we don't regress beyond acceptable performance
    assert!(tick_duration.as_millis() <= 30);
    assert!(query_duration.as_millis() <= 10);
}
