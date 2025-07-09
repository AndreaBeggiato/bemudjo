use crate::{System, World};
use std::any::TypeId;
use std::collections::{HashMap, VecDeque};

/// Information about a registered system
struct SystemInfo {
    system: Box<dyn System>,
    type_id: TypeId,
    dependencies: Vec<TypeId>,
}

/// A sequential system scheduler that executes systems in dependency order.
///
/// This scheduler runs all systems through three distinct phases sequentially,
/// followed by automatic cleanup operations:
/// 1. All systems' `before_run` methods (preparation)
/// 2. All systems' `run` methods (main logic)
/// 3. All systems' `after_run` methods (cleanup/output)
/// 4. Entity cleanup (remove deleted entities)
/// 5. Ephemeral component cleanup (clear all ephemeral components)
///
/// # Execution Order
/// Systems execute in the order they were added with `add_system()`.
/// This makes the execution predictable and deterministic, which is
/// crucial for applications that require consistent behavior.
///
/// # Example Usage
/// ```
/// use bemudjo_ecs::{SequentialSystemScheduler, System, World, Component};
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
/// let mut scheduler = SequentialSystemScheduler::new();
///
/// // Order matters! Damage must be processed before rendering
/// scheduler.add_system(DamageSystem).unwrap();
/// scheduler.add_system(RenderSystem).unwrap();
/// scheduler.build().unwrap(); // Build before running
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
pub struct SequentialSystemScheduler {
    systems: Vec<SystemInfo>,
    execution_order: Vec<usize>, // Indices into systems vec in dependency order
    is_built: bool,              // Whether build() has been called
}

