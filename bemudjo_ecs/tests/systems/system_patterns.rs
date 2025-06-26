//! Integration tests for ECS System API patterns
//!
//! Tests focus on different system implementation patterns and
//! advanced usage scenarios of the System trait.

use bemudjo_ecs::{Component, SequentialSystemScheduler, System, World};
use std::cell::RefCell;
use std::rc::Rc;

// Test Components
#[derive(Clone, Debug, PartialEq)]
struct Counter {
    value: i32,
}
impl Component for Counter {}

#[derive(Clone, Debug, PartialEq)]
struct Tag {
    name: String,
}
impl Component for Tag {}

#[derive(Clone, Debug, PartialEq)]
struct Timer {
    remaining: f32,
}
impl Component for Timer {}

// System that demonstrates state sharing between phases
struct StatefulSystem {
    shared_state: Rc<RefCell<Vec<String>>>,
}

impl StatefulSystem {
    fn new() -> Self {
        Self {
            shared_state: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl System for StatefulSystem {
    fn before_run(&self, world: &World) {
        let entity_count = world.entities().count();
        self.shared_state
            .borrow_mut()
            .push(format!("Before: {} entities", entity_count));
    }

    fn run(&self, world: &mut World) {
        // Add a counter to track system runs
        let entity = world.spawn_entity();
        world.add_component(entity, Counter { value: 1 }).unwrap();

        self.shared_state
            .borrow_mut()
            .push("Run: Added entity".to_string());
    }

    fn after_run(&self, world: &World) {
        let entity_count = world.entities().count();
        self.shared_state
            .borrow_mut()
            .push(format!("After: {} entities", entity_count));
    }
}

// System that only uses before_run phase
struct ReadOnlySystem {
    observations: Rc<RefCell<Vec<i32>>>,
}

impl ReadOnlySystem {
    fn new() -> Self {
        Self {
            observations: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl System for ReadOnlySystem {
    fn before_run(&self, world: &World) {
        let total_counter_value: i32 = world
            .entities()
            .filter_map(|&entity| world.get_component::<Counter>(entity))
            .map(|counter| counter.value)
            .sum();

        self.observations.borrow_mut().push(total_counter_value);
    }
}

// System that only uses after_run phase
struct PostProcessSystem {
    results: Rc<RefCell<Vec<String>>>,
}

impl PostProcessSystem {
    fn new() -> Self {
        Self {
            results: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl System for PostProcessSystem {
    fn after_run(&self, world: &World) {
        for &entity in world.entities() {
            if let Some(tag) = world.get_component::<Tag>(entity) {
                self.results.borrow_mut().push(tag.name.clone());
            }
        }
    }
}

// System that demonstrates complex component queries
struct QuerySystem;

impl System for QuerySystem {
    fn run(&self, world: &mut World) {
        // Find entities with both Counter and Tag components
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if world.has_component::<Counter>(entity) && world.has_component::<Tag>(entity) {
                // Increment counter for tagged entities
                world
                    .update_component::<Counter, _>(entity, |mut counter| {
                        counter.value += 10;
                        counter
                    })
                    .ok();
            }
        }
    }
}

// System that demonstrates timer-based behavior
struct TimerSystem;

impl System for TimerSystem {
    fn run(&self, world: &mut World) {
        let dt = 0.016; // Simulate 60 FPS (16ms per frame)
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if world.has_component::<Timer>(entity) {
                world
                    .update_component::<Timer, _>(entity, |mut timer| {
                        timer.remaining -= dt;
                        timer
                    })
                    .ok();

                // Remove expired timers
                if let Some(timer) = world.get_component::<Timer>(entity) {
                    if timer.remaining <= 0.0 {
                        world.remove_component::<Timer>(entity);
                        // Add a tag to mark completion
                        world
                            .add_component(
                                entity,
                                Tag {
                                    name: "Timer Expired".to_string(),
                                },
                            )
                            .ok();
                    }
                }
            }
        }
    }
}

#[test]
fn test_system_phases_execution_order() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let stateful_system = StatefulSystem::new();
    let state_handle = stateful_system.shared_state.clone();

    scheduler.add_system(stateful_system).unwrap();

    scheduler.build().unwrap();

    // Initial run
    scheduler.run_tick(&mut world);

    let state = state_handle.borrow().clone();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], "Before: 0 entities");
    assert_eq!(state[1], "Run: Added entity");
    assert_eq!(state[2], "After: 1 entities");

    // Second run
    scheduler.run_tick(&mut world);

    let state = state_handle.borrow().clone();
    assert_eq!(state.len(), 6);
    assert_eq!(state[3], "Before: 1 entities");
    assert_eq!(state[4], "Run: Added entity");
    assert_eq!(state[5], "After: 2 entities");
}

#[test]
fn test_read_only_system_pattern() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let read_only_system = ReadOnlySystem::new();
    let observations_handle = read_only_system.observations.clone();

    scheduler.add_system(read_only_system).unwrap();

    scheduler.build().unwrap();

    // Create entities with counters
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    world.add_component(entity1, Counter { value: 5 }).unwrap();
    world.add_component(entity2, Counter { value: 10 }).unwrap();
    world.add_component(entity3, Counter { value: 15 }).unwrap();

    // Run system
    scheduler.run_tick(&mut world);

    let observations = observations_handle.borrow().clone();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0], 30); // 5 + 10 + 15

    // Add another entity and run again
    let entity4 = world.spawn_entity();
    world.add_component(entity4, Counter { value: 20 }).unwrap();

    scheduler.run_tick(&mut world);

    let observations = observations_handle.borrow().clone();
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[1], 50); // 5 + 10 + 15 + 20
}

