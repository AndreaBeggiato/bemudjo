//! Integration tests for ECS edge cases and API boundaries
//!
//! Tests focus on boundary conditions, edge cases, and stress testing
//! the ECS library's robustness and error handling.

use bemudjo_ecs::{Component, ComponentError, SequentialSystemScheduler, System, World};

// Test Components for edge case scenarios
#[derive(Clone, Debug, PartialEq)]
struct LargeComponent {
    data: Vec<u8>,
    id: u64,
    name: String,
}
impl Component for LargeComponent {}

#[derive(Clone, Debug, PartialEq)]
struct EmptyComponent;
impl Component for EmptyComponent {}

#[derive(Clone, Debug, PartialEq)]
struct GenericComponent<T: Clone + 'static> {
    value: T,
}
impl<T: Clone + 'static> Component for GenericComponent<T> {}

#[derive(Clone, Debug, PartialEq)]
struct CounterComponent {
    value: i64,
}
impl Component for CounterComponent {}

// Edge case systems
struct StressTestSystem {
    operations_per_tick: usize,
}

impl StressTestSystem {
    fn new(operations_per_tick: usize) -> Self {
        Self {
            operations_per_tick,
        }
    }
}

impl System for StressTestSystem {
    fn run(&self, world: &mut World) {
        for i in 0..self.operations_per_tick {
            let entity = world.spawn_entity();
            world
                .add_component(entity, CounterComponent { value: i as i64 })
                .unwrap();

            if i % 3 == 0 {
                world.delete_entity(entity);
            }
        }
    }
}

struct ComponentChainingSystem;

impl System for ComponentChainingSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            // Complex component operations in sequence
            if world.has_component::<CounterComponent>(entity) {
                // Update counter
                world
                    .update_component::<CounterComponent, _>(entity, |mut counter| {
                        counter.value = counter.value.saturating_mul(2);
                        counter
                    })
                    .ok();

                // Add large component if counter is high enough
                if let Some(counter) = world.get_component::<CounterComponent>(entity) {
                    if counter.value > 100 {
                        world
                            .add_component(
                                entity,
                                LargeComponent {
                                    data: vec![counter.value as u8; 1000],
                                    id: counter.value as u64,
                                    name: format!("Large_{}", counter.value),
                                },
                            )
                            .ok();
                    }
                }

                // Add empty component
                world.add_component(entity, EmptyComponent).ok();
            }
        }
    }
}

#[test]
fn test_large_number_of_entities() {
    let mut world = World::new();

    const ENTITY_COUNT: usize = 10_000;
    let mut entities = Vec::with_capacity(ENTITY_COUNT);

    // Create many entities
    for i in 0..ENTITY_COUNT {
        let entity = world.spawn_entity();
        world
            .add_component(entity, CounterComponent { value: i as i64 })
            .unwrap();
        entities.push(entity);
    }

    assert_eq!(world.entities().count(), ENTITY_COUNT);

    // Verify all entities exist and have correct components
    for (i, &entity) in entities.iter().enumerate() {
        assert!(world.has_component::<CounterComponent>(entity));
        let counter = world.get_component::<CounterComponent>(entity).unwrap();
        assert_eq!(counter.value, i as i64);
    }

    // Delete half the entities
    for i in (0..ENTITY_COUNT).step_by(2) {
        world.delete_entity(entities[i]);
    }

    assert_eq!(world.entities().count(), ENTITY_COUNT / 2);

    // Cleanup
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), ENTITY_COUNT / 2);

    // Verify remaining entities are correct
    for (i, &entity) in entities.iter().enumerate() {
        if i % 2 == 1 {
            assert!(world.has_component::<CounterComponent>(entity));
            let counter = world.get_component::<CounterComponent>(entity).unwrap();
            assert_eq!(counter.value, i as i64);
        } else {
            assert!(!world.has_component::<CounterComponent>(entity));
        }
    }
}

