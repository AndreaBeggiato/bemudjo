//! System Error Handling Integration Tests
//!
//! Tests focused on error propagation, graceful failure,
//! and recovery mechanisms in system execution.

use bemudjo_ecs::{Component, ComponentError, SequentialSystemScheduler, System, World};
use std::cell::RefCell;
use std::rc::Rc;

// Test Components
#[derive(Clone, Debug, PartialEq)]
struct Counter {
    value: i32,
}
impl Component for Counter {}

#[derive(Clone, Debug, PartialEq)]
struct Health {
    current: u32,
    max: u32,
}
impl Component for Health {}

#[derive(Clone, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
struct ErrorTracker {
    errors: Vec<String>,
}
impl Component for ErrorTracker {}

// Error-prone systems for testing

struct ComponentErrorSystem {
    error_log: Rc<RefCell<Vec<String>>>,
}

impl ComponentErrorSystem {
    fn new(error_log: Rc<RefCell<Vec<String>>>) -> Self {
        Self { error_log }
    }
}

impl System for ComponentErrorSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            // Try to add duplicate components (should fail)
            if world.has_component::<Counter>(entity) {
                let result = world.add_component(entity, Counter { value: 999 });
                if let Err(ComponentError::ComponentAlreadyExists) = result {
                    self.error_log
                        .borrow_mut()
                        .push("Duplicate component error handled".to_string());
                }
            }

            // Try to update non-existent component (should fail)
            let result = world.update_component::<Health, _>(entity, |mut health| {
                health.current += 10;
                health
            });
            if let Err(ComponentError::ComponentNotFound) = result {
                self.error_log
                    .borrow_mut()
                    .push("Component not found error handled".to_string());
            }
        }

        // Try operations on fake entity
        let fake_entity = {
            let mut temp_world = World::new();
            temp_world.spawn_entity()
        };

        let result = world.add_component(fake_entity, Position { x: 0.0, y: 0.0 });
        if let Err(ComponentError::ComponentNotFound) = result {
            self.error_log
                .borrow_mut()
                .push("Fake entity error handled".to_string());
        }
    }
}

struct RecoverySystem {
    recovery_log: Rc<RefCell<Vec<String>>>,
}

impl RecoverySystem {
    fn new(recovery_log: Rc<RefCell<Vec<String>>>) -> Self {
        Self { recovery_log }
    }
}

impl System for RecoverySystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            // Try to fix missing components
            if !world.has_component::<Health>(entity) && world.has_component::<Counter>(entity) {
                world
                    .add_component(
                        entity,
                        Health {
                            current: 100,
                            max: 100,
                        },
                    )
                    .ok();
                self.recovery_log
                    .borrow_mut()
                    .push("Added missing health component".to_string());
            }

            // Try to recover from errors by checking state
            if let Some(health) = world.get_component::<Health>(entity) {
                if health.current > health.max {
                    // Fix invalid state
                    world.replace_component(
                        entity,
                        Health {
                            current: health.max,
                            max: health.max,
                        },
                    );
                    self.recovery_log
                        .borrow_mut()
                        .push("Fixed invalid health state".to_string());
                }
            }
        }
    }
}

struct PanicRecoverySystem {
    panic_count: Rc<RefCell<u32>>,
    recovery_log: Rc<RefCell<Vec<String>>>,
}

impl PanicRecoverySystem {
    fn new(panic_count: Rc<RefCell<u32>>, recovery_log: Rc<RefCell<Vec<String>>>) -> Self {
        Self {
            panic_count,
            recovery_log,
        }
    }
}