#[test]
fn test_post_process_system_pattern() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let post_process_system = PostProcessSystem::new();
    let results_handle = post_process_system.results.clone();

    scheduler.add_system(post_process_system).unwrap();

    scheduler.build().unwrap();

    // Create entities with tags
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    world
        .add_component(
            entity1,
            Tag {
                name: "Player".to_string(),
            },
        )
        .unwrap();
    world
        .add_component(
            entity2,
            Tag {
                name: "Enemy".to_string(),
            },
        )
        .unwrap();
    world.add_component(entity3, Counter { value: 42 }).unwrap(); // No tag

    // Run system
    scheduler.run_tick(&mut world);

    let results = results_handle.borrow().clone();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&"Player".to_string()));
    assert!(results.contains(&"Enemy".to_string()));

    // Add more tagged entities
    let entity4 = world.spawn_entity();
    world
        .add_component(
            entity4,
            Tag {
                name: "NPC".to_string(),
            },
        )
        .unwrap();

    scheduler.run_tick(&mut world);

    let results = results_handle.borrow().clone();
    assert_eq!(results.len(), 5); // Previous 2 + new 3
    assert!(results.contains(&"NPC".to_string()));
}

#[test]
fn test_complex_system_interactions() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Add multiple systems that interact
    scheduler.add_system(QuerySystem).unwrap();
    scheduler.add_system(TimerSystem).unwrap();

    scheduler.build().unwrap();

    // Create entities with different component combinations
    let entity1 = world.spawn_entity();
    world.add_component(entity1, Counter { value: 1 }).unwrap();
    world
        .add_component(
            entity1,
            Tag {
                name: "Tagged Counter".to_string(),
            },
        )
        .unwrap();

    let entity2 = world.spawn_entity();
    world.add_component(entity2, Counter { value: 2 }).unwrap();
    // No tag - should not be affected by QuerySystem

    let entity3 = world.spawn_entity();
    world
        .add_component(entity3, Timer { remaining: 0.04 })
        .unwrap(); // Will expire in ~2.5 ticks

    let entity4 = world.spawn_entity();
    world
        .add_component(entity4, Timer { remaining: 0.08 })
        .unwrap(); // Will expire in ~5 ticks
    world
        .add_component(entity4, Counter { value: 100 })
        .unwrap();

    // Run first tick
    scheduler.run_tick(&mut world);

    // Verify QuerySystem effects
    let counter1 = world.get_component::<Counter>(entity1).unwrap();
    assert_eq!(counter1.value, 11); // 1 + 10

    let counter2 = world.get_component::<Counter>(entity2).unwrap();
    assert_eq!(counter2.value, 2); // Unchanged (no tag)

    // Verify TimerSystem effects
    assert!(world.has_component::<Timer>(entity3));
    assert!(world.has_component::<Timer>(entity4));

    // Run two more ticks to expire entity3's timer
    scheduler.run_tick(&mut world);
    scheduler.run_tick(&mut world);

    // entity3's timer should have expired
    assert!(!world.has_component::<Timer>(entity3));
    assert!(world.has_component::<Tag>(entity3)); // Should have "Timer Expired" tag

    let expired_tag = world.get_component::<Tag>(entity3).unwrap();
    assert_eq!(expired_tag.name, "Timer Expired");

    // entity4's timer should still be running
    assert!(world.has_component::<Timer>(entity4));

    // Run more ticks to expire entity4's timer
    for _ in 0..4 {
        scheduler.run_tick(&mut world);
    }

    // entity4's timer should now be expired
    assert!(!world.has_component::<Timer>(entity4));
    assert!(world.has_component::<Tag>(entity4));

    // entity4 should have been affected by QuerySystem after getting the tag
    let counter4 = world.get_component::<Counter>(entity4).unwrap();
    assert!(counter4.value > 100); // Should have been incremented
}

