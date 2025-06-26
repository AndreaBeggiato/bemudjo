//! System Scheduler Integration Tests
//!
//! Tests focused on system scheduler behavior, execution order,
//! and system lifecycle management.

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

// Test Systems
struct CounterSystem {
    increment: i32,
    execution_log: Rc<RefCell<Vec<String>>>,
}

impl CounterSystem {
    fn new(increment: i32, log: Rc<RefCell<Vec<String>>>) -> Self {
        Self {
            increment,
            execution_log: log,
        }
    }
}

impl System for CounterSystem {
    fn before_run(&self, _world: &World) {
        self.execution_log
            .borrow_mut()
            .push(format!("Before CounterSystem({})", self.increment));
    }

    fn run(&self, world: &mut World) {
        self.execution_log
            .borrow_mut()
            .push(format!("Run CounterSystem({})", self.increment));

        let entities: Vec<_> = world.entities().cloned().collect();
        for entity in entities {
            if world.has_component::<Counter>(entity) {
                world
                    .update_component::<Counter, _>(entity, |mut counter| {
                        counter.value += self.increment;
                        counter
                    })
                    .ok();
            }
        }
    }

    fn after_run(&self, _world: &World) {
        self.execution_log
            .borrow_mut()
            .push(format!("After CounterSystem({})", self.increment));
    }
}

struct MovementSystem {
    delta_time: f32,
    execution_log: Rc<RefCell<Vec<String>>>,
}

impl MovementSystem {
    fn new(delta_time: f32, log: Rc<RefCell<Vec<String>>>) -> Self {
        Self {
            delta_time,
            execution_log: log,
        }
    }
}

impl System for MovementSystem {
    fn run(&self, world: &mut World) {
        self.execution_log
            .borrow_mut()
            .push("Run MovementSystem".to_string());

        let entities: Vec<_> = world.entities().cloned().collect();
        for entity in entities {
            if let (Some(pos), Some(vel)) = (
                world.get_component::<Position>(entity),
                world.get_component::<Velocity>(entity),
            ) {
                let new_pos = Position {
                    x: pos.x + vel.x * self.delta_time,
                    y: pos.y + vel.y * self.delta_time,
                };
                world.replace_component(entity, new_pos);
            }
        }
    }
}

struct EntitySpawnerSystem {
    spawn_count: usize,
    execution_log: Rc<RefCell<Vec<String>>>,
}

impl EntitySpawnerSystem {
    fn new(spawn_count: usize, log: Rc<RefCell<Vec<String>>>) -> Self {
        Self {
            spawn_count,
            execution_log: log,
        }
    }
}

impl System for EntitySpawnerSystem {
    fn run(&self, world: &mut World) {
        self.execution_log
            .borrow_mut()
            .push(format!("Run EntitySpawnerSystem({})", self.spawn_count));

        for i in 0..self.spawn_count {
            let entity = world.spawn_entity();
            world
                .add_component(entity, Counter { value: i as i32 })
                .unwrap();
        }
    }
}

struct CleanupSystem {
    execution_log: Rc<RefCell<Vec<String>>>,
}

impl CleanupSystem {
    fn new(log: Rc<RefCell<Vec<String>>>) -> Self {
        Self { execution_log: log }
    }
}

impl System for CleanupSystem {
    fn run(&self, world: &mut World) {
        self.execution_log
            .borrow_mut()
            .push("Run CleanupSystem".to_string());

        let entities: Vec<_> = world.entities().cloned().collect();
        for entity in entities {
            if let Some(health) = world.get_component::<Health>(entity) {
                if health.current == 0 {
                    world.delete_entity(entity);
                }
            }
        }
    }
}

#[test]
fn test_basic_scheduler_execution() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // Add systems
    scheduler
        .add_system(CounterSystem::new(1, log.clone()))
        .unwrap();
    scheduler
        .add_system(CounterSystem::new(10, log.clone()))
        .unwrap();

    scheduler.build().unwrap();
    assert_eq!(scheduler.system_count(), 2);

    // Create test entity
    let entity = world.spawn_entity();
    world.add_component(entity, Counter { value: 0 }).unwrap();

    // Run one tick
    scheduler.run_tick(&mut world);

    // Verify execution order - all before_run, then all run, then all after_run
    let execution_log = log.borrow().clone();
    assert_eq!(execution_log.len(), 6); // 3 phases * 2 systems

    assert_eq!(execution_log[0], "Before CounterSystem(1)");
    assert_eq!(execution_log[1], "Before CounterSystem(10)");
    assert_eq!(execution_log[2], "Run CounterSystem(1)");
    assert_eq!(execution_log[3], "Run CounterSystem(10)");
    assert_eq!(execution_log[4], "After CounterSystem(1)");
    assert_eq!(execution_log[5], "After CounterSystem(10)");

    // Verify counter value
    let counter = world.get_component::<Counter>(entity).unwrap();
    assert_eq!(counter.value, 11); // 0 + 1 + 10
}