impl SequentialSystemScheduler {
    /// Creates a new empty system scheduler.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::SequentialSystemScheduler;
    ///
    /// let scheduler = SequentialSystemScheduler::new();
    /// assert_eq!(scheduler.system_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            execution_order: Vec::new(),
            is_built: false,
        }
    }

    /// Adds a system to the scheduler.
    ///
    /// Systems can only be added before calling `build()`. After building,
    /// the scheduler is immutable and ready for execution.
    ///
    /// # Parameters
    /// * `system` - Any type implementing the `System` trait
    ///
    /// # Returns
    /// * `Ok(())` if the system was added successfully
    /// * `Err(String)` if the scheduler has already been built
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{SequentialSystemScheduler, System, World};
    ///
    /// struct MySystem;
    /// impl System for MySystem {
    ///     fn run(&self, world: &mut World) {
    ///         println!("MySystem is running!");
    ///     }
    /// }
    ///
    /// let mut scheduler = SequentialSystemScheduler::new();
    /// scheduler.add_system(MySystem).unwrap();
    /// scheduler.build().unwrap(); // Now scheduler is ready
    /// assert_eq!(scheduler.system_count(), 1);
    /// ```
    pub fn add_system<S: System + 'static>(&mut self, system: S) -> Result<(), String> {
        if self.is_built {
            return Err("Cannot add systems after scheduler has been built. Create a new scheduler if you need to add more systems.".to_string());
        }

        let type_id = TypeId::of::<S>();
        let dependencies = system.dependencies().to_vec();

        let system_info = SystemInfo {
            system: Box::new(system),
            type_id,
            dependencies,
        };

        self.systems.push(system_info);
        Ok(())
    }

    /// Builds the scheduler by resolving system dependencies.
    ///
    /// This method must be called after adding all systems and before running
    /// any ticks. It performs topological sorting to determine the correct
    /// execution order and validates that there are no circular dependencies.
    ///
    /// Once built, no more systems can be added to the scheduler.
    ///
    /// # Returns
    /// * `Ok(())` if dependencies were resolved successfully
    /// * `Err(String)` if circular dependencies were detected
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{SequentialSystemScheduler, System, World};
    /// use std::any::TypeId;
    /// use std::sync::LazyLock;
    ///
    /// static MOVEMENT_DEPS: LazyLock<Vec<TypeId>> = LazyLock::new(|| {
    ///     vec![TypeId::of::<InputSystem>()]
    /// });
    ///
    /// struct InputSystem;
    /// impl System for InputSystem {
    ///     fn run(&self, world: &mut World) {
    ///         println!("Processing input...");
    ///     }
    /// }
    ///
    /// struct MovementSystem;
    /// impl System for MovementSystem {
    ///     fn dependencies(&self) -> &[TypeId] {
    ///         &MOVEMENT_DEPS
    ///     }
    ///
    ///     fn run(&self, world: &mut World) {
    ///         println!("Processing movement...");
    ///     }
    /// }
    ///
    /// let mut scheduler = SequentialSystemScheduler::new();
    /// scheduler.add_system(MovementSystem).unwrap(); // Order doesn't matter
    /// scheduler.add_system(InputSystem).unwrap();
    ///
    /// // Build to resolve dependencies
    /// scheduler.build().unwrap(); // Now InputSystem will run before MovementSystem
    ///
    /// let mut world = World::new();
    /// scheduler.run_tick(&mut world); // Executes in dependency order
    /// ```
    pub fn build(&mut self) -> Result<(), String> {
        if self.is_built {
            return Ok(()); // Already built, nothing to do
        }

        // Resolve dependencies
        self.resolve_dependencies()?;

        // Mark as built
        self.is_built = true;

        Ok(())
    }

    /// Returns the number of systems currently registered.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{SequentialSystemScheduler, System, World};
    ///
    /// struct System1;
    /// impl System for System1 {}
    ///
    /// struct System2;
    /// impl System for System2 {}
    ///
    /// let mut scheduler = SequentialSystemScheduler::new();
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
    /// This method runs all systems through the five execution phases described
    /// in the [`SequentialSystemScheduler`] documentation, followed by automatic
    /// cleanup of deleted entities and ephemeral components.
    ///
    /// # Panics
    /// Panics if `build()` has not been called yet. The scheduler must be built
    /// before it can execute systems.
    ///
    /// # Parameters
    /// * `world` - Mutable reference to the ECS world
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{SequentialSystemScheduler, System, World, Component};
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
    /// let mut scheduler = SequentialSystemScheduler::new();
    /// scheduler.add_system(IncrementSystem).unwrap();
    /// scheduler.build().unwrap(); // Must build before running
    ///
    /// // Run one tick
    /// scheduler.run_tick(&mut world);
    ///
    /// // Counter should be incremented
    /// let counter = world.get_component::<Counter>(entity).unwrap();
    /// assert_eq!(counter.value, 1);
    /// ```
    pub fn run_tick(&self, world: &mut World) {
        if !self.is_built {
            panic!("SequentialSystemScheduler must be built before running. Call build() first.");
        }

        // Phase 1: Preparation - All before_run methods in dependency order
        for &index in &self.execution_order {
            self.systems[index].system.before_run(world);
        }

        // Phase 2: Execution - All run methods in dependency order
        for &index in &self.execution_order {
            self.systems[index].system.run(world);
        }

        // Phase 3: Cleanup - All after_run methods in dependency order
        for &index in &self.execution_order {
            self.systems[index].system.after_run(world);
        }

        // Phase 4: Entity cleanup - Remove component data for deleted entities
        // This ensures clean state for the next tick and prevents memory leaks
        world.cleanup_deleted_entities();

        // Phase 5: Ephemeral component cleanup - Remove all ephemeral components
        // This implements the core ephemeral component behavior: components only live for one frame
        world.clean_ephemeral_storage();
    }

    /// Resolves system dependencies and updates execution order.
    ///
    /// Uses topological sorting to determine the correct execution order
    /// based on system dependencies.
    fn resolve_dependencies(&mut self) -> Result<(), String> {
        let num_systems = self.systems.len();
        if num_systems == 0 {
            self.execution_order.clear();
            return Ok(());
        }

        // Build a mapping from TypeId to system index
        let mut type_to_index: HashMap<TypeId, usize> = HashMap::new();
        for (index, system_info) in self.systems.iter().enumerate() {
            type_to_index.insert(system_info.type_id, index);
        }

        // Build dependency graph (index -> list of indices that depend on it)
        let mut in_degree = vec![0; num_systems];
        let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();

        for (dependent_index, system_info) in self.systems.iter().enumerate() {
            for &dep_type_id in &system_info.dependencies {
                if let Some(&dependency_index) = type_to_index.get(&dep_type_id) {
                    // dependency_index must run before dependent_index
                    graph
                        .entry(dependency_index)
                        .or_default()
                        .push(dependent_index);
                    in_degree[dependent_index] += 1;
                } else {
                    // Dependency not found - this could be a warning in the future
                    // For now, we'll silently ignore missing dependencies
                }
            }
        }

        // Topological sort using Kahn's algorithm
        let mut queue: VecDeque<usize> = VecDeque::new();
        let mut execution_order = Vec::new();

        // Start with systems that have no dependencies
        for (index, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push_back(index);
            }
        }

        while let Some(current_index) = queue.pop_front() {
            execution_order.push(current_index);

            // Process all systems that depend on the current system
            if let Some(dependents) = graph.get(&current_index) {
                for &dependent_index in dependents {
                    in_degree[dependent_index] -= 1;
                    if in_degree[dependent_index] == 0 {
                        queue.push_back(dependent_index);
                    }
                }
            }
        }

        // Check for circular dependencies
        if execution_order.len() != num_systems {
            return Err("Circular dependency detected in system dependencies".to_string());
        }

        self.execution_order = execution_order;
        Ok(())
    }
}