impl System for PanicRecoverySystem {
    fn run(&self, world: &mut World) {
        let count = *self.panic_count.borrow();

        // Simulate different error conditions based on run count
        match count {
            0 => {
                // First run: cause some errors but handle them
                let entity = world.spawn_entity();
                world.add_component(entity, Counter { value: -1 }).unwrap();

                // Try to add duplicate (will fail)
                world.add_component(entity, Counter { value: -2 }).ok();
                self.recovery_log
                    .borrow_mut()
                    .push("Handled first run errors".to_string());
            }
            1 => {
                // Second run: try operations on potentially deleted entities
                let entities: Vec<_> = world.entities().cloned().collect();
                for entity in entities {
                    if let Some(counter) = world.get_component::<Counter>(entity) {
                        if counter.value < 0 {
                            world.delete_entity(entity);
                            // Try more operations on deleted entity (should fail gracefully)
                            world
                                .add_component(entity, Health { current: 1, max: 1 })
                                .ok();
                            world.update_component::<Counter, _>(entity, |c| c).ok();
                        }
                    }
                }
                self.recovery_log
                    .borrow_mut()
                    .push("Handled deleted entity operations".to_string());
            }
            _ => {
                // Subsequent runs: continue normally
                let entity = world.spawn_entity();
                world
                    .add_component(
                        entity,
                        Counter {
                            value: count as i32,
                        },
                    )
                    .unwrap();
                self.recovery_log
                    .borrow_mut()
                    .push(format!("Normal operation: {count}"));
            }
        }

        *self.panic_count.borrow_mut() += 1;
    }
}

struct ValidationSystem {
    validation_errors: Rc<RefCell<Vec<String>>>,
}

impl ValidationSystem {
    fn new(validation_errors: Rc<RefCell<Vec<String>>>) -> Self {
        Self { validation_errors }
    }
}

impl System for ValidationSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            // Validate entity state
            if let Some(health) = world.get_component::<Health>(entity) {
                if health.current > health.max {
                    self.validation_errors.borrow_mut().push(format!(
                        "Invalid health: current {} > max {}",
                        health.current, health.max
                    ));
                }

                if health.max == 0 {
                    self.validation_errors
                        .borrow_mut()
                        .push("Invalid health: max is 0".to_string());
                }
            }

            if let Some(counter) = world.get_component::<Counter>(entity) {
                if counter.value < -1000 || counter.value > 1000 {
                    self.validation_errors
                        .borrow_mut()
                        .push(format!("Counter value out of bounds: {}", counter.value));
                }
            }
        }
    }
}

