use crate::World;
use std::any::TypeId;

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
    /// Returns the dependencies of this system.
    ///
    /// Dependencies are systems that must execute before this system runs.
    /// The system scheduler uses this information to determine execution order.
    ///
    /// # Implementation Note
    /// Due to Rust's current limitations, `TypeId::of::<T>()` is not yet stable as a const function.
    /// Therefore, we use `LazyLock` to create static dependency arrays that are initialized once
    /// on first access. This provides type safety while working on stable Rust.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{System, World};
    /// use std::any::TypeId;
    /// use std::sync::LazyLock;
    ///
    /// // Define dependencies using LazyLock for lazy initialization
    /// static MOVEMENT_SYSTEM_DEPS: LazyLock<Vec<TypeId>> = LazyLock::new(|| {
    ///     vec![TypeId::of::<InputSystem>()]
    /// });
    ///
    /// struct InputSystem;
    /// impl System for InputSystem {
    ///     fn run(&self, world: &mut World) {
    ///         // Process input
    ///     }
    /// }
    ///
    /// struct MovementSystem;
    /// impl System for MovementSystem {
    ///     fn dependencies(&self) -> &[TypeId] {
    ///         &MOVEMENT_SYSTEM_DEPS
    ///     }
    ///
    ///     fn run(&self, world: &mut World) {
    ///         // Movement logic here
    ///     }
    /// }
    /// ```
    ///
    /// # Why LazyLock?
    /// We can't use const arrays like `const DEPS: &[TypeId] = &[TypeId::of::<T>()]` because:
    /// - `TypeId::of::<T>()` requires the unstable `const_type_id` feature (nightly-only)
    /// - We want to maintain compatibility with stable Rust
    /// - LazyLock provides thread-safe lazy initialization with minimal overhead
    /// - Dependencies are typically queried once during scheduler setup, so performance impact is negligible
    fn dependencies(&self) -> &[TypeId] {
        &[] // Default: no dependencies
    }

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