#[test]
fn test_system_execution_order() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // Add systems in specific order
    scheduler
        .add_system(EntitySpawnerSystem::new(2, log.clone()))
        .unwrap();
    scheduler
        .add_system(CounterSystem::new(5, log.clone()))
        .unwrap();
    scheduler
        .add_system(MovementSystem::new(0.016, log.clone()))
        .unwrap();

    scheduler.build().unwrap();
    assert_eq!(scheduler.system_count(), 3);

    // Run one tick
    scheduler.run_tick(&mut world);

    // Verify execution order
    let execution_log = log.borrow().clone();
    let run_calls: Vec<_> = execution_log
        .iter()
        .filter(|msg| msg.starts_with("Run "))
        .collect();

    assert_eq!(run_calls.len(), 3);
    assert_eq!(run_calls[0], "Run EntitySpawnerSystem(2)");
    assert_eq!(run_calls[1], "Run CounterSystem(5)");
    assert_eq!(run_calls[2], "Run MovementSystem");

    // Verify entities were spawned and processed
    assert_eq!(world.entities().count(), 2);

    for &entity in world.entities() {
        let counter = world.get_component::<Counter>(entity).unwrap();
        assert!(counter.value >= 5); // Should have been incremented by CounterSystem
    }
}

#[test]
fn test_multiple_tick_execution() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    scheduler
        .add_system(CounterSystem::new(1, log.clone()))
        .unwrap();

    scheduler.build().unwrap();

    // Create test entity
    let entity = world.spawn_entity();
    world.add_component(entity, Counter { value: 0 }).unwrap();

    // Run multiple ticks
    for tick in 1..=5 {
        scheduler.run_tick(&mut world);

        let counter = world.get_component::<Counter>(entity).unwrap();
        assert_eq!(counter.value, tick);
    }

    // Verify each tick executed properly
    let execution_log = log.borrow().clone();
    let run_count = execution_log
        .iter()
        .filter(|msg| msg.starts_with("Run "))
        .count();
    assert_eq!(run_count, 5);
}

#[test]
fn test_system_with_entity_creation_and_deletion() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    scheduler
        .add_system(EntitySpawnerSystem::new(3, log.clone()))
        .unwrap();
    scheduler
        .add_system(CleanupSystem::new(log.clone()))
        .unwrap();

    scheduler.build().unwrap();

    // Create entities with different health values
    let healthy_entity = world.spawn_entity();
    world
        .add_component(
            healthy_entity,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();

    let dead_entity = world.spawn_entity();
    world
        .add_component(
            dead_entity,
            Health {
                current: 0,
                max: 100,
            },
        )
        .unwrap();

    assert_eq!(world.entities().count(), 2);

    // Run one tick
    scheduler.run_tick(&mut world);

    // EntitySpawnerSystem should have created 3 new entities
    // CleanupSystem should have deleted the dead entity
    assert_eq!(world.entities().count(), 4); // 1 healthy + 3 new - 1 dead = 3, but deleted entities count until cleanup

    // Verify dead entity is marked for deletion
    assert!(!world.has_component::<Health>(dead_entity));

    // Cleanup
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 4); // 1 healthy + 3 new

    // Verify healthy entity still exists
    assert!(world.has_component::<Health>(healthy_entity));
}

#[test]
fn test_empty_scheduler() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    assert_eq!(scheduler.system_count(), 0);

    scheduler.build().unwrap();

    // Create some entities
    let entity = world.spawn_entity();
    world.add_component(entity, Counter { value: 42 }).unwrap();

    let initial_counter = world.get_component::<Counter>(entity).unwrap().value;

    // Run tick with no systems
    scheduler.run_tick(&mut world);

    // World should be unchanged
    let final_counter = world.get_component::<Counter>(entity).unwrap().value;
    assert_eq!(initial_counter, final_counter);
    assert_eq!(world.entities().count(), 1);
}

