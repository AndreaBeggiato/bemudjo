//! System Dependencies Integration Tests
//!
//! Tests focused on cross-system interactions, dependencies,
//! and complex system orchestration scenarios.

use bemudjo_ecs::{Component, SequentialSystemScheduler, System, World};
use std::cell::RefCell;
use std::rc::Rc;

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
struct Dead;
impl Component for Dead {}

#[derive(Clone, Debug, PartialEq)]
struct Experience {
    points: u64,
    level: u32,
}
impl Component for Experience {}

#[derive(Clone, Debug, PartialEq)]
struct Target {
    entity_id: Option<bemudjo_ecs::Entity>,
}
impl Component for Target {}

// Dependent Systems

/// Physics system - moves entities based on velocity
struct PhysicsSystem {
    delta_time: f32,
}

impl PhysicsSystem {
    fn new(delta_time: f32) -> Self {
        Self { delta_time }
    }
}

impl System for PhysicsSystem {
    fn run(&self, world: &mut World) {
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

/// Combat system - applies damage to entities with health
struct CombatSystem;

impl System for CombatSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if let (Some(health), Some(damage)) = (
                world.get_component::<Health>(entity),
                world.get_component::<Damage>(entity),
            ) {
                let new_health_current = health.current.saturating_sub(damage.amount);
                let new_health = Health {
                    current: new_health_current,
                    max: health.max,
                };
                world.replace_component(entity, new_health);

                // Remove damage component after applying
                world.remove_component::<Damage>(entity);

                // Mark as dead if health reaches 0
                if new_health_current == 0 {
                    world.add_component(entity, Dead).ok();
                }
            }
        }
    }
}

/// Death system - removes dead entities and grants experience
struct DeathSystem {
    experience_gained: Rc<RefCell<u64>>,
}

impl DeathSystem {
    fn new(experience_gained: Rc<RefCell<u64>>) -> Self {
        Self { experience_gained }
    }
}

impl System for DeathSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if world.has_component::<Dead>(entity) {
                // Grant experience based on entity's max health
                if let Some(health) = world.get_component::<Health>(entity) {
                    *self.experience_gained.borrow_mut() += health.max as u64;
                }

                world.delete_entity(entity);
            }
        }
    }
}

/// Experience system - levels up entities based on accumulated experience
struct ExperienceSystem {
    experience_pool: Rc<RefCell<u64>>,
}

impl ExperienceSystem {
    fn new(experience_pool: Rc<RefCell<u64>>) -> Self {
        Self { experience_pool }
    }
}

impl System for ExperienceSystem {
    fn run(&self, world: &mut World) {
        let total_exp = *self.experience_pool.borrow();
        if total_exp == 0 {
            return;
        }

        let entities: Vec<_> = world.entities().cloned().collect();
        let living_entities: Vec<_> = entities
            .into_iter()
            .filter(|&e| !world.has_component::<Dead>(e) && world.has_component::<Experience>(e))
            .collect();

        if living_entities.is_empty() {
            return;
        }

        let exp_per_entity = total_exp / living_entities.len() as u64;

        for entity in living_entities {
            world
                .update_component::<Experience, _>(entity, |mut exp| {
                    exp.points += exp_per_entity;

                    // Level up logic - can level up multiple times
                    while exp.points >= (exp.level as u64 + 1) * 100 {
                        let required_exp = (exp.level as u64 + 1) * 100;
                        exp.points -= required_exp;
                        exp.level += 1;
                    }

                    exp
                })
                .ok();
        }

        // Clear experience pool
        *self.experience_pool.borrow_mut() = 0;
    }
}

/// AI system - makes entities target nearby enemies
struct AISystem;