#[test]
fn test_system_with_no_entities() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let stateful_system = StatefulSystem::new();
    let read_only_system = ReadOnlySystem::new();
    let post_process_system = PostProcessSystem::new();

    let state_handle = stateful_system.shared_state.clone();
    let obs_handle = read_only_system.observations.clone();
    let results_handle = post_process_system.results.clone();

    scheduler.add_system(stateful_system).unwrap();
    scheduler.add_system(read_only_system).unwrap();
    scheduler.add_system(post_process_system).unwrap();

    scheduler.build().unwrap();

    // Run on empty world
    scheduler.run_tick(&mut world);

    // StatefulSystem should still execute all phases
    let state = state_handle.borrow().clone();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], "Before: 0 entities");
    assert_eq!(state[1], "Run: Added entity");
    assert_eq!(state[2], "After: 1 entities");

    // ReadOnlySystem should observe sum of 0
    let observations = obs_handle.borrow().clone();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0], 0);

    // PostProcessSystem should have no results initially
    let results = results_handle.borrow().clone();
    assert_eq!(results.len(), 0);

    // Verify world now has one entity (from StatefulSystem)
    assert_eq!(world.entities().count(), 1);
}

#[test]
fn test_system_error_resilience() {
    // Test that systems handle missing components gracefully
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(QuerySystem).unwrap();
    scheduler.add_system(TimerSystem).unwrap();

    scheduler.build().unwrap();

    // Create entities with partial component sets
    let entity1 = world.spawn_entity();
    world.add_component(entity1, Counter { value: 5 }).unwrap();
    // Missing Tag component

    let entity2 = world.spawn_entity();
    world
        .add_component(
            entity2,
            Tag {
                name: "Lonely Tag".to_string(),
            },
        )
        .unwrap();
    // Missing Counter component

    let entity3 = world.spawn_entity();
    // No components at all

    // Should run without panicking
    scheduler.run_tick(&mut world);

    // Verify entities are unaffected (since they don't have complete component sets)
    let counter1 = world.get_component::<Counter>(entity1).unwrap();
    assert_eq!(counter1.value, 5); // Unchanged

    let tag2 = world.get_component::<Tag>(entity2).unwrap();
    assert_eq!(tag2.name, "Lonely Tag"); // Unchanged

    // entity3 should still exist but unchanged
    assert!(world.entities().any(|&e| e == entity3));
}