#[test]
fn test_scheduler_with_complex_systems() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // Add systems in a specific order to test interactions
    scheduler
        .add_system(EntitySpawnerSystem::new(1, log.clone()))
        .unwrap();
    scheduler
        .add_system(MovementSystem::new(1.0, log.clone()))
        .unwrap();
    scheduler
        .add_system(CounterSystem::new(100, log.clone()))
        .unwrap();

    scheduler.build().unwrap();

    // Create initial entities
    let moving_entity = world.spawn_entity();
    world
        .add_component(moving_entity, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(moving_entity, Velocity { x: 2.0, y: 3.0 })
        .unwrap();

    let counter_entity = world.spawn_entity();
    world
        .add_component(counter_entity, Counter { value: 0 })
        .unwrap();

    assert_eq!(world.entities().count(), 2);

    // Run multiple ticks
    for tick in 1..=3 {
        scheduler.run_tick(&mut world);

        // Verify movement
        let pos = world.get_component::<Position>(moving_entity).unwrap();
        assert_eq!(pos.x, tick as f32 * 2.0);
        assert_eq!(pos.y, tick as f32 * 3.0);

        // Verify counter updates (original entity + newly spawned entities each tick)
        let counter = world.get_component::<Counter>(counter_entity).unwrap();
        assert_eq!(counter.value, tick * 100);

        // Verify entity count (original 2 + 1 per tick from spawner)
        assert_eq!(world.entities().count(), 2 + tick as usize);
    }
}

#[test]
fn test_system_phase_interactions() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // System that modifies world state in different phases
    struct PhaseTestSystem {
        phase_log: Rc<RefCell<Vec<(String, usize)>>>,
    }

    impl PhaseTestSystem {
        fn new(log: Rc<RefCell<Vec<(String, usize)>>>) -> Self {
            Self { phase_log: log }
        }
    }

    impl System for PhaseTestSystem {
        fn before_run(&self, world: &World) {
            let count = world.entities().count();
            self.phase_log
                .borrow_mut()
                .push(("before".to_string(), count));
        }

        fn run(&self, world: &mut World) {
            let count = world.entities().count();
            self.phase_log
                .borrow_mut()
                .push(("run_start".to_string(), count));

            // Spawn an entity
            let entity = world.spawn_entity();
            world.add_component(entity, Counter { value: 1 }).unwrap();

            let count = world.entities().count();
            self.phase_log
                .borrow_mut()
                .push(("run_end".to_string(), count));
        }

        fn after_run(&self, world: &World) {
            let count = world.entities().count();
            self.phase_log
                .borrow_mut()
                .push(("after".to_string(), count));
        }
    }

    let log = Rc::new(RefCell::new(Vec::new()));
    scheduler
        .add_system(PhaseTestSystem::new(log.clone()))
        .unwrap();

    scheduler.build().unwrap();

    // Run one tick
    scheduler.run_tick(&mut world);

    let phase_log = log.borrow().clone();
    assert_eq!(phase_log.len(), 4);

    assert_eq!(phase_log[0], ("before".to_string(), 0));
    assert_eq!(phase_log[1], ("run_start".to_string(), 0));
    assert_eq!(phase_log[2], ("run_end".to_string(), 1));
    assert_eq!(phase_log[3], ("after".to_string(), 1));

    assert_eq!(world.entities().count(), 1);
}

#[test]
fn test_scheduler_error_resilience() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // System that might fail operations
    struct ErrorProneSystem;

    impl System for ErrorProneSystem {
        fn run(&self, world: &mut World) {
            // Try to operate on non-existent entities
            let fake_entity = {
                let mut temp_world = World::new();
                temp_world.spawn_entity()
            };

            // These operations should fail gracefully
            world.add_component(fake_entity, Counter { value: 1 }).ok();
            world
                .update_component::<Counter, _>(fake_entity, |c| c)
                .ok();

            // This should succeed
            let entity = world.spawn_entity();
            world.add_component(entity, Counter { value: 42 }).unwrap();
        }
    }

    scheduler.add_system(ErrorProneSystem).unwrap();
    scheduler.build().unwrap();

    // Should not panic despite errors
    scheduler.run_tick(&mut world);

    // Verify successful operations completed
    assert_eq!(world.entities().count(), 1);
    let entity = world.entities().next().cloned().unwrap();
    let counter = world.get_component::<Counter>(entity).unwrap();
    assert_eq!(counter.value, 42);
}

#[test]
fn test_scheduler_performance_with_many_systems() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // Add many systems
    for _i in 0..100 {
        scheduler
            .add_system(CounterSystem::new(1, log.clone()))
            .unwrap();
    }

    scheduler.build().unwrap();
    assert_eq!(scheduler.system_count(), 100);

    // Create test entity
    let entity = world.spawn_entity();
    world.add_component(entity, Counter { value: 0 }).unwrap();

    let start_time = std::time::Instant::now();

    // Run one tick
    scheduler.run_tick(&mut world);

    let duration = start_time.elapsed();
    assert!(duration.as_millis() < 100); // Should complete quickly

    // Verify all systems executed
    let counter = world.get_component::<Counter>(entity).unwrap();
    assert_eq!(counter.value, 100); // Each system adds 1

    let execution_log = log.borrow().clone();
    let run_count = execution_log
        .iter()
        .filter(|msg| msg.starts_with("Run "))
        .count();
    assert_eq!(run_count, 100);
}