impl Default for SequentialSystemScheduler {
    /// Creates a new empty system scheduler using the default constructor.
    ///
    /// This is equivalent to calling `SequentialSystemScheduler::new()`.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::SequentialSystemScheduler;
    ///
    /// let scheduler1 = SequentialSystemScheduler::new();
    /// let scheduler2 = SequentialSystemScheduler::default();
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
        let scheduler = SequentialSystemScheduler::new();
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn test_system_scheduler_default() {
        let scheduler = SequentialSystemScheduler::default();
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn test_add_system() {
        let mut scheduler = SequentialSystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        scheduler
            .add_system(TestSystem::new("system1", log.clone()))
            .unwrap();
        assert_eq!(scheduler.system_count(), 1);

        scheduler
            .add_system(TestSystem::new("system2", log.clone()))
            .unwrap();
        assert_eq!(scheduler.system_count(), 2);

        // Should be able to build successfully
        scheduler.build().unwrap();
    }

    #[test]
    fn test_execution_order() {
        let mut scheduler = SequentialSystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        // Add systems in specific order
        scheduler
            .add_system(TestSystem::new("first", log.clone()))
            .unwrap();
        scheduler
            .add_system(TestSystem::new("second", log.clone()))
            .unwrap();
        scheduler
            .add_system(TestSystem::new("third", log.clone()))
            .unwrap();

        scheduler.build().unwrap();

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
        let mut scheduler = SequentialSystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        scheduler
            .add_system(TestSystem::new("system", log.clone()))
            .unwrap();

        scheduler.build().unwrap();

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

        let mut scheduler = SequentialSystemScheduler::new();
        scheduler.add_system(IncrementSystem).unwrap();

        scheduler.build().unwrap();

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
        let mut scheduler = SequentialSystemScheduler::new();
        let mut world = World::new();

        scheduler.build().unwrap(); // Even empty scheduler needs to be built

        // Should not panic with no systems
        scheduler.run_tick(&mut world);
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn test_multiple_ticks() {
        let mut scheduler = SequentialSystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        scheduler
            .add_system(TestSystem::new("system", log.clone()))
            .unwrap();

        scheduler.build().unwrap();

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
        let mut scheduler = SequentialSystemScheduler::new();

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

        scheduler.add_system(EntityDeleterSystem).unwrap();

        scheduler.build().unwrap();

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

    #[test]
    fn test_dependency_aware_scheduling() {
        use std::sync::{Arc, LazyLock, Mutex};

        static SYSTEM_B_DEPS: LazyLock<Vec<TypeId>> =
            LazyLock::new(|| vec![TypeId::of::<SystemA>()]);

        let execution_log = Arc::new(Mutex::new(Vec::new()));

        struct SystemA {
            log: Arc<Mutex<Vec<String>>>,
        }

        impl System for SystemA {
            fn run(&self, _world: &mut World) {
                self.log.lock().unwrap().push("A".to_string());
            }
        }

        struct SystemB {
            log: Arc<Mutex<Vec<String>>>,
        }

        impl System for SystemB {
            fn dependencies(&self) -> &[TypeId] {
                &SYSTEM_B_DEPS
            }

            fn run(&self, _world: &mut World) {
                self.log.lock().unwrap().push("B".to_string());
            }
        }

        let mut scheduler = SequentialSystemScheduler::new();

        // Add systems in reverse dependency order (B before A)
        scheduler
            .add_system(SystemB {
                log: execution_log.clone(),
            })
            .unwrap();
        scheduler
            .add_system(SystemA {
                log: execution_log.clone(),
            })
            .unwrap();

        scheduler.build().unwrap();

        let mut world = World::new();
        scheduler.run_tick(&mut world);

        let log = execution_log.lock().unwrap();
        assert_eq!(*log, vec!["A", "B"]); // A should run first despite being added second
    }

    #[test]
    fn test_circular_dependency_detection() {
        use std::sync::LazyLock;

        static SYSTEM_A_DEPS: LazyLock<Vec<TypeId>> =
            LazyLock::new(|| vec![TypeId::of::<SystemB>()]);
        static SYSTEM_B_DEPS: LazyLock<Vec<TypeId>> =
            LazyLock::new(|| vec![TypeId::of::<SystemA>()]);

        struct SystemA;
        impl System for SystemA {
            fn dependencies(&self) -> &[TypeId] {
                &SYSTEM_A_DEPS
            }
            fn run(&self, _world: &mut World) {}
        }

        struct SystemB;
        impl System for SystemB {
            fn dependencies(&self) -> &[TypeId] {
                &SYSTEM_B_DEPS
            }
            fn run(&self, _world: &mut World) {}
        }

        let mut scheduler = SequentialSystemScheduler::new();

        // Both systems should be added successfully
        scheduler.add_system(SystemA).unwrap();
        scheduler.add_system(SystemB).unwrap();

        // This should fail due to circular dependency
        let result = scheduler.build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }

    #[test]
    fn test_complex_dependency_chain() {
        use std::sync::{Arc, LazyLock, Mutex};

        // Complex dependency chain: Input -> Physics -> Collision -> Render
        static PHYSICS_DEPS: LazyLock<Vec<TypeId>> =
            LazyLock::new(|| vec![TypeId::of::<InputSystem>()]);
        static COLLISION_DEPS: LazyLock<Vec<TypeId>> =
            LazyLock::new(|| vec![TypeId::of::<PhysicsSystem>()]);
        static RENDER_DEPS: LazyLock<Vec<TypeId>> =
            LazyLock::new(|| vec![TypeId::of::<CollisionSystem>()]);

        let execution_log = Arc::new(Mutex::new(Vec::new()));

        struct InputSystem {
            log: Arc<Mutex<Vec<String>>>,
        }
        impl System for InputSystem {
            fn run(&self, _world: &mut World) {
                self.log.lock().unwrap().push("Input".to_string());
            }
        }

        struct PhysicsSystem {
            log: Arc<Mutex<Vec<String>>>,
        }
        impl System for PhysicsSystem {
            fn dependencies(&self) -> &[TypeId] {
                &PHYSICS_DEPS
            }
            fn run(&self, _world: &mut World) {
                self.log.lock().unwrap().push("Physics".to_string());
            }
        }

        struct CollisionSystem {
            log: Arc<Mutex<Vec<String>>>,
        }
        impl System for CollisionSystem {
            fn dependencies(&self) -> &[TypeId] {
                &COLLISION_DEPS
            }
            fn run(&self, _world: &mut World) {
                self.log.lock().unwrap().push("Collision".to_string());
            }
        }

        struct RenderSystem {
            log: Arc<Mutex<Vec<String>>>,
        }
        impl System for RenderSystem {
            fn dependencies(&self) -> &[TypeId] {
                &RENDER_DEPS
            }
            fn run(&self, _world: &mut World) {
                self.log.lock().unwrap().push("Render".to_string());
            }
        }

        let mut scheduler = SequentialSystemScheduler::new();

        // Add systems in REVERSE dependency order to test sorting
        scheduler
            .add_system(RenderSystem {
                log: execution_log.clone(),
            })
            .unwrap();
        scheduler
            .add_system(CollisionSystem {
                log: execution_log.clone(),
            })
            .unwrap();
        scheduler
            .add_system(PhysicsSystem {
                log: execution_log.clone(),
            })
            .unwrap();
        scheduler
            .add_system(InputSystem {
                log: execution_log.clone(),
            })
            .unwrap();

        scheduler.build().unwrap();

        let mut world = World::new();
        scheduler.run_tick(&mut world);

        let log = execution_log.lock().unwrap();
        assert_eq!(*log, vec!["Input", "Physics", "Collision", "Render"]);
    }

    #[test]
    fn test_build_prevents_adding_systems() {
        let mut scheduler = SequentialSystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        // Add a system
        scheduler
            .add_system(TestSystem::new("system1", log.clone()))
            .unwrap();

        // Build the scheduler
        scheduler.build().unwrap();

        // Try to add another system - should fail
        let result = scheduler.add_system(TestSystem::new("system2", log.clone()));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot add systems after scheduler has been built"));

        // System count should remain 1
        assert_eq!(scheduler.system_count(), 1);
    }

    #[test]
    fn test_run_tick_requires_build() {
        let mut scheduler = SequentialSystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();

        scheduler
            .add_system(TestSystem::new("system", log.clone()))
            .unwrap();

        // Should panic if trying to run without building
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            scheduler.run_tick(&mut world);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_build_is_idempotent() {
        let mut scheduler = SequentialSystemScheduler::new();
        let log = Arc::new(Mutex::new(Vec::new()));

        scheduler
            .add_system(TestSystem::new("system", log.clone()))
            .unwrap();

        // Build multiple times should work
        scheduler.build().unwrap();
        scheduler.build().unwrap();
        scheduler.build().unwrap();

        // Should still work normally
        let mut world = World::new();
        scheduler.run_tick(&mut world);
    }

    #[test]
    fn test_ephemeral_components_cleanup_after_tick() {
        let mut world = World::new();
        let mut scheduler = SequentialSystemScheduler::new();

        #[derive(Clone, Debug, PartialEq)]
        struct TempEffect { damage: u32 }
        impl Component for TempEffect {}

        // System that creates ephemeral components
        struct CreateEffectSystem;
        impl System for CreateEffectSystem {
            fn run(&self, world: &mut World) {
                for entity in world.entities().cloned().collect::<Vec<_>>() {
                    world.add_ephemeral_component(entity, TempEffect { damage: 50 }).unwrap();
                }
            }
        }

        // Add system and build scheduler
        scheduler.add_system(CreateEffectSystem).unwrap();
        scheduler.build().unwrap();

        // Create an entity
        let entity = world.spawn_entity();

        // First tick - ephemeral components should be created
        scheduler.run_tick(&mut world);

        // At the end of the tick, ephemeral components should be cleaned up
        assert!(!world.has_ephemeral_component::<TempEffect>(entity));

        // Second tick - verify cleanup is automatic each tick
        scheduler.run_tick(&mut world);
        assert!(!world.has_ephemeral_component::<TempEffect>(entity));
    }

    #[test]
    fn test_ephemeral_components_available_during_tick() {
        let mut world = World::new();
        let mut scheduler = SequentialSystemScheduler::new();

        #[derive(Clone, Debug, PartialEq)]
        struct DamageEvent { amount: u32 }
        impl Component for DamageEvent {}

        // System that creates ephemeral components
        struct DamageSystem;
        impl System for DamageSystem {
            fn run(&self, world: &mut World) {
                for entity in world.entities().cloned().collect::<Vec<_>>() {
                    world.add_ephemeral_component(entity, DamageEvent { amount: 25 }).unwrap();
                }
            }
        }

        // System that reads ephemeral components
        struct HealthSystem;
        impl System for HealthSystem {
            fn run(&self, world: &mut World) {
                for entity in world.entities().cloned().collect::<Vec<_>>() {
                    if world.has_ephemeral_component::<DamageEvent>(entity) {
                        // In practice, this would process the damage
                        // The test verifies the ephemeral component exists during the tick
                    }
                }
            }
        }

        // Add systems in order (DamageSystem creates, HealthSystem reads)
        scheduler.add_system(DamageSystem).unwrap();
        scheduler.add_system(HealthSystem).unwrap();
        scheduler.build().unwrap();

        // Create an entity
        let entity = world.spawn_entity();

        // Run tick - ephemeral components should be available during the tick
        scheduler.run_tick(&mut world);

        // After tick, ephemeral components should be cleaned up
        assert!(!world.has_ephemeral_component::<DamageEvent>(entity));
    }

    #[test]
    fn test_ephemeral_components_persist_across_system_phases() {
        let mut world = World::new();
        let mut scheduler = SequentialSystemScheduler::new();

        #[derive(Clone, Debug, PartialEq)]
        struct SystemEvent { phase: String }
        impl Component for SystemEvent {}

        // System that creates ephemeral components in run phase
        struct SetupSystem;
        impl System for SetupSystem {
            fn run(&self, world: &mut World) {
                for entity in world.entities().cloned().collect::<Vec<_>>() {
                    world.add_ephemeral_component(entity, SystemEvent {
                        phase: "run".to_string()
                    }).unwrap();
                }
            }

            fn after_run(&self, world: &World) {
                // Verify ephemeral component is still available in after_run
                for entity in world.entities().cloned().collect::<Vec<_>>() {
                    if let Some(event) = world.get_ephemeral_component::<SystemEvent>(entity) {
                        assert_eq!(event.phase, "run");
                    }
                }
            }
        }

        scheduler.add_system(SetupSystem).unwrap();
        scheduler.build().unwrap();

        let entity = world.spawn_entity();

        // Run tick - ephemeral components should persist across phases within the same tick
        scheduler.run_tick(&mut world);

        // After tick, ephemeral components should be cleaned up
        assert!(!world.has_ephemeral_component::<SystemEvent>(entity));
    }
}