#[test]
fn test_component_error_handling() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let error_log = Rc::new(RefCell::new(Vec::new()));
    scheduler
        .add_system(ComponentErrorSystem::new(error_log.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Create entity with counter
    let entity = world.spawn_entity();
    world.add_component(entity, Counter { value: 42 }).unwrap();

    // Run system
    scheduler.run_tick(&mut world);

    // Verify errors were handled gracefully
    let errors = error_log.borrow().clone();
    assert!(errors.contains(&"Duplicate component error handled".to_string()));
    assert!(errors.contains(&"Component not found error handled".to_string()));
    assert!(errors.contains(&"Fake entity error handled".to_string()));

    // Entity should still exist and be unchanged
    assert_eq!(world.entities().count(), 1);
    let counter = world.get_component::<Counter>(entity).unwrap();
    assert_eq!(counter.value, 42); // Unchanged
}

#[test]
fn test_error_recovery_system() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let recovery_log = Rc::new(RefCell::new(Vec::new()));
    scheduler
        .add_system(RecoverySystem::new(recovery_log.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Create entity with missing health component
    let entity1 = world.spawn_entity();
    world.add_component(entity1, Counter { value: 1 }).unwrap();

    // Create entity with invalid health state
    let entity2 = world.spawn_entity();
    world
        .add_component(
            entity2,
            Health {
                current: 150,
                max: 100,
            },
        )
        .unwrap(); // Invalid

    // Run recovery system
    scheduler.run_tick(&mut world);

    // Verify recovery actions
    let recovery = recovery_log.borrow().clone();
    assert!(recovery.contains(&"Added missing health component".to_string()));
    assert!(recovery.contains(&"Fixed invalid health state".to_string()));

    // Verify entity1 now has health
    assert!(world.has_component::<Health>(entity1));
    let health1 = world.get_component::<Health>(entity1).unwrap();
    assert_eq!(health1.current, 100);
    assert_eq!(health1.max, 100);

    // Verify entity2 has fixed health
    let health2 = world.get_component::<Health>(entity2).unwrap();
    assert_eq!(health2.current, 100); // Fixed to max
    assert_eq!(health2.max, 100);
}

#[test]
fn test_panic_recovery_and_continuation() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let panic_count = Rc::new(RefCell::new(0u32));
    let recovery_log = Rc::new(RefCell::new(Vec::new()));

    scheduler
        .add_system(PanicRecoverySystem::new(
            panic_count.clone(),
            recovery_log.clone(),
        ))
        .unwrap();
    scheduler.build().unwrap();

    // Run multiple ticks to test different error scenarios
    for _ in 0..5 {
        scheduler.run_tick(&mut world);
        world.cleanup_deleted_entities(); // Clean up any deleted entities
    }

    let recovery = recovery_log.borrow().clone();
    assert!(recovery.contains(&"Handled first run errors".to_string()));
    assert!(recovery.contains(&"Handled deleted entity operations".to_string()));
    assert!(recovery
        .iter()
        .any(|msg| msg.starts_with("Normal operation:")));

    // Verify system continued running after errors
    assert_eq!(*panic_count.borrow(), 5);

    // Should have some entities from normal operations
    assert!(world.entities().count() > 0);
}

#[test]
fn test_validation_system_error_detection() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let validation_errors = Rc::new(RefCell::new(Vec::new()));
    scheduler
        .add_system(ValidationSystem::new(validation_errors.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Create entities with various invalid states
    let entity1 = world.spawn_entity();
    world
        .add_component(
            entity1,
            Health {
                current: 150,
                max: 100,
            },
        )
        .unwrap(); // Invalid

    let entity2 = world.spawn_entity();
    world
        .add_component(
            entity2,
            Health {
                current: 50,
                max: 0,
            },
        )
        .unwrap(); // Invalid

    let entity3 = world.spawn_entity();
    world
        .add_component(entity3, Counter { value: 2000 })
        .unwrap(); // Out of bounds

    let entity4 = world.spawn_entity();
    world
        .add_component(entity4, Counter { value: -2000 })
        .unwrap(); // Out of bounds

    let entity5 = world.spawn_entity();
    world
        .add_component(
            entity5,
            Health {
                current: 80,
                max: 100,
            },
        )
        .unwrap(); // Valid
    world.add_component(entity5, Counter { value: 42 }).unwrap(); // Valid

    // Run validation
    scheduler.run_tick(&mut world);

    let errors = validation_errors.borrow().clone();

    assert!(errors.iter().any(|e| e.contains("current 150 > max 100")));
    assert!(errors.iter().any(|e| e.contains("current 50 > max 0")));
    assert!(errors.contains(&"Invalid health: max is 0".to_string()));
    assert!(errors
        .iter()
        .any(|e| e.contains("Counter value out of bounds: 2000")));
    assert!(errors
        .iter()
        .any(|e| e.contains("Counter value out of bounds: -2000")));

    // Should have found 5 errors (entity2 has 2 health issues, entity5 is valid)
    assert_eq!(errors.len(), 5);
}

#[test]
fn test_system_error_isolation() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let error_log = Rc::new(RefCell::new(Vec::new()));
    let recovery_log = Rc::new(RefCell::new(Vec::new()));

    // Add multiple systems - errors in one shouldn't affect others
    scheduler
        .add_system(ComponentErrorSystem::new(error_log.clone()))
        .unwrap();
    scheduler
        .add_system(RecoverySystem::new(recovery_log.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Create entity that will trigger errors in first system and recovery in second
    let entity = world.spawn_entity();
    world.add_component(entity, Counter { value: 42 }).unwrap(); // Will trigger duplicate error and missing health recovery

    // Run both systems
    scheduler.run_tick(&mut world);

    // Verify both systems executed despite errors
    let errors = error_log.borrow().clone();
    let recovery = recovery_log.borrow().clone();

    assert!(!errors.is_empty()); // Errors occurred
    assert!(!recovery.is_empty()); // Recovery occurred

    // Entity should have been fixed by recovery system
    assert!(world.has_component::<Health>(entity));
    let health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(health.current, 100);
}

#[test]
fn test_cascading_error_handling() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    struct CascadingErrorSystem {
        step: Rc<RefCell<u32>>,
        error_log: Rc<RefCell<Vec<String>>>,
    }

    impl CascadingErrorSystem {
        fn new(step: Rc<RefCell<u32>>, error_log: Rc<RefCell<Vec<String>>>) -> Self {
            Self { step, error_log }
        }
    }

    impl System for CascadingErrorSystem {
        fn run(&self, world: &mut World) {
            let current_step = *self.step.borrow();

            match current_step {
                0 => {
                    // Create entity
                    let entity = world.spawn_entity();
                    world.add_component(entity, Counter { value: 0 }).unwrap();
                    self.error_log
                        .borrow_mut()
                        .push("Step 0: Created entity".to_string());
                }
                1 => {
                    // Try to cause error with first entity
                    let first_entity = world.entities().next().cloned();
                    if let Some(entity) = first_entity {
                        world.add_component(entity, Counter { value: 1 }).ok(); // Will fail
                                                                                // Continue with valid operations
                        world
                            .add_component(
                                entity,
                                Health {
                                    current: 100,
                                    max: 100,
                                },
                            )
                            .ok();
                        self.error_log
                            .borrow_mut()
                            .push("Step 1: Handled errors and continued".to_string());
                    }
                }
                2 => {
                    // Delete entity and try operations
                    let first_entity = world.entities().next().cloned();
                    if let Some(entity) = first_entity {
                        world.delete_entity(entity);
                        // Try operations on deleted entity
                        world.update_component::<Counter, _>(entity, |c| c).ok();
                        world
                            .add_component(entity, Position { x: 0.0, y: 0.0 })
                            .ok();
                        self.error_log
                            .borrow_mut()
                            .push("Step 2: Handled operations on deleted entity".to_string());
                    }
                }
                _ => {
                    // Recovery: create new entity
                    let entity = world.spawn_entity();
                    world
                        .add_component(
                            entity,
                            Counter {
                                value: current_step as i32,
                            },
                        )
                        .unwrap();
                    self.error_log
                        .borrow_mut()
                        .push(format!("Step {current_step}: Recovery"));
                }
            }

            *self.step.borrow_mut() += 1;
        }
    }

    let step = Rc::new(RefCell::new(0u32));
    let error_log = Rc::new(RefCell::new(Vec::new()));

    scheduler
        .add_system(CascadingErrorSystem::new(step.clone(), error_log.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Run multiple ticks to test cascading scenarios
    for _ in 0..5 {
        scheduler.run_tick(&mut world);
        world.cleanup_deleted_entities();
    }

    let errors = error_log.borrow().clone();
    assert!(errors.iter().any(|e| e.contains("Step 0: Created entity")));
    assert!(errors
        .iter()
        .any(|e| e.contains("Step 1: Handled errors and continued")));
    assert!(errors
        .iter()
        .any(|e| e.contains("Step 2: Handled operations on deleted entity")));
    assert!(errors.iter().any(|e| e.contains("Recovery")));

    // System should have continued running through all steps
    assert_eq!(*step.borrow(), 5);

    // Should have recovered with new entities
    assert!(world.entities().count() > 0);
}

#[test]
fn test_error_handling_with_complex_state() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let error_log = Rc::new(RefCell::new(Vec::new()));
    let recovery_log = Rc::new(RefCell::new(Vec::new()));
    let validation_errors = Rc::new(RefCell::new(Vec::new()));

    scheduler
        .add_system(ComponentErrorSystem::new(error_log.clone()))
        .unwrap();
    scheduler
        .add_system(RecoverySystem::new(recovery_log.clone()))
        .unwrap();
    scheduler
        .add_system(ValidationSystem::new(validation_errors.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Create complex initial state
    for i in 0..10 {
        let entity = world.spawn_entity();

        if i % 2 == 0 {
            world.add_component(entity, Counter { value: i }).unwrap();
        }

        if i % 3 == 0 {
            // Some invalid health states
            world
                .add_component(
                    entity,
                    Health {
                        current: if i == 6 { 200 } else { 50 },
                        max: if i == 9 { 0 } else { 100 },
                    },
                )
                .unwrap();
        }

        if i % 5 == 0 {
            world
                .add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: 0.0,
                    },
                )
                .unwrap();
        }
    }

    assert_eq!(world.entities().count(), 10);

    // Run systems multiple times
    for _ in 0..3 {
        scheduler.run_tick(&mut world);
    }

    // Verify all systems handled their errors and continued
    assert!(!error_log.borrow().is_empty());
    assert!(!recovery_log.borrow().is_empty());
    assert!(!validation_errors.borrow().is_empty());

    // All entities should still exist
    assert_eq!(world.entities().count(), 10);

    // All counter entities should now have health (added by recovery system)
    for &entity in world.entities() {
        if world.has_component::<Counter>(entity) {
            assert!(world.has_component::<Health>(entity));
        }
    }

    // Invalid health states should still be detected (validation runs after recovery)
    let validation = validation_errors.borrow().clone();
    assert!(validation.iter().any(|e| e.contains("max is 0")));
}
