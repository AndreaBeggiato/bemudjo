//! System Integration Tests for Ephemeral Components
//!
//! Tests focused on ephemeral component behavior within the system scheduler,
//! including cross-system communication and lifecycle management.

use bemudjo_ecs::{Component, SequentialSystemScheduler, System, World};
use std::cell::RefCell;
use std::rc::Rc;

// Test Components
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

#[derive(Clone, Debug, PartialEq)]
struct DamageEvent {
    amount: u32,
    source: String,
}
impl Component for DamageEvent {}

#[derive(Clone, Debug, PartialEq)]
struct HealEvent {
    amount: u32,
}
impl Component for HealEvent {}

#[derive(Clone, Debug, PartialEq)]
struct MovementEvent {
    dx: f32,
    dy: f32,
}
impl Component for MovementEvent {}

#[derive(Clone, Debug, PartialEq)]
struct DeathEvent {
    cause: String,
}
impl Component for DeathEvent {}

// Test Systems

/// System that creates damage events based on proximity
struct CombatSystem;
impl System for CombatSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for &entity in &entities {
            if let Some(pos) = world.get_component::<Position>(entity) {
                if let Some(health) = world.get_component::<Health>(entity) {
                    // Create damage event for entities at specific positions
                    if pos.x > 50.0 && health.current > 0 {
                        world
                            .add_ephemeral_component(
                                entity,
                                DamageEvent {
                                    amount: 10,
                                    source: "combat".to_string(),
                                },
                            )
                            .ok();
                    }
                }
            }
        }
    }
}

/// System that processes damage events and applies them to health
struct DamageProcessingSystem;
impl System for DamageProcessingSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();
        for &entity in &entities {
            if world.has_ephemeral_component::<DamageEvent>(entity) {
                if let (Some(damage), Some(health)) = (
                    world.get_ephemeral_component::<DamageEvent>(entity),
                    world.get_component::<Health>(entity),
                ) {
                    let damage_amount = damage.amount;
                    let damage_source = damage.source.clone();
                    let new_health = health.current.saturating_sub(damage_amount);

                    // Update health
                    world.replace_component(
                        entity,
                        Health {
                            current: new_health,
                            max: health.max,
                        },
                    );

                    // Create death event if health reaches 0
                    if new_health == 0 {
                        world
                            .add_ephemeral_component(
                                entity,
                                DeathEvent {
                                    cause: damage_source,
                                },
                            )
                            .ok();
                    }
                }
            }
        }
    }
}

/// System that processes healing events
struct HealingSystem;
impl System for HealingSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for &entity in &entities {
            if world.has_ephemeral_component::<HealEvent>(entity) {
                if let Some(heal) = world.get_ephemeral_component::<HealEvent>(entity) {
                    if let Some(health) = world.get_component::<Health>(entity) {
                        let new_health = (health.current + heal.amount).min(health.max);

                        world.replace_component(
                            entity,
                            Health {
                                current: new_health,
                                max: health.max,
                            },
                        );
                    }
                }
            }
        }
    }
}

/// System that processes movement events
struct MovementSystem;
impl System for MovementSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for &entity in &entities {
            if world.has_ephemeral_component::<MovementEvent>(entity) {
                if let Some(movement) = world.get_ephemeral_component::<MovementEvent>(entity) {
                    if let Some(pos) = world.get_component::<Position>(entity) {
                        world.replace_component(
                            entity,
                            Position {
                                x: pos.x + movement.dx,
                                y: pos.y + movement.dy,
                            },
                        );
                    }
                }
            }
        }
    }
}

/// System that logs events for testing
struct LoggingSystem {
    events: Rc<RefCell<Vec<String>>>,
}

impl LoggingSystem {
    fn new(events: Rc<RefCell<Vec<String>>>) -> Self {
        Self { events }
    }
}

impl System for LoggingSystem {
    fn after_run(&self, world: &World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for &entity in &entities {
            if world.has_ephemeral_component::<DeathEvent>(entity) {
                if let Some(death) = world.get_ephemeral_component::<DeathEvent>(entity) {
                    self.events
                        .borrow_mut()
                        .push(format!("Entity died from {}", death.cause));
                }
            }
        }
    }
}