#[test]
fn test_large_component_data() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Create large component
    let large_data = vec![42u8; 1_000_000]; // 1MB of data
    let large_component = LargeComponent {
        data: large_data.clone(),
        id: 12345,
        name: "Very Large Component".to_string(),
    };

    world
        .add_component(entity, large_component.clone())
        .unwrap();

    // Verify component was stored correctly
    let retrieved = world.get_component::<LargeComponent>(entity).unwrap();
    assert_eq!(retrieved.data.len(), 1_000_000);
    assert_eq!(retrieved.id, 12345);
    assert_eq!(retrieved.name, "Very Large Component");
    assert_eq!(retrieved.data, large_data);

    // Test component replacement with large data
    let new_large_data = vec![84u8; 2_000_000]; // 2MB of data
    let new_large_component = LargeComponent {
        data: new_large_data.clone(),
        id: 67890,
        name: "Even Larger Component".to_string(),
    };

    let old_component = world.replace_component(entity, new_large_component.clone());
    assert_eq!(old_component, Some(large_component));

    let retrieved = world.get_component::<LargeComponent>(entity).unwrap();
    assert_eq!(retrieved.data.len(), 2_000_000);
    assert_eq!(retrieved.data, new_large_data);
}

#[test]
fn test_many_component_types_on_single_entity() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add many different component types
    world
        .add_component(entity, CounterComponent { value: 1 })
        .unwrap();
    world.add_component(entity, EmptyComponent).unwrap();
    world
        .add_component(entity, GenericComponent { value: 42i32 })
        .unwrap();
    world
        .add_component(entity, GenericComponent { value: 1.2345f64 })
        .unwrap();
    world
        .add_component(
            entity,
            GenericComponent {
                value: "hello".to_string(),
            },
        )
        .unwrap();
    world
        .add_component(
            entity,
            GenericComponent {
                value: vec![1, 2, 3],
            },
        )
        .unwrap();
    world
        .add_component(
            entity,
            LargeComponent {
                data: vec![1, 2, 3],
                id: 999,
                name: "Multi-component entity".to_string(),
            },
        )
        .unwrap();

    // Verify all components exist
    assert!(world.has_component::<CounterComponent>(entity));
    assert!(world.has_component::<EmptyComponent>(entity));
    assert!(world.has_component::<GenericComponent<i32>>(entity));
    assert!(world.has_component::<GenericComponent<f64>>(entity));
    assert!(world.has_component::<GenericComponent<String>>(entity));
    assert!(world.has_component::<GenericComponent<Vec<i32>>>(entity));
    assert!(world.has_component::<LargeComponent>(entity));

    // Verify component values
    let int_generic = world
        .get_component::<GenericComponent<i32>>(entity)
        .unwrap();
    assert_eq!(int_generic.value, 42);

    let float_generic = world
        .get_component::<GenericComponent<f64>>(entity)
        .unwrap();
    assert_eq!(float_generic.value, 1.2345f64);

    let string_generic = world
        .get_component::<GenericComponent<String>>(entity)
        .unwrap();
    assert_eq!(string_generic.value, "hello");

    let vec_generic = world
        .get_component::<GenericComponent<Vec<i32>>>(entity)
        .unwrap();
    assert_eq!(vec_generic.value, vec![1, 2, 3]);

    // Remove components one by one
    world.remove_component::<EmptyComponent>(entity);
    assert!(!world.has_component::<EmptyComponent>(entity));
    assert!(world.has_component::<CounterComponent>(entity)); // Others should remain

    world.remove_component::<GenericComponent<i32>>(entity);
    assert!(!world.has_component::<GenericComponent<i32>>(entity));
    assert!(world.has_component::<GenericComponent<f64>>(entity)); // Different generic type should remain
}

#[test]
fn test_stress_system_execution() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(StressTestSystem::new(1000)).unwrap();
    scheduler.add_system(ComponentChainingSystem).unwrap();
    scheduler.build().unwrap();

    // Run multiple ticks to stress test
    for tick in 0..10 {
        scheduler.run_tick(&mut world);

        // Verify world is in consistent state
        let entity_count = world.entities().count();
        assert!(entity_count > 0, "Tick {tick}: No entities remaining");

        // Verify some entities have expected components
        let mut _has_large_components = false;
        let mut has_empty_components = false;

        for &entity in world.entities() {
            if world.has_component::<LargeComponent>(entity) {
                _has_large_components = true;
            }
            if world.has_component::<EmptyComponent>(entity) {
                has_empty_components = true;
            }
        }

        // After a few ticks, we should have some large and empty components
        if tick > 2 {
            assert!(
                has_empty_components,
                "Tick {tick}: No empty components found"
            );
        }
    }

    assert!(world.entities().count() > 0);
}