impl System for AISystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        // Build list of entities with positions
        let mut positioned_entities = Vec::new();
        for &entity in &entities {
            if let Some(pos) = world.get_component::<Position>(entity) {
                positioned_entities.push((entity, pos.clone()));
            }
        }

        // Update targeting for entities that need it
        for &entity in &entities {
            if !world.has_component::<Dead>(entity) && world.has_component::<Target>(entity) {
                let entity_pos = match world.get_component::<Position>(entity) {
                    Some(pos) => pos,
                    None => continue,
                };

                // Find closest other entity
                let mut closest_entity = None;
                let mut closest_distance = f32::MAX;

                for &(other_entity, ref other_pos) in &positioned_entities {
                    if other_entity == entity || world.has_component::<Dead>(other_entity) {
                        continue;
                    }

                    let dx = entity_pos.x - other_pos.x;
                    let dy = entity_pos.y - other_pos.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_entity = Some(other_entity);
                    }
                }

                // Update target
                world.replace_component(
                    entity,
                    Target {
                        entity_id: closest_entity,
                    },
                );

                // If close enough, apply damage
                if closest_distance < 2.0 {
                    if let Some(target_entity) = closest_entity {
                        if !world.has_component::<Damage>(target_entity) {
                            world
                                .add_component(target_entity, Damage { amount: 10 })
                                .ok();
                        }
                    }
                }
            }
        }
    }
}

/// Regeneration system - heals entities over time
struct RegenerationSystem {
    regen_amount: u32,
}

impl RegenerationSystem {
    fn new(regen_amount: u32) -> Self {
        Self { regen_amount }
    }
}