#[test]
fn test_ephemeral_components_cross_system_communication() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Create test entities
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    // Set up entities with different positions
    world
        .add_component(entity1, Position { x: 60.0, y: 10.0 })
        .unwrap(); // Will take damage
    world
        .add_component(entity2, Position { x: 30.0, y: 20.0 })
        .unwrap(); // Won't take damage
    world
        .add_component(entity3, Position { x: 70.0, y: 30.0 })
        .unwrap(); // Will take damage

    world
        .add_component(
            entity1,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            entity2,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            entity3,
            Health {
                current: 5,
                max: 100,
            },
        )
        .unwrap(); // Low health

    // Set up event logging
    let events = Rc::new(RefCell::new(Vec::new()));

    // Add systems in order
    scheduler.add_system(CombatSystem).unwrap();
    scheduler.add_system(DamageProcessingSystem).unwrap();
    scheduler.add_system(HealingSystem).unwrap();
    scheduler
        .add_system(LoggingSystem::new(events.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Before tick - no ephemeral components
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity1));
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity2));
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity3));

    // Run one tick
    scheduler.run_tick(&mut world);

    // After tick - ephemeral components should be cleaned up
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity1));
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity2));
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity3));
    assert!(!world.has_ephemeral_component::<DeathEvent>(entity3));

    // Check health changes
    assert_eq!(world.get_component::<Health>(entity1).unwrap().current, 90); // Took damage
    assert_eq!(world.get_component::<Health>(entity2).unwrap().current, 100); // No damage
    assert_eq!(world.get_component::<Health>(entity3).unwrap().current, 0); // Died

    // Check logged events
    let logged_events = events.borrow();
    assert_eq!(logged_events.len(), 1);
    assert!(logged_events[0].contains("died from combat"));
}

#[test]
fn test_ephemeral_components_persist_across_system_phases() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let entity = world.spawn_entity();
    world
        .add_component(entity, Position { x: 0.0, y: 0.0 })
        .unwrap();

    let events = Rc::new(RefCell::new(Vec::new()));

    // System that creates movement events in before_run
    struct MovementCreatorSystem;
    impl System for MovementCreatorSystem {
        fn run(&self, world: &mut World) {
            for entity in world.entities().cloned().collect::<Vec<_>>() {
                if world.has_component::<Position>(entity) {
                    world
                        .add_ephemeral_component(entity, MovementEvent { dx: 5.0, dy: 3.0 })
                        .ok();
                }
            }
        }
    }

    // System that processes movement in run phase
    struct MovementProcessorSystem {
        events: Rc<RefCell<Vec<String>>>,
    }

    impl MovementProcessorSystem {
        fn new(events: Rc<RefCell<Vec<String>>>) -> Self {
            Self { events }
        }
    }

    impl System for MovementProcessorSystem {
        fn run(&self, world: &mut World) {
            for entity in world.entities().cloned().collect::<Vec<_>>() {
                if world.has_ephemeral_component::<MovementEvent>(entity) {
                    self.events
                        .borrow_mut()
                        .push("Movement event found in run phase".to_string());

                    if let Some(movement) = world.get_ephemeral_component::<MovementEvent>(entity) {
                        if let Some(pos) = world.get_component::<Position>(entity) {
                            world.replace_component(
                                entity,
                                Position {
                                    x: pos.x + movement.dx,
                                    y: pos.y + movement.dy,
                                },
                            );
                        }
                    }
                }
            }
        }

        fn after_run(&self, world: &World) {
            for entity in world.entities().cloned().collect::<Vec<_>>() {
                if world.has_ephemeral_component::<MovementEvent>(entity) {
                    self.events
                        .borrow_mut()
                        .push("Movement event still exists in after_run phase".to_string());
                }
            }
        }
    }

    scheduler.add_system(MovementCreatorSystem).unwrap();
    scheduler
        .add_system(MovementProcessorSystem::new(events.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Run tick
    scheduler.run_tick(&mut world);

    // Check that position was updated
    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 5.0);
    assert_eq!(pos.y, 3.0);

    // Check that movement event persisted across all system phases
    let logged_events = events.borrow();
    assert_eq!(logged_events.len(), 2);
    assert_eq!(logged_events[0], "Movement event found in run phase");
    assert_eq!(
        logged_events[1],
        "Movement event still exists in after_run phase"
    );

    // After tick, ephemeral components should be gone
    assert!(!world.has_ephemeral_component::<MovementEvent>(entity));
}

#[test]
fn test_ephemeral_components_with_multiple_ticks() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let entity = world.spawn_entity();
    world
        .add_component(
            entity,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();

    // System that creates heal events every tick
    struct HealGeneratorSystem;
    impl System for HealGeneratorSystem {
        fn run(&self, world: &mut World) {
            for entity in world.entities().cloned().collect::<Vec<_>>() {
                if world.has_component::<Health>(entity) {
                    world
                        .add_ephemeral_component(entity, HealEvent { amount: 10 })
                        .ok();
                }
            }
        }
    }

    scheduler.add_system(HealGeneratorSystem).unwrap();
    scheduler.add_system(HealingSystem).unwrap();
    scheduler.build().unwrap();

    // Damage the entity first
    world.replace_component(
        entity,
        Health {
            current: 60,
            max: 100,
        },
    );

    // Run multiple ticks
    for i in 0..3 {
        // Before tick - no ephemeral components (after first tick)
        if i > 0 {
            assert!(!world.has_ephemeral_component::<HealEvent>(entity));
        }

        scheduler.run_tick(&mut world);

        // After tick - ephemeral components cleaned up
        assert!(!world.has_ephemeral_component::<HealEvent>(entity));

        // Health should increase each tick
        let health = world.get_component::<Health>(entity).unwrap();
        assert_eq!(health.current, 70 + (i * 10));
    }
}

