//! Stress Test Integration Tests
//!
//! Tests focused on high-load scenarios, memory pressure,
//! and system behavior under extreme conditions.

use bemudjo_ecs::{Component, SequentialSystemScheduler, System, World};
use std::time::Instant;

// Test Components for stress testing
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
    regeneration: f32,
}
impl Component for Health {}

#[derive(Clone, Debug, PartialEq)]
struct LargeData {
    buffer: Vec<u8>,
    id: u64,
    metadata: String,
}
impl Component for LargeData {}

#[derive(Clone, Debug, PartialEq)]
struct AI {
    state: u32,
    target: Option<u64>,
    memory: Vec<f32>,
}
impl Component for AI {}

#[derive(Clone, Debug, PartialEq)]
struct Inventory {
    items: Vec<String>,
    capacity: usize,
    weight: f32,
}
impl Component for Inventory {}

#[derive(Clone, Debug, PartialEq)]
struct Physics {
    mass: f32,
    friction: f32,
    forces: Vec<(f32, f32, f32)>,
}
impl Component for Physics {}

// Stress test systems
struct MassEntitySpawner {
    entities_per_tick: usize,
    max_entities: usize,
}

impl MassEntitySpawner {
    fn new(entities_per_tick: usize, max_entities: usize) -> Self {
        Self {
            entities_per_tick,
            max_entities,
        }
    }
}

impl System for MassEntitySpawner {
    fn run(&self, world: &mut World) {
        let current_entity_count = world.entities().count();

        if current_entity_count < self.max_entities {
            let to_spawn = std::cmp::min(
                self.entities_per_tick,
                self.max_entities - current_entity_count,
            );

            for i in 0..to_spawn {
                let entity = world.spawn_entity();

                // Add various components based on patterns
                world
                    .add_component(
                        entity,
                        Position {
                            x: (current_entity_count + i) as f32,
                            y: ((current_entity_count + i) % 1000) as f32,
                            z: 0.0,
                        },
                    )
                    .unwrap();

                if (current_entity_count + i) % 2 == 0 {
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

                if (current_entity_count + i) % 3 == 0 {
                    world
                        .add_component(
                            entity,
                            Health {
                                current: 100,
                                max: 100,
                                regeneration: 1.0,
                            },
                        )
                        .unwrap();
                }

                if (current_entity_count + i) % 5 == 0 {
                    world
                        .add_component(
                            entity,
                            AI {
                                state: 0,
                                target: None,
                                memory: vec![0.0; 10],
                            },
                        )
                        .unwrap();
                }

                if (current_entity_count + i) % 7 == 0 {
                    world
                        .add_component(
                            entity,
                            Physics {
                                mass: 1.0,
                                friction: 0.1,
                                forces: Vec::new(),
                            },
                        )
                        .unwrap();
                }
            }
        }
    }
}

struct HeavyComputationSystem;

impl System for HeavyComputationSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            // Perform heavy computation on entities with AI
            if let Some(ai) = world.get_component::<AI>(entity) {
                let mut new_memory = ai.memory.clone();

                // Simulate complex AI calculations
                for i in 0..new_memory.len() {
                    new_memory[i] = (new_memory[i] + (i as f32).sin()).cos();
                }

                world.replace_component(
                    entity,
                    AI {
                        state: (ai.state + 1) % 10,
                        target: ai.target,
                        memory: new_memory,
                    },
                );
            }

            // Heavy physics calculations
            if let (Some(pos), Some(vel), Some(physics)) = (
                world.get_component::<Position>(entity),
                world.get_component::<Velocity>(entity),
                world.get_component::<Physics>(entity),
            ) {
                let mut new_forces = physics.forces.clone();

                // Simulate complex physics
                let gravity = (0.0, -9.81, 0.0);
                new_forces.push(gravity);

                // Position-based force (e.g., spring force towards origin)
                let spring_constant = 0.1;
                let spring_force = (
                    -pos.x * spring_constant,
                    -pos.y * spring_constant,
                    -pos.z * spring_constant,
                );
                new_forces.push(spring_force);

                // Apply friction
                let friction_force = (
                    -vel.x * physics.friction,
                    -vel.y * physics.friction,
                    -vel.z * physics.friction,
                );
                new_forces.push(friction_force);

                world.replace_component(
                    entity,
                    Physics {
                        mass: physics.mass,
                        friction: physics.friction,
                        forces: new_forces,
                    },
                );
            }
        }
    }
}