#[test]
fn test_multiple_systems_same_type() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Add multiple instances of the same system type
    scheduler.add_system(QuerySystem).unwrap();
    scheduler.add_system(QuerySystem).unwrap();
    scheduler.add_system(QuerySystem).unwrap();

    scheduler.build().unwrap();

    assert_eq!(scheduler.system_count(), 3);

    let entity = world.spawn_entity();
    world.add_component(entity, Counter { value: 1 }).unwrap();
    world
        .add_component(
            entity,
            Tag {
                name: "Multi".to_string(),
            },
        )
        .unwrap();

    // Run one tick - each QuerySystem should increment by 10
    scheduler.run_tick(&mut world);

    let counter = world.get_component::<Counter>(entity).unwrap();
    assert_eq!(counter.value, 31); // 1 + 10 + 10 + 10
}

#[test]
fn test_system_execution_with_entity_deletion() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // System that deletes entities based on a condition
    struct DeletionSystem;
    impl System for DeletionSystem {
        fn run(&self, world: &mut World) {
            let entities: Vec<_> = world.entities().cloned().collect();

            for entity in entities {
                if let Some(counter) = world.get_component::<Counter>(entity) {
                    if counter.value < 0 {
                        world.delete_entity(entity);
                    }
                }
            }
        }
    }

    scheduler.add_system(DeletionSystem).unwrap();
    scheduler.add_system(QuerySystem).unwrap(); // This should handle deleted entities gracefully

    scheduler.build().unwrap();

    // Create entities with different counter values
    let entity1 = world.spawn_entity();
    world.add_component(entity1, Counter { value: 5 }).unwrap();
    world
        .add_component(
            entity1,
            Tag {
                name: "Positive".to_string(),
            },
        )
        .unwrap();

    let entity2 = world.spawn_entity();
    world.add_component(entity2, Counter { value: -5 }).unwrap();
    world
        .add_component(
            entity2,
            Tag {
                name: "Negative".to_string(),
            },
        )
        .unwrap();

    let entity3 = world.spawn_entity();
    world.add_component(entity3, Counter { value: 0 }).unwrap();
    world
        .add_component(
            entity3,
            Tag {
                name: "Zero".to_string(),
            },
        )
        .unwrap();

    assert_eq!(world.entities().count(), 3);

    // Run tick - entity2 should be deleted, others should have counters incremented
    scheduler.run_tick(&mut world);

    assert_eq!(world.entities().count(), 2);
    assert!(!world.has_component::<Counter>(entity2)); // Should be gone

    // Verify remaining entities were processed by QuerySystem
    let counter1 = world.get_component::<Counter>(entity1).unwrap();
    assert_eq!(counter1.value, 15); // 5 + 10

    let counter3 = world.get_component::<Counter>(entity3).unwrap();
    assert_eq!(counter3.value, 10); // 0 + 10
}

#[test]
fn test_empty_scheduler() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    assert_eq!(scheduler.system_count(), 0);

    scheduler.build().unwrap();

    // Should handle empty scheduler gracefully
    scheduler.run_tick(&mut world);

    // World should be unchanged
    assert_eq!(world.entities().count(), 0);
}

#[test]
fn test_system_with_default_implementations() {
    struct MinimalSystem;
    impl System for MinimalSystem {
        fn run(&self, world: &mut World) {
            // Only implement run, use defaults for before_run and after_run
            let entity = world.spawn_entity();
            world.add_component(entity, Counter { value: 42 }).unwrap();
        }
    }

    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(MinimalSystem).unwrap();

    scheduler.build().unwrap();

    scheduler.build().unwrap();

    scheduler.run_tick(&mut world);

    assert_eq!(world.entities().count(), 1);
    let entity = world.entities().next().cloned().unwrap();
    let counter = world.get_component::<Counter>(entity).unwrap();
    assert_eq!(counter.value, 42);
}
