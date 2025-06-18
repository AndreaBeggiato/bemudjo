use crate::World;

/// A trait defining the interface for systems that process entities.
///
/// Systems operate in three distinct phases to enable safe parallel execution
/// and clear separation of concerns:
///
/// 1. `before_run` - Read-only preparation phase
/// 2. `run` - Main logic with world mutations
/// 3. `after_run` - Read-only cleanup/output phase
///
/// # Example
/// ```
/// use bemudjo_ecs::{System, World, Component};
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Velocity { x: f32, y: f32 }
/// impl Component for Velocity {}
///
/// struct MovementSystem;
///
/// impl System for MovementSystem {
///     fn run(&self, world: &mut World) {
///         // Process all entities with both Position and Velocity
///         for entity in world.entities().cloned().collect::<Vec<_>>() {
///             if let (Some(pos), Some(vel)) = (
///                 world.get_component::<Position>(entity),
///                 world.get_component::<Velocity>(entity)
///             ) {
///                 let new_pos = Position {
///                     x: pos.x + vel.x,
///                     y: pos.y + vel.y,
///                 };
///                 world.replace_component(entity, new_pos);
///             }
///         }
///     }
/// }
/// ```
pub trait System {
    /// Called before the main execution phase.
    ///
    /// Use this for read-only preparation work such as:
    /// - Querying world state
    /// - Preparing data structures
    /// - Input validation
    ///
    /// This phase is safe for parallel execution since it only reads world state.
    fn before_run(&self, _world: &World) {}

    /// Main system execution phase with mutable world access.
    ///
    /// Use this for:
    /// - Modifying components
    /// - Spawning/despawning entities
    /// - Core application logic
    ///
    /// This phase runs sequentially to ensure data safety.
    fn run(&self, _world: &mut World) {}

    /// Called after the main execution phase.
    ///
    /// Use this for read-only cleanup work such as:
    /// - Rendering
    /// - Logging
    /// - Network updates
    /// - Statistics collection
    ///
    /// This phase is safe for parallel execution since it only reads world state.
    fn after_run(&self, _world: &World) {}
}

/// A simple system scheduler that executes systems in registration order.
///
/// The scheduler runs all systems through three distinct phases:
/// 1. All systems' `before_run` methods (preparation)
/// 2. All systems' `run` methods (main logic)
/// 3. All systems' `after_run` methods (cleanup/output)
///
/// # Execution Order
/// Systems execute in the order they were added with `add_system()`.
/// This makes the execution predictable and deterministic, which is
/// crucial for applications that require consistent behavior.
///
/// # Example Usage
/// ```
/// use bemudjo_ecs::{SystemScheduler, System, World, Component};
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Health { value: u32 }
/// impl Component for Health {}
///
/// struct DamageSystem;
/// impl System for DamageSystem {
///     fn run(&self, world: &mut World) {
///         // Process damage logic
///         println!("Processing damage...");
///     }
/// }
///
/// struct RenderSystem;
/// impl System for RenderSystem {
///     fn after_run(&self, world: &World) {
///         // Render entities
///         println!("Rendering frame...");
///     }
/// }
///
/// // Setup
/// let mut world = World::new();
/// let mut scheduler = SystemScheduler::new();
///
/// // Order matters! Damage must be processed before rendering
/// scheduler.add_system(DamageSystem);
/// scheduler.add_system(RenderSystem);
///
/// // Application loop
/// loop {
///     scheduler.run_tick(&mut world);
///     // Sleep until next tick...
///     break; // For example purposes
/// }
/// ```
///
/// # Performance Characteristics
/// - Low overhead: Simple iteration through systems
/// - Predictable timing: No complex dependency resolution
/// - Cache-friendly: Sequential execution pattern
/// - Deterministic: Same order every time
pub struct SystemScheduler {
    systems: Vec<Box<dyn System>>,
}