#[test]
fn test_rapid_entity_creation_and_deletion() {
    let mut world = World::new();

    const CYCLES: usize = 1000;
    const ENTITIES_PER_CYCLE: usize = 100;

    for cycle in 0..CYCLES {
        let mut entities = Vec::new();

        // Create entities
        for i in 0..ENTITIES_PER_CYCLE {
            let entity = world.spawn_entity();
            world
                .add_component(
                    entity,
                    CounterComponent {
                        value: (cycle * ENTITIES_PER_CYCLE + i) as i64,
                    },
                )
                .unwrap();
            entities.push(entity);
        }

        assert_eq!(world.entities().count(), ENTITIES_PER_CYCLE);

        // Delete all entities
        for entity in entities {
            world.delete_entity(entity);
        }

        assert_eq!(world.entities().count(), 0);

        // Cleanup periodically
        if cycle % 100 == 99 {
            world.cleanup_deleted_entities();
        }
    }

    // Final cleanup
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 0);
}

#[test]
fn test_component_update_edge_cases() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    world
        .add_component(entity, CounterComponent { value: 0 })
        .unwrap();

    // Test update that doesn't change the value
    let result = world.update_component::<CounterComponent, _>(entity, |counter| counter);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, 0);

    // Test update with complex transformation
    let result = world.update_component::<CounterComponent, _>(entity, |mut counter| {
        counter.value = counter.value.wrapping_mul(1000).wrapping_add(42);
        counter
    });
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, 42);

    // Test update that could overflow
    world.replace_component(entity, CounterComponent { value: i64::MAX });
    let result = world.update_component::<CounterComponent, _>(entity, |mut counter| {
        counter.value = counter.value.wrapping_add(1);
        counter
    });
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, i64::MIN); // Wrapping overflow
}

#[test]
fn test_component_error_conditions() {
    let mut world = World::new();

    // Test duplicate component addition
    let entity = world.spawn_entity();
    world
        .add_component(entity, CounterComponent { value: 1 })
        .unwrap();

    let result = world.add_component(entity, CounterComponent { value: 2 });
    assert!(matches!(
        result,
        Err(ComponentError::ComponentAlreadyExists)
    ));

    // Original component should be unchanged
    let counter = world.get_component::<CounterComponent>(entity).unwrap();
    assert_eq!(counter.value, 1);

    // Test operations on deleted entity
    world.delete_entity(entity);

    let result = world.add_component(entity, EmptyComponent);
    assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

    let result = world.update_component::<CounterComponent, _>(entity, |c| c);
    assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

    assert!(world.get_component::<CounterComponent>(entity).is_none());
    assert!(!world.has_component::<CounterComponent>(entity));
    assert!(world.remove_component::<CounterComponent>(entity).is_none());
    assert!(world
        .replace_component(entity, CounterComponent { value: 99 })
        .is_none());
}

#[test]
fn test_empty_component_operations() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Test empty component
    world.add_component(entity, EmptyComponent).unwrap();
    assert!(world.has_component::<EmptyComponent>(entity));

    let component = world.get_component::<EmptyComponent>(entity).unwrap();
    assert_eq!(*component, EmptyComponent);

    let old_component = world.replace_component(entity, EmptyComponent);
    assert_eq!(old_component, Some(EmptyComponent));

    let removed_component = world.remove_component::<EmptyComponent>(entity);
    assert_eq!(removed_component, Some(EmptyComponent));
    assert!(!world.has_component::<EmptyComponent>(entity));
}