/// Example implementation of a system with dependencies.
///
/// This demonstrates the recommended pattern for creating systems with static dependencies
/// while maintaining compatibility with trait objects and stable Rust.
///
/// # Why LazyLock?
/// We use `LazyLock` instead of const arrays because `TypeId::of::<T>()` requires
/// the unstable `const_type_id` feature. LazyLock provides thread-safe lazy initialization
/// that works on stable Rust with minimal performance overhead.
///
/// # Example
/// ```
/// use bemudjo_ecs::{System, World};
/// use std::any::TypeId;
/// use std::sync::LazyLock;
///
/// // Define dependencies using LazyLock
/// static MOVEMENT_SYSTEM_DEPS: LazyLock<Vec<TypeId>> = LazyLock::new(|| {
///     vec![TypeId::of::<InputSystem>()]
/// });
///
/// // A system with no dependencies
/// struct InputSystem;
/// impl System for InputSystem {
///     fn run(&self, world: &mut World) {
///         // Process input
///     }
/// }
///
/// // A system that depends on InputSystem
/// struct MovementSystem;
/// impl System for MovementSystem {
///     fn dependencies(&self) -> &[TypeId] {
///         &MOVEMENT_SYSTEM_DEPS
///     }
///
///     fn run(&self, world: &mut World) {
///         // Movement logic - guaranteed to run after InputSystem
///     }
/// }
/// ```
#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use std::any::TypeId;

    #[test]
    fn test_system_dependencies() {
        use std::sync::LazyLock;

        // Define dependencies using LazyLock for stable Rust compatibility
        static PHYSICS_SYSTEM_DEPS: LazyLock<Vec<TypeId>> =
            LazyLock::new(|| vec![TypeId::of::<InputSystem>()]);

        static COLLISION_SYSTEM_DEPS: LazyLock<Vec<TypeId>> =
            LazyLock::new(|| vec![TypeId::of::<PhysicsSystem>()]);

        static RENDER_SYSTEM_DEPS: LazyLock<Vec<TypeId>> = LazyLock::new(|| {
            vec![
                TypeId::of::<PhysicsSystem>(),
                TypeId::of::<CollisionSystem>(),
            ]
        });

        // Test systems using the LazyLock approach
        struct InputSystem;
        impl System for InputSystem {
            fn run(&self, _world: &mut World) {
                // Input processing logic
            }
        }

        struct PhysicsSystem;
        impl System for PhysicsSystem {
            fn dependencies(&self) -> &[TypeId] {
                &PHYSICS_SYSTEM_DEPS
            }

            fn run(&self, _world: &mut World) {
                // Physics simulation
            }
        }

        struct CollisionSystem;
        impl System for CollisionSystem {
            fn dependencies(&self) -> &[TypeId] {
                &COLLISION_SYSTEM_DEPS
            }

            fn run(&self, _world: &mut World) {
                // Collision detection and response
            }
        }

        struct RenderSystem;
        impl System for RenderSystem {
            fn dependencies(&self) -> &[TypeId] {
                &RENDER_SYSTEM_DEPS
            }

            fn run(&self, _world: &mut World) {
                // Rendering logic
            }
        }

        // Test the complete dependency chain
        let input = InputSystem;
        let physics = PhysicsSystem;
        let collision = CollisionSystem;
        let render = RenderSystem;

        // Verify dependency chains
        assert_eq!(input.dependencies().len(), 0); // No dependencies

        let physics_deps = physics.dependencies();
        assert_eq!(physics_deps.len(), 1);
        assert!(physics_deps.contains(&TypeId::of::<InputSystem>()));

        let collision_deps = collision.dependencies();
        assert_eq!(collision_deps.len(), 1);
        assert!(collision_deps.contains(&TypeId::of::<PhysicsSystem>()));

        let render_deps = render.dependencies();
        assert_eq!(render_deps.len(), 2);
        assert!(render_deps.contains(&TypeId::of::<PhysicsSystem>()));
        assert!(render_deps.contains(&TypeId::of::<CollisionSystem>()));
    }

    #[test]
    fn test_default_dependencies_behavior() {
        // Test that systems without custom dependencies return empty slice
        struct SimpleSystem;
        impl System for SimpleSystem {
            fn run(&self, _world: &mut World) {}
        }

        let system = SimpleSystem;
        assert_eq!(system.dependencies().len(), 0);
        assert!(system.dependencies().is_empty());
    }
    #[test]
    fn test_duplicate_dependencies() {
        use std::sync::LazyLock;

        // Test system with duplicate dependencies in the list
        static DUPLICATE_DEPS: LazyLock<Vec<TypeId>> = LazyLock::new(|| {
            vec![
                TypeId::of::<SimpleSystem>(),
                TypeId::of::<SimpleSystem>(), // Duplicate
                TypeId::of::<SimpleSystem>(), // Another duplicate
            ]
        });

        struct DuplicateDepSystem;
        impl System for DuplicateDepSystem {
            fn dependencies(&self) -> &[TypeId] {
                &DUPLICATE_DEPS
            }

            fn run(&self, _world: &mut World) {}
        }

        struct SimpleSystem;
        impl System for SimpleSystem {
            fn run(&self, _world: &mut World) {}
        }

        let system = DuplicateDepSystem;
        let deps = system.dependencies();

        // Should have 3 entries (duplicates preserved as-is)
        assert_eq!(deps.len(), 3);

        // All should be the same TypeId
        let expected_type = TypeId::of::<SimpleSystem>();
        assert!(deps.iter().all(|&dep| dep == expected_type));
    }

    #[test]
    fn test_many_dependencies() {
        use std::sync::LazyLock;

        // Test system with many dependencies
        static MANY_DEPS: LazyLock<Vec<TypeId>> = LazyLock::new(|| {
            vec![
                TypeId::of::<System1>(),
                TypeId::of::<System2>(),
                TypeId::of::<System3>(),
                TypeId::of::<System4>(),
                TypeId::of::<System5>(),
            ]
        });

        struct System1;
        impl System for System1 {
            fn run(&self, _world: &mut World) {}
        }

        struct System2;
        impl System for System2 {
            fn run(&self, _world: &mut World) {}
        }

        struct System3;
        impl System for System3 {
            fn run(&self, _world: &mut World) {}
        }

        struct System4;
        impl System for System4 {
            fn run(&self, _world: &mut World) {}
        }

        struct System5;
        impl System for System5 {
            fn run(&self, _world: &mut World) {}
        }

        struct ManyDepsSystem;
        impl System for ManyDepsSystem {
            fn dependencies(&self) -> &[TypeId] {
                &MANY_DEPS
            }

            fn run(&self, _world: &mut World) {}
        }

        let system = ManyDepsSystem;
        let deps = system.dependencies();

        assert_eq!(deps.len(), 5);
        assert!(deps.contains(&TypeId::of::<System1>()));
        assert!(deps.contains(&TypeId::of::<System2>()));
        assert!(deps.contains(&TypeId::of::<System3>()));
        assert!(deps.contains(&TypeId::of::<System4>()));
        assert!(deps.contains(&TypeId::of::<System5>()));
    }

    #[test]
    fn test_dependencies_trait_object_safety() {
        use std::sync::LazyLock;

        // Test that dependencies work correctly with trait objects
        static SYSTEM_B_DEPS: LazyLock<Vec<TypeId>> =
            LazyLock::new(|| vec![TypeId::of::<SystemA>()]);

        struct SystemA;
        impl System for SystemA {
            fn run(&self, _world: &mut World) {}
        }

        struct SystemB;
        impl System for SystemB {
            fn dependencies(&self) -> &[TypeId] {
                &SYSTEM_B_DEPS
            }

            fn run(&self, _world: &mut World) {}
        }

        // Create systems as trait objects
        let systems: Vec<Box<dyn System>> = vec![Box::new(SystemA), Box::new(SystemB)];

        // Test that we can call dependencies through trait objects
        assert_eq!(systems[0].dependencies().len(), 0);
        assert_eq!(systems[1].dependencies().len(), 1);
        assert_eq!(systems[1].dependencies()[0], TypeId::of::<SystemA>());
    }
}