impl SystemScheduler {
    /// Creates a new empty system scheduler.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::SystemScheduler;
    ///
    /// let scheduler = SystemScheduler::new();
    /// assert_eq!(scheduler.system_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Adds a system to the scheduler.
    ///
    /// Systems will execute in the order they are added. Each system
    /// must implement the `System` trait.
    ///
    /// # Parameters
    /// * `system` - Any type implementing the `System` trait
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{SystemScheduler, System, World};
    ///
    /// struct MySystem;
    /// impl System for MySystem {
    ///     fn run(&self, world: &mut World) {
    ///         println!("MySystem is running!");
    ///     }
    /// }
    ///
    /// let mut scheduler = SystemScheduler::new();
    /// scheduler.add_system(MySystem);
    /// assert_eq!(scheduler.system_count(), 1);
    /// ```
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// Returns the number of systems currently registered.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{SystemScheduler, System, World};
    ///
    /// struct System1;
    /// impl System for System1 {}
    ///
    /// struct System2;
    /// impl System for System2 {}
    ///
    /// let mut scheduler = SystemScheduler::new();
    /// assert_eq!(scheduler.system_count(), 0);
    ///
    /// scheduler.add_system(System1);
    /// assert_eq!(scheduler.system_count(), 1);
    ///
    /// scheduler.add_system(System2);
    /// assert_eq!(scheduler.system_count(), 2);
    /// ```
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Executes one complete tick of all registered systems.
    ///
    /// This method runs all systems through their four phases:
    /// 1. **Preparation Phase**: All systems' `before_run` methods
    /// 2. **Execution Phase**: All systems' `run` methods
    /// 3. **Cleanup Phase**: All systems' `after_run` methods
    /// 4. **Entity Cleanup**: Automatic cleanup of deleted entities
    ///
    /// Systems execute in registration order within each phase. After all systems
    /// complete, deleted entities are automatically cleaned up to maintain optimal
    /// performance and prevent memory leaks.
    ///
    /// # Parameters
    /// * `world` - Mutable reference to the ECS world
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{SystemScheduler, System, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Counter { value: u32 }
    /// impl Component for Counter {}
    ///
    /// struct IncrementSystem;
    /// impl System for IncrementSystem {
    ///     fn run(&self, world: &mut World) {
    ///         for entity in world.entities().cloned().collect::<Vec<_>>() {
    ///             if let Some(counter) = world.get_component::<Counter>(entity) {
    ///                 let new_counter = Counter { value: counter.value + 1 };
    ///                 world.replace_component(entity, new_counter);
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    /// world.add_component(entity, Counter { value: 0 }).unwrap();
    ///
    /// let mut scheduler = SystemScheduler::new();
    /// scheduler.add_system(IncrementSystem);
    ///
    /// // Run one tick
    /// scheduler.run_tick(&mut world);
    ///
    /// // Counter should be incremented
    /// let counter = world.get_component::<Counter>(entity).unwrap();
    /// assert_eq!(counter.value, 1);
    /// ```
    ///
    /// # Typical Application Loop
    /// ```
    /// use bemudjo_ecs::{SystemScheduler, World};
    /// use std::time::{Duration, Instant};
    ///
    /// let mut world = World::new();
    /// let scheduler = SystemScheduler::new();
    ///
    /// // Application runs at fixed timestep (e.g., 60 FPS or 10 TPS)
    /// let tick_duration = Duration::from_millis(100); // 10 TPS
    ///
    /// for _tick in 0..5 { // Run 5 ticks for example
    ///     let start = Instant::now();
    ///
    ///     // Execute all systems for this tick
    ///     scheduler.run_tick(&mut world);
    ///
    ///     // Sleep until next tick (timing control)
    ///     let elapsed = start.elapsed();
    ///     if elapsed < tick_duration {
    ///         std::thread::sleep(tick_duration - elapsed);
    ///     }
    /// }
    /// ```
    pub fn run_tick(&self, world: &mut World) {
        // Phase 1: Preparation - All before_run methods
        // This phase is read-only and could be parallelized in the future
        for system in &self.systems {
            system.before_run(world);
        }

        // Phase 2: Execution - All run methods
        // This phase modifies the world and must be sequential for safety
        for system in &self.systems {
            system.run(world);
        }

        // Phase 3: Cleanup - All after_run methods
        // This phase is read-only and could be parallelized in the future
        for system in &self.systems {
            system.after_run(world);
        }

        // Phase 4: Entity cleanup - Remove component data for deleted entities
        // This ensures clean state for the next tick and prevents memory leaks
        world.cleanup_deleted_entities();
    }
}

impl Default for SystemScheduler {
    /// Creates a new empty system scheduler using the default constructor.
    ///
    /// This is equivalent to calling `SystemScheduler::new()`.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::SystemScheduler;
    ///
    /// let scheduler1 = SystemScheduler::new();
    /// let scheduler2 = SystemScheduler::default();
    ///
    /// assert_eq!(scheduler1.system_count(), scheduler2.system_count());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Component, World};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, PartialEq)]
    struct Counter {
        count: u32,
    }
    impl Component for Counter {}

    struct TestSystem {
        name: String,
        execution_log: Arc<Mutex<Vec<String>>>,
    }

    impl TestSystem {
        fn new(name: &str, log: Arc<Mutex<Vec<String>>>) -> Self {
            Self {
                name: name.to_string(),
                execution_log: log,
            }
        }
    }

    impl System for TestSystem {
        fn before_run(&self, _world: &World) {
            self.execution_log
                .lock()
                .unwrap()
                .push(format!("{}_before", self.name));
        }

        fn run(&self, _world: &mut World) {
            self.execution_log
                .lock()
                .unwrap()
                .push(format!("{}_run", self.name));
        }

        fn after_run(&self, _world: &World) {
            self.execution_log
                .lock()
                .unwrap()
                .push(format!("{}_after", self.name));
        }
    }