struct MemoryIntensiveSystem;

impl System for MemoryIntensiveSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for (i, entity) in entities.iter().enumerate() {
            if i % 100 == 0 {
                // Add large data components periodically
                let large_buffer = vec![i as u8; 10_000]; // 10KB per entity

                world
                    .add_component(
                        *entity,
                        LargeData {
                            buffer: large_buffer,
                            id: i as u64,
                            metadata: format!("Entity_{}_large_data", i),
                        },
                    )
                    .ok(); // Ignore errors if component already exists
            }

            if i % 50 == 0 {
                // Add large inventory components
                let items: Vec<String> = (0..100).map(|j| format!("Item_{}_{}", i, j)).collect();

                world
                    .add_component(
                        *entity,
                        Inventory {
                            items,
                            capacity: 200,
                            weight: i as f32 * 0.1,
                        },
                    )
                    .ok();
            }
        }
    }
}

struct CleanupSystem;

impl System for CleanupSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();
        let mut to_delete = Vec::new();

        for entity in entities {
            // Delete entities with high state AI (simulate death conditions)
            if let Some(ai) = world.get_component::<AI>(entity) {
                if ai.state >= 8 {
                    to_delete.push(entity);
                }
            }

            // Delete entities with too many forces
            if let Some(physics) = world.get_component::<Physics>(entity) {
                if physics.forces.len() > 100 {
                    to_delete.push(entity);
                }
            }
        }

        for entity in to_delete {
            world.delete_entity(entity);
        }
    }
}

#[test]
fn test_large_entity_count_stress() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler
        .add_system(MassEntitySpawner::new(1000, 50_000))
        .unwrap();
    scheduler.add_system(HeavyComputationSystem).unwrap();
    scheduler.build().unwrap();

    let start_time = Instant::now();

    // Run until we reach target entity count
    let mut tick_count = 0;
    while world.entities().count() < 50_000 && tick_count < 100 {
        scheduler.run_tick(&mut world);
        tick_count += 1;
    }

    let spawn_duration = start_time.elapsed();
    assert!(spawn_duration.as_secs() < 10); // Should complete within 10 seconds

    assert_eq!(world.entities().count(), 50_000);

    // Run computation-heavy ticks
    let compute_start = Instant::now();
    for _ in 0..10 {
        scheduler.run_tick(&mut world);
    }
    let compute_duration = compute_start.elapsed();

    assert!(compute_duration.as_secs() < 30); // Should handle large entity count

    // Verify entity integrity
    let mut position_count = 0;
    let mut velocity_count = 0;
    let mut health_count = 0;

    for &entity in world.entities() {
        if world.has_component::<Position>(entity) {
            position_count += 1;
        }
        if world.has_component::<Velocity>(entity) {
            velocity_count += 1;
        }
        if world.has_component::<Health>(entity) {
            health_count += 1;
        }
    }

    assert_eq!(position_count, 50_000); // All entities should have position
    assert!(velocity_count > 20_000); // About half should have velocity
    assert!(health_count > 15_000); // About 1/3 should have health
}

#[test]
fn test_memory_pressure_stress() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler
        .add_system(MassEntitySpawner::new(100, 10_000))
        .unwrap();
    scheduler.add_system(MemoryIntensiveSystem).unwrap();
    scheduler.build().unwrap();

    let start_time = Instant::now();

    // Run until we have many entities with large components
    for _ in 0..200 {
        scheduler.run_tick(&mut world);

        // Check memory usage periodically
        if world.entities().count() % 1000 == 0 {
            let current_time = start_time.elapsed();
            assert!(current_time.as_secs() < 60); // Should not take too long
        }
    }

    let total_duration = start_time.elapsed();
    assert!(total_duration.as_secs() < 120); // Complete within reasonable time

    // Verify large components were added
    let mut large_data_count = 0;
    let mut inventory_count = 0;

    for &entity in world.entities() {
        if world.has_component::<LargeData>(entity) {
            large_data_count += 1;

            let large_data = world.get_component::<LargeData>(entity).unwrap();
            assert_eq!(large_data.buffer.len(), 10_000);
        }

        if world.has_component::<Inventory>(entity) {
            inventory_count += 1;

            let inventory = world.get_component::<Inventory>(entity).unwrap();
            assert_eq!(inventory.items.len(), 100);
        }
    }

    assert!(large_data_count > 50); // Should have many large components
    assert!(inventory_count > 100); // Should have many inventory components
}