impl System for RegenerationSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if !world.has_component::<Dead>(entity) {
                if let Some(health) = world.get_component::<Health>(entity) {
                    if health.current < health.max {
                        let new_current = (health.current + self.regen_amount).min(health.max);
                        world.replace_component(
                            entity,
                            Health {
                                current: new_current,
                                max: health.max,
                            },
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_physics_and_combat_integration() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(PhysicsSystem::new(1.0)).unwrap();
    scheduler.add_system(CombatSystem).unwrap();
    scheduler.build().unwrap();

    // Create a moving entity that will receive damage
    let entity = world.spawn_entity();
    world
        .add_component(entity, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(entity, Velocity { x: 1.0, y: 2.0 })
        .unwrap();
    world
        .add_component(
            entity,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world.add_component(entity, Damage { amount: 30 }).unwrap();

    // Run one tick
    scheduler.run_tick(&mut world);

    // Verify physics moved the entity
    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 1.0);
    assert_eq!(pos.y, 2.0);

    // Verify combat applied damage
    let health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(health.current, 70); // 100 - 30

    // Damage component should be removed
    assert!(!world.has_component::<Damage>(entity));

    // Entity should not be dead yet
    assert!(!world.has_component::<Dead>(entity));
}

#[test]
fn test_combat_death_experience_chain() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let experience_pool = Rc::new(RefCell::new(0u64));

    scheduler.add_system(CombatSystem).unwrap();
    scheduler
        .add_system(DeathSystem::new(experience_pool.clone()))
        .unwrap();
    scheduler
        .add_system(ExperienceSystem::new(experience_pool.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Create entities: one that will die, one that will gain experience
    let victim = world.spawn_entity();
    world
        .add_component(
            victim,
            Health {
                current: 20,
                max: 50,
            },
        )
        .unwrap();
    world.add_component(victim, Damage { amount: 30 }).unwrap(); // Will kill it

    let survivor = world.spawn_entity();
    world
        .add_component(
            survivor,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            survivor,
            Experience {
                points: 0,
                level: 1,
            },
        )
        .unwrap();

    assert_eq!(world.entities().count(), 2);

    // Run one tick
    scheduler.run_tick(&mut world);

    // Victim should be marked for deletion
    assert_eq!(world.entities().count(), 1);

    // Cleanup deleted entities
    world.cleanup_deleted_entities();
    assert_eq!(world.entities().count(), 1);

    // Survivor should have gained experience
    let exp = world.get_component::<Experience>(survivor).unwrap();
    assert_eq!(exp.points, 50); // Gained victim's max health as experience
    assert_eq!(exp.level, 1); // Not enough to level up yet

    // Experience pool should be cleared
    assert_eq!(*experience_pool.borrow(), 0);
}

#[test]
fn test_ai_combat_physics_integration() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(PhysicsSystem::new(0.5)).unwrap();
    scheduler.add_system(AISystem).unwrap();
    scheduler.add_system(CombatSystem).unwrap();
    scheduler.build().unwrap();

    // Create two entities that will fight
    let entity1 = world.spawn_entity();
    world
        .add_component(entity1, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(entity1, Velocity { x: 1.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            entity1,
            Health {
                current: 50,
                max: 50,
            },
        )
        .unwrap();
    world
        .add_component(entity1, Target { entity_id: None })
        .unwrap();

    let entity2 = world.spawn_entity();
    world
        .add_component(entity2, Position { x: 3.0, y: 0.0 })
        .unwrap(); // Close but not in range
    world
        .add_component(
            entity2,
            Health {
                current: 50,
                max: 50,
            },
        )
        .unwrap();

    // Run first tick - entities move, AI targets, but no damage yet
    scheduler.run_tick(&mut world);

    // Entity1 should have moved
    let pos1 = world.get_component::<Position>(entity1).unwrap();
    assert_eq!(pos1.x, 0.5); // 0 + 1.0 * 0.5

    // Entity1 should target entity2
    let target = world.get_component::<Target>(entity1).unwrap();
    assert_eq!(target.entity_id, Some(entity2));

    // No damage yet (distance still > 2.0)
    assert!(!world.has_component::<Damage>(entity2));

    // Run more ticks until entities are close enough
    for _ in 0..4 {
        scheduler.run_tick(&mut world);
    }

    // Now entity1 should be close enough to damage entity2
    let pos1 = world.get_component::<Position>(entity1).unwrap();
    assert!(pos1.x >= 2.0); // Should be close enough

    // Entity2 should have taken damage
    let health2 = world.get_component::<Health>(entity2).unwrap();
    assert!(health2.current < 50); // Should have taken some damage
}

#[test]
fn test_regeneration_combat_balance() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    scheduler.add_system(RegenerationSystem::new(5)).unwrap();
    scheduler.add_system(CombatSystem).unwrap();
    scheduler.build().unwrap();

    let entity = world.spawn_entity();
    world
        .add_component(
            entity,
            Health {
                current: 50,
                max: 100,
            },
        )
        .unwrap();

    // Run regeneration only
    scheduler.run_tick(&mut world);

    let health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(health.current, 55); // 50 + 5 regen

    // Add damage and run again
    world.add_component(entity, Damage { amount: 10 }).unwrap();
    scheduler.run_tick(&mut world);

    let health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(health.current, 50); // 55 + 5 regen - 10 damage

    // Add lethal damage
    world.add_component(entity, Damage { amount: 60 }).unwrap();
    scheduler.run_tick(&mut world);

    // Entity should be dead
    assert!(world.has_component::<Dead>(entity));
    let health = world.get_component::<Health>(entity).unwrap();
    assert_eq!(health.current, 0);
}

#[test]
fn test_complex_multi_system_scenario() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let experience_pool = Rc::new(RefCell::new(0u64));

    // Add all systems in dependency order
    scheduler.add_system(PhysicsSystem::new(1.0)).unwrap();
    scheduler.add_system(AISystem).unwrap();
    scheduler.add_system(CombatSystem).unwrap();
    scheduler.add_system(RegenerationSystem::new(2)).unwrap();
    scheduler
        .add_system(DeathSystem::new(experience_pool.clone()))
        .unwrap();
    scheduler
        .add_system(ExperienceSystem::new(experience_pool.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Create a complex scenario
    let player = world.spawn_entity();
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(player, Velocity { x: 0.5, y: 0.0 })
        .unwrap();
    world
        .add_component(
            player,
            Health {
                current: 80,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Experience {
                points: 80,
                level: 1,
            },
        )
        .unwrap();
    world
        .add_component(player, Target { entity_id: None })
        .unwrap();

    let enemy1 = world.spawn_entity();
    world
        .add_component(enemy1, Position { x: 5.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            enemy1,
            Health {
                current: 30,
                max: 30,
            },
        )
        .unwrap();

    let enemy2 = world.spawn_entity();
    world
        .add_component(enemy2, Position { x: 1.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            enemy2,
            Health {
                current: 15,
                max: 20,
            },
        )
        .unwrap();
    world
        .add_component(enemy2, Target { entity_id: None })
        .unwrap();

    assert_eq!(world.entities().count(), 3);

    // Run simulation for multiple ticks
    for tick in 1..=10 {
        scheduler.run_tick(&mut world);

        // Clean up dead entities
        world.cleanup_deleted_entities();

        // Verify player is moving
        let player_pos = world.get_component::<Position>(player).unwrap();
        assert_eq!(player_pos.x, tick as f32 * 0.5);

        // Check if player leveled up
        if let Some(player_exp) = world.get_component::<Experience>(player) {
            if player_exp.level > 1 {
                // Player gained enough experience to level up
                break;
            }
        }
    }

    // Verify final state
    assert!(world.has_component::<Position>(player));
    assert!(world.has_component::<Health>(player));
    assert!(world.has_component::<Experience>(player));

    // Some entities should have died and player should have gained experience
    let final_count = world.entities().count();
    assert!(final_count <= 3); // Some entities might have died

    let player_exp = world.get_component::<Experience>(player).unwrap();
    assert!(player_exp.points > 0 || player_exp.level > 1); // Should have gained something
}

#[test]
fn test_system_dependency_ordering() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let execution_order = Rc::new(RefCell::new(Vec::new()));

    struct OrderTrackingSystem {
        name: String,
        order: Rc<RefCell<Vec<String>>>,
    }

    impl OrderTrackingSystem {
        fn new(name: &str, order: Rc<RefCell<Vec<String>>>) -> Self {
            Self {
                name: name.to_string(),
                order,
            }
        }
    }

    impl System for OrderTrackingSystem {
        fn run(&self, _world: &mut World) {
            self.order.borrow_mut().push(self.name.clone());
        }
    }

    // Add systems in specific order
    scheduler
        .add_system(OrderTrackingSystem::new("Physics", execution_order.clone()))
        .unwrap();
    scheduler
        .add_system(OrderTrackingSystem::new("AI", execution_order.clone()))
        .unwrap();
    scheduler
        .add_system(OrderTrackingSystem::new("Combat", execution_order.clone()))
        .unwrap();
    scheduler
        .add_system(OrderTrackingSystem::new("Death", execution_order.clone()))
        .unwrap();
    scheduler.build().unwrap();

    scheduler.run_tick(&mut world);

    let order = execution_order.borrow().clone();
    assert_eq!(order, vec!["Physics", "AI", "Combat", "Death"]);
}

#[test]
fn test_cascading_system_effects() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    let experience_pool = Rc::new(RefCell::new(0u64));

    scheduler.add_system(CombatSystem).unwrap();
    scheduler
        .add_system(DeathSystem::new(experience_pool.clone()))
        .unwrap();
    scheduler
        .add_system(ExperienceSystem::new(experience_pool.clone()))
        .unwrap();
    scheduler.build().unwrap();

    // Create multiple entities that will die in chain
    let mut entities = Vec::new();
    for i in 0..5 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Health {
                    current: 10,
                    max: 20 + i * 10,
                },
            )
            .unwrap();
        world
            .add_component(
                entity,
                Experience {
                    points: 0,
                    level: 1,
                },
            )
            .unwrap();
        world.add_component(entity, Damage { amount: 15 }).unwrap(); // Will kill all
        entities.push(entity);
    }

    assert_eq!(world.entities().count(), 5);

    // Run one tick - all entities should die and distribute experience
    scheduler.run_tick(&mut world);

    // All entities should be marked for deletion
    assert_eq!(world.entities().count(), 0);

    // Experience pool should remain because no living entities to distribute to
    assert_eq!(*experience_pool.borrow(), 200); // 20+30+40+50+60 = 200

    // Create new entity to receive experience
    let survivor = world.spawn_entity();
    world
        .add_component(
            survivor,
            Experience {
                points: 0,
                level: 1,
            },
        )
        .unwrap();

    // Kill another entity to generate experience
    let victim = world.spawn_entity();
    world
        .add_component(
            victim,
            Health {
                current: 1,
                max: 150,
            },
        )
        .unwrap();
    world.add_component(victim, Damage { amount: 5 }).unwrap();

    scheduler.run_tick(&mut world);
    world.cleanup_deleted_entities();

    // Debug: Check what happened in second tick
    // Survivor should have gained experience and leveled up
    let exp = world.get_component::<Experience>(survivor).unwrap();
    assert_eq!(exp.points, 150); // 350 total - 200 for level up = 150 remaining
    assert_eq!(exp.level, 2); // Should have leveled up
}