#[test]
fn test_ephemeral_components_with_entity_deletion_in_systems() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Create entities
    let entity1 = world.spawn_entity();
    let entity2 = world.spawn_entity();
    let entity3 = world.spawn_entity();

    world
        .add_component(
            entity1,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            entity2,
            Health {
                current: 5,
                max: 100,
            },
        )
        .unwrap(); // Will die
    world
        .add_component(
            entity3,
            Health {
                current: 50,
                max: 100,
            },
        )
        .unwrap();

    // System that creates damage events
    struct DamageGeneratorSystem;
    impl System for DamageGeneratorSystem {
        fn run(&self, world: &mut World) {
            for entity in world.entities().cloned().collect::<Vec<_>>() {
                if world.has_component::<Health>(entity) {
                    world
                        .add_ephemeral_component(
                            entity,
                            DamageEvent {
                                amount: 10,
                                source: "poison".to_string(),
                            },
                        )
                        .ok();
                }
            }
        }
    }

    // System that processes damage and deletes dead entities
    struct DeadlyDamageSystem;
    impl System for DeadlyDamageSystem {
        fn run(&self, world: &mut World) {
            let entities: Vec<_> = world.entities().cloned().collect();
            for &entity in &entities {
                if world.has_ephemeral_component::<DamageEvent>(entity) {
                    if let (Some(damage), Some(health)) = (
                        world.get_ephemeral_component::<DamageEvent>(entity),
                        world.get_component::<Health>(entity),
                    ) {
                        let damage_amount = damage.amount;
                        let new_health = health.current.saturating_sub(damage_amount);

                        if new_health == 0 {
                            world.delete_entity(entity);
                        } else {
                            world.replace_component(
                                entity,
                                Health {
                                    current: new_health,
                                    max: health.max,
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    scheduler.add_system(DamageGeneratorSystem).unwrap();
    scheduler.add_system(DeadlyDamageSystem).unwrap();
    scheduler.build().unwrap();

    // Run tick
    scheduler.run_tick(&mut world);

    // entity2 should be deleted, others should survive with reduced health
    assert!(world.entities().any(|&e| e == entity1));
    assert!(!world.entities().any(|&e| e == entity2)); // Deleted
    assert!(world.entities().any(|&e| e == entity3));

    // Check health of surviving entities
    assert_eq!(world.get_component::<Health>(entity1).unwrap().current, 90);
    assert_eq!(world.get_component::<Health>(entity3).unwrap().current, 40);

    // No ephemeral components should remain
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity1));
    assert!(!world.has_ephemeral_component::<DamageEvent>(entity3));
}

#[test]
fn test_ephemeral_components_performance_with_many_entities() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Create many entities
    let mut entities = Vec::new();
    for i in 0..1000 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Health {
                    current: 100,
                    max: 100,
                },
            )
            .unwrap();
        world
            .add_component(
                entity,
                Position {
                    x: (i % 50) as f32,
                    y: (i / 50) as f32,
                },
            )
            .unwrap();
        entities.push(entity);
    }

    // System that creates ephemeral components for all entities
    struct MassEventSystem;
    impl System for MassEventSystem {
        fn run(&self, world: &mut World) {
            for entity in world.entities().cloned().collect::<Vec<_>>() {
                // Add different types of ephemeral components
                world
                    .add_ephemeral_component(
                        entity,
                        DamageEvent {
                            amount: 1,
                            source: "aoe".to_string(),
                        },
                    )
                    .ok();

                world
                    .add_ephemeral_component(entity, MovementEvent { dx: 0.1, dy: 0.1 })
                    .ok();
            }
        }
    }

    scheduler.add_system(MassEventSystem).unwrap();
    scheduler.add_system(DamageProcessingSystem).unwrap();
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.build().unwrap();

    // Run multiple ticks to test performance and correctness
    for _ in 0..5 {
        scheduler.run_tick(&mut world);

        // Verify all ephemeral components are cleaned up
        for &entity in &entities {
            assert!(!world.has_ephemeral_component::<DamageEvent>(entity));
            assert!(!world.has_ephemeral_component::<MovementEvent>(entity));
        }
    }

    // Verify entities moved and took damage
    for &entity in entities.iter().take(10) {
        // Check first 10
        let health = world.get_component::<Health>(entity).unwrap();
        assert_eq!(health.current, 95); // 5 ticks * 1 damage each

        let pos = world.get_component::<Position>(entity).unwrap();
        assert!(pos.x > 0.0); // Should have moved
        assert!(pos.y > 0.0);
    }
}