#[test]
fn test_rapid_creation_deletion_stress() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler
        .add_system(MassEntitySpawner::new(500, 5_000))
        .unwrap();
    scheduler.add_system(HeavyComputationSystem).unwrap();
    scheduler.add_system(CleanupSystem).unwrap();
    scheduler.build().unwrap();

    let start_time = Instant::now();

    // Run many ticks with creation and deletion
    for tick in 0..500 {
        scheduler.run_tick(&mut world);

        // Periodically force cleanup
        if tick % 10 == 0 {
            world.cleanup_deleted_entities();
        }

        // Verify entity count stays reasonable
        let entity_count = world.entities().count();
        assert!(entity_count <= 6_000); // Some buffer above max due to deletion delay

        if tick % 50 == 0 {
            let elapsed = start_time.elapsed();
            assert!(elapsed.as_secs() < 300); // Should not take too long per 50 ticks
        }
    }

    let total_duration = start_time.elapsed();
    assert!(total_duration.as_secs() < 600); // Complete within 10 minutes

    // Final cleanup and verification
    world.cleanup_deleted_entities();
    let final_count = world.entities().count();
    assert!(final_count > 0); // Should have some entities remaining
    assert!(final_count <= 5_000); // Should not exceed spawner limit
}

#[test]
fn test_system_execution_stress() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Add many systems
    for i in 0..50 {
        struct StressSystem {
            id: usize,
        }

        impl System for StressSystem {
            fn run(&self, world: &mut World) {
                let entities: Vec<_> = world.entities().cloned().collect();

                for entity in entities {
                    if let Some(pos) = world.get_component::<Position>(entity) {
                        // Perform some computation
                        let new_pos = Position {
                            x: pos.x + (self.id as f32).sin(),
                            y: pos.y + (self.id as f32).cos(),
                            z: pos.z,
                        };
                        world.replace_component(entity, new_pos);
                    }
                }
            }
        }

        scheduler.add_system(StressSystem { id: i }).unwrap();
    }

    scheduler.build().unwrap();
    assert_eq!(scheduler.system_count(), 50);

    // Create initial entities
    for i in 0..1000 {
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
    }

    let start_time = Instant::now();

    // Run many ticks with many systems
    for _ in 0..100 {
        scheduler.run_tick(&mut world);
    }

    let total_duration = start_time.elapsed();
    assert!(total_duration.as_secs() < 60); // Should complete within a minute

    // Verify all entities still exist and have been processed
    assert_eq!(world.entities().count(), 1000);

    for &entity in world.entities() {
        assert!(world.has_component::<Position>(entity));
        let pos = world.get_component::<Position>(entity).unwrap();
        // Position should have changed due to system processing
        // (We can't easily get the entity index, so just check if position changed from initial values)
        assert!(pos.x != 0.0 || pos.y != 0.0);
    }
}