    #[test]
    fn test_system_scheduler_new() {
        let scheduler = SystemScheduler::new();
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn test_system_scheduler_default() {
        let scheduler = SystemScheduler::default();
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn test_add_system() {
        let mut scheduler = SystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        scheduler.add_system(TestSystem::new("system1", log.clone()));
        assert_eq!(scheduler.system_count(), 1);

        scheduler.add_system(TestSystem::new("system2", log.clone()));
        assert_eq!(scheduler.system_count(), 2);
    }

    #[test]
    fn test_execution_order() {
        let mut scheduler = SystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        // Add systems in specific order
        scheduler.add_system(TestSystem::new("first", log.clone()));
        scheduler.add_system(TestSystem::new("second", log.clone()));
        scheduler.add_system(TestSystem::new("third", log.clone()));

        let mut world = World::new();
        scheduler.run_tick(&mut world);

        let execution_order = log.lock().unwrap();
        let expected = vec![
            "first_before",
            "second_before",
            "third_before", // All before_run
            "first_run",
            "second_run",
            "third_run", // All run
            "first_after",
            "second_after",
            "third_after", // All after_run
        ];

        assert_eq!(*execution_order, expected);
    }

    #[test]
    fn test_three_phase_execution() {
        let mut scheduler = SystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        scheduler.add_system(TestSystem::new("system", log.clone()));

        let mut world = World::new();
        scheduler.run_tick(&mut world);

        let execution_order = log.lock().unwrap();
        assert_eq!(
            *execution_order,
            vec!["system_before", "system_run", "system_after"]
        );
    }

    struct IncrementSystem;
    impl System for IncrementSystem {
        fn run(&self, world: &mut World) {
            for entity in world.entities().cloned().collect::<Vec<_>>() {
                if let Some(counter) = world.get_component::<Counter>(entity) {
                    let new_counter = Counter {
                        count: counter.count + 1,
                    };
                    world.replace_component(entity, new_counter);
                }
            }
        }
    }

    #[test]
    fn test_system_modifies_world() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world.add_component(entity, Counter { count: 0 }).unwrap();

        let mut scheduler = SystemScheduler::new();
        scheduler.add_system(IncrementSystem);

        // Run one tick
        scheduler.run_tick(&mut world);

        // Counter should be incremented
        let counter = world.get_component::<Counter>(entity).unwrap();
        assert_eq!(counter.count, 1);

        // Run another tick
        scheduler.run_tick(&mut world);
        let counter = world.get_component::<Counter>(entity).unwrap();
        assert_eq!(counter.count, 2);
    }

    #[test]
    fn test_empty_scheduler() {
        let scheduler = SystemScheduler::new();
        let mut world = World::new();

        // Should not panic with no systems
        scheduler.run_tick(&mut world);
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn test_multiple_ticks() {
        let mut scheduler = SystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        scheduler.add_system(TestSystem::new("system", log.clone()));

        let mut world = World::new();

        // Run multiple ticks
        scheduler.run_tick(&mut world);
        scheduler.run_tick(&mut world);

        let execution_order = log.lock().unwrap();
        let expected = vec![
            "system_before",
            "system_run",
            "system_after", // Tick 1
            "system_before",
            "system_run",
            "system_after", // Tick 2
        ];

        assert_eq!(*execution_order, expected);
    }

    #[test]
    fn test_automatic_entity_cleanup() {
        let mut world = World::new();
        let mut scheduler = SystemScheduler::new();

        // Create a system that deletes entities
        struct EntityDeleterSystem;
        impl System for EntityDeleterSystem {
            fn run(&self, world: &mut World) {
                // Delete all entities with counter value >= 5
                let to_delete: Vec<_> = world
                    .entities()
                    .cloned()
                    .filter(|&entity| {
                        if let Some(counter) = world.get_component::<Counter>(entity) {
                            counter.count >= 5
                        } else {
                            false
                        }
                    })
                    .collect();

                for entity in to_delete {
                    world.delete_entity(entity);
                }
            }
        }

        scheduler.add_system(EntityDeleterSystem);

        // Create some entities with counters
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        world.add_component(entity1, Counter { count: 3 }).unwrap();
        world.add_component(entity2, Counter { count: 5 }).unwrap(); // Will be deleted
        world.add_component(entity3, Counter { count: 7 }).unwrap(); // Will be deleted

        // Before tick: 3 entities, all have components
        assert_eq!(world.entities().count(), 3);
        assert!(world.has_component::<Counter>(entity1));
        assert!(world.has_component::<Counter>(entity2));
        assert!(world.has_component::<Counter>(entity3));

        // Run one tick - should delete entities 2 and 3, and automatically clean them up
        scheduler.run_tick(&mut world);

        // After tick: 1 entity remains, deleted entities are cleaned up
        assert_eq!(world.entities().count(), 1);
        assert!(world.has_component::<Counter>(entity1)); // Should remain
        assert!(!world.has_component::<Counter>(entity2)); // Should be cleaned up
        assert!(!world.has_component::<Counter>(entity3)); // Should be cleaned up

        // Verify component data was cleaned from storage
        assert_eq!(world.get_component::<Counter>(entity2), None);
        assert_eq!(world.get_component::<Counter>(entity3), None);

        // Entities should not be in soft_deleted list anymore (internal verification)
        // This is verified implicitly by the fact that manual cleanup isn't needed
    }
}