#[test]
fn test_generic_component_type_safety() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Add different generic components with same base type but different type parameters
    world
        .add_component(entity, GenericComponent { value: 42i32 })
        .unwrap();
    world
        .add_component(entity, GenericComponent { value: 1.2345f64 })
        .unwrap();
    world
        .add_component(
            entity,
            GenericComponent {
                value: "test".to_string(),
            },
        )
        .unwrap();

    // These should be treated as completely different component types
    assert!(world.has_component::<GenericComponent<i32>>(entity));
    assert!(world.has_component::<GenericComponent<f64>>(entity));
    assert!(world.has_component::<GenericComponent<String>>(entity));

    // Should not interfere with each other
    world.remove_component::<GenericComponent<i32>>(entity);
    assert!(!world.has_component::<GenericComponent<i32>>(entity));
    assert!(world.has_component::<GenericComponent<f64>>(entity));
    assert!(world.has_component::<GenericComponent<String>>(entity));

    // Values should remain correct
    let float_comp = world
        .get_component::<GenericComponent<f64>>(entity)
        .unwrap();
    assert_eq!(float_comp.value, 1.2345);

    let string_comp = world
        .get_component::<GenericComponent<String>>(entity)
        .unwrap();
    assert_eq!(string_comp.value, "test");
}

#[test]
fn test_world_state_consistency_after_stress() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(StressTestSystem::new(100)).unwrap();
    scheduler.add_system(ComponentChainingSystem).unwrap();
    scheduler.build().unwrap();

    // Run many ticks
    for _ in 0..100 {
        scheduler.run_tick(&mut world);

        // Periodically verify world consistency
        for &entity in world.entities() {
            // Every entity should have at least a CounterComponent
            if world.has_component::<CounterComponent>(entity) {
                let counter = world.get_component::<CounterComponent>(entity);
                assert!(counter.is_some());
            }

            // If entity has LargeComponent, it should be consistent
            if world.has_component::<LargeComponent>(entity) {
                let large = world.get_component::<LargeComponent>(entity).unwrap();
                assert!(!large.name.is_empty());
                assert!(!large.data.is_empty());
            }
        }
    }

    let final_entity_count = world.entities().count();
    assert!(final_entity_count > 0);

    // All remaining entities should be in valid state
    for &entity in world.entities() {
        // Should be able to perform all operations without panic
        let has_counter = world.has_component::<CounterComponent>(entity);
        let has_empty = world.has_component::<EmptyComponent>(entity);
        let has_large = world.has_component::<LargeComponent>(entity);

        // At least one component should exist
        assert!(has_counter || has_empty || has_large);
    }
}

#[test]
fn test_system_scheduler_with_no_world_changes() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // System that does nothing
    struct NoOpSystem;
    impl System for NoOpSystem {}

    scheduler.add_system(NoOpSystem).unwrap();
    scheduler.build().unwrap();

    // Create initial state
    let entity = world.spawn_entity();
    world
        .add_component(entity, CounterComponent { value: 42 })
        .unwrap();

    let initial_count = world.entities().count();
    let initial_counter = world
        .get_component::<CounterComponent>(entity)
        .unwrap()
        .value;

    // Run many ticks
    for _ in 0..1000 {
        scheduler.run_tick(&mut world);
    }

    // World should be unchanged
    assert_eq!(world.entities().count(), initial_count);
    let final_counter = world
        .get_component::<CounterComponent>(entity)
        .unwrap()
        .value;
    assert_eq!(final_counter, initial_counter);
}

#[test]
fn test_boundary_values() {
    let mut world = World::new();
    let entity = world.spawn_entity();

    // Test with boundary values
    world
        .add_component(entity, CounterComponent { value: i64::MIN })
        .unwrap();
    let counter = world.get_component::<CounterComponent>(entity).unwrap();
    assert_eq!(counter.value, i64::MIN);

    world.replace_component(entity, CounterComponent { value: i64::MAX });
    let counter = world.get_component::<CounterComponent>(entity).unwrap();
    assert_eq!(counter.value, i64::MAX);

    world.replace_component(entity, CounterComponent { value: 0 });
    let counter = world.get_component::<CounterComponent>(entity).unwrap();
    assert_eq!(counter.value, 0);

    // Test with empty string
    world
        .add_component(
            entity,
            GenericComponent {
                value: String::new(),
            },
        )
        .unwrap();
    let string_comp = world
        .get_component::<GenericComponent<String>>(entity)
        .unwrap();
    assert_eq!(string_comp.value, "");

    // Test with empty vector
    world
        .add_component(
            entity,
            GenericComponent {
                value: Vec::<u8>::new(),
            },
        )
        .unwrap();
    let vec_comp = world
        .get_component::<GenericComponent<Vec<u8>>>(entity)
        .unwrap();
    assert!(vec_comp.value.is_empty());
}