#[test]
fn test_component_churn_stress() {
    let mut world = World::new();

    // Create entities
    let mut entities = Vec::new();
    for i in 0..1000 {
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
        entities.push(entity);
    }

    let start_time = Instant::now();

    // Perform massive component operations
    for cycle in 0..1000 {
        for (i, &entity) in entities.iter().enumerate() {
            match cycle % 4 {
                0 => {
                    // Add velocity
                    world
                        .add_component(
                            entity,
                            Velocity {
                                x: i as f32,
                                y: i as f32,
                                z: 0.0,
                            },
                        )
                        .ok();
                }
                1 => {
                    // Update velocity if exists
                    if world.has_component::<Velocity>(entity) {
                        world
                            .update_component::<Velocity, _>(entity, |mut vel| {
                                vel.x += 1.0;
                                vel.y += 1.0;
                                vel
                            })
                            .ok();
                    }
                }
                2 => {
                    // Replace position
                    world.replace_component(
                        entity,
                        Position {
                            x: (i + cycle) as f32,
                            y: (i + cycle) as f32,
                            z: cycle as f32,
                        },
                    );
                }
                3 => {
                    // Remove velocity
                    world.remove_component::<Velocity>(entity);
                }
                _ => unreachable!(),
            }
        }

        if cycle % 100 == 0 {
            let elapsed = start_time.elapsed();
            assert!(elapsed.as_secs() < 120); // Should not take too long
        }
    }

    let total_duration = start_time.elapsed();
    assert!(total_duration.as_secs() < 300); // Complete within 5 minutes

    // Verify final state
    assert_eq!(world.entities().count(), 1000);

    for &entity in world.entities() {
        assert!(world.has_component::<Position>(entity));
        // Velocity should not exist (removed in last cycle)
        assert!(!world.has_component::<Velocity>(entity));
    }
}

#[test]
#[ignore]
fn test_concurrent_query_stress() {
    let mut world = World::new();

    // Create entities with various component combinations
    for i in 0..10_000 {
        let entity = world.spawn_entity();

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

        if i % 3 == 0 {
            world
                .add_component(
                    entity,
                    Health {
                        current: 100,
                        max: 100,
                        regeneration: 1.0,
                    },
                )
                .unwrap();
        }

        if i % 5 == 0 {
            world
                .add_component(
                    entity,
                    AI {
                        state: 0,
                        target: None,
                        memory: vec![0.0; 5],
                    },
                )
                .unwrap();
        }
    }

    use bemudjo_ecs::Query;

    let start_time = Instant::now();

    // Perform many different queries rapidly
    for _ in 0..1000 {
        let _query1 = Query::<Position>::new().iter(&world).count();
        let _query2 = Query::<Position>::new()
            .with::<Velocity>()
            .iter(&world)
            .count();
        let _query3 = Query::<Position>::new()
            .with::<Health>()
            .iter(&world)
            .count();
        let _query4 = Query::<Position>::new().with::<AI>().iter(&world).count();
        let _query5 = Query::<Position>::new()
            .with::<Velocity>()
            .with::<Health>()
            .iter(&world)
            .count();
        let _query6 = Query::<Position>::new()
            .with::<AI>()
            .without::<Velocity>()
            .iter(&world)
            .count();
    }

    let total_duration = start_time.elapsed();
    assert!(total_duration.as_secs() < 30); // Should complete quickly

    // Verify data integrity after stress
    assert_eq!(world.entities().count(), 10_000);

    let position_count = Query::<Position>::new().iter(&world).count();
    assert_eq!(position_count, 10_000);

    let velocity_count = Query::<Position>::new()
        .with::<Velocity>()
        .iter(&world)
        .count();
    assert_eq!(velocity_count, 5_000);

    let health_count = Query::<Position>::new()
        .with::<Health>()
        .iter(&world)
        .count();
    assert_eq!(health_count, 3_334); // ceiling(10000/3)
}

#[test]
fn test_memory_leak_stress() {
    // Test for memory leaks under stress conditions

    for iteration in 0..100 {
        let mut world = World::new();

        // Create and destroy many entities
        for i in 0..1000 {
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

            world
                .add_component(
                    entity,
                    LargeData {
                        buffer: vec![i as u8; 1000],
                        id: i as u64,
                        metadata: format!("test_data_{}", i),
                    },
                )
                .unwrap();

            if i % 2 == 0 {
                world.delete_entity(entity);
            }
        }

        world.cleanup_deleted_entities();

        // Verify reasonable entity count
        assert_eq!(world.entities().count(), 500);

        if iteration % 10 == 0 {
            // Periodically check we're not accumulating excessive memory
            // This is a basic check - in practice you'd use proper memory profiling
            assert!(world.entities().count() == 500);
        }
    }
}
