use crate::{Component, Entity, World};
use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;

/// A unified query for filtering entities by component type.
///
/// Queries provide an efficient, iterator-based API for accessing entities
/// that have specific components. They support filtering with `.with()` and `.without()`
/// methods for regular components, and `.with_ephemeral()` and `.without_ephemeral()`
/// methods for ephemeral components.
///
/// # Basic Usage
/// ```
/// use bemudjo_ecs::{Query, World, Component};
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// let mut world = World::new();
/// let entity = world.spawn_entity();
/// world.add_component(entity, Position { x: 10.0, y: 20.0 }).unwrap();
///
/// let query = Query::<Position>::new();
/// for (entity, position) in query.iter(&world) {
///     println!("Entity {:?} at ({}, {})", entity, position.x, position.y);
/// }
/// ```
///
/// # Advanced Filtering
/// ```
/// use bemudjo_ecs::{Query, World, Component};
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Velocity { x: f32, y: f32 }
/// impl Component for Velocity {}
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Dead;
/// impl Component for Dead {}
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct DamageEvent { amount: u32 }
/// impl Component for DamageEvent {}
///
/// // Complex filtering with regular and ephemeral components
/// let complex_query = Query::<Position>::new()
///     .with::<Velocity>()                    // Must have regular Velocity
///     .without::<Dead>()                     // Must not have regular Dead
///     .with_ephemeral::<DamageEvent>()       // Must have ephemeral DamageEvent
///     .without_ephemeral::<Dead>();          // Must not have ephemeral Dead
/// ```
///
/// # Performance Benefits
/// - Skip entities without the required component using efficient set operations
/// - Direct component access without hash lookups for filtered entities
/// - Composable with iterator combinators for complex operations
/// - **Exact size hints**: Iterator provides precise entity counts for optimal memory allocation
/// - **Mixed filtering**: Combine regular and ephemeral component filtering
///
/// # Design Philosophy
/// Queries maintain the decoupled architecture by being independent structs
/// that operate on World references, rather than methods on World itself.
/// The unified design ensures all queries return the same iterator type
/// regardless of filtering complexity.
#[derive(Debug)]
pub struct Query<T> {
    /// Component types that entities must have (in addition to T)
    with_components: HashSet<TypeId>,
    /// Component types that entities must NOT have
    without_components: HashSet<TypeId>,
    /// Ephemeral component types that entities must have
    with_ephemeral_components: HashSet<TypeId>,
    /// Ephemeral component types that entities must NOT have
    without_ephemeral_components: HashSet<TypeId>,
    /// Zero-sized type marker for the primary component type
    _marker: PhantomData<T>,
}

impl<T: Component> Query<T> {
    /// Creates a new query for the specified component type.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Health { value: u32 }
    /// impl Component for Health {}
    ///
    /// let query = Query::<Health>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            with_components: HashSet::new(),
            without_components: HashSet::new(),
            with_ephemeral_components: HashSet::new(),
            without_ephemeral_components: HashSet::new(),
            _marker: PhantomData,
        }
    }

    /// Adds a condition that entities must also have another component type.
    ///
    /// Returns the same `Query<T>` type for seamless chaining and composability.
    /// Duplicate component types are automatically deduplicated.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Velocity { x: f32, y: f32 }
    /// impl Component for Velocity {}
    ///
    /// // Find entities with both Position and Velocity
    /// let movement_query = Query::<Position>::new()
    ///     .with::<Velocity>();
    /// ```
    pub fn with<C: Component>(mut self) -> Self {
        let type_id = TypeId::of::<C>();
        self.with_components.insert(type_id);
        self
    }

    /// Adds a condition that entities must NOT have another component type.
    ///
    /// Returns the same `Query<T>` type for seamless chaining and composability.
    /// Duplicate component types are automatically deduplicated.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Health { value: u32 }
    /// impl Component for Health {}
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Dead;
    /// impl Component for Dead {}
    ///
    /// // Find living entities
    /// let living_query = Query::<Health>::new()
    ///     .without::<Dead>();
    /// ```
    pub fn without<C: Component>(mut self) -> Self {
        let type_id = TypeId::of::<C>();
        self.without_components.insert(type_id);
        self
    }

    /// Adds a condition that entities must also have another ephemeral component type.
    ///
    /// Returns the same `Query<T>` type for seamless chaining and composability.
    /// Duplicate component types are automatically deduplicated.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct DamageEvent { amount: u32 }
    /// impl Component for DamageEvent {}
    ///
    /// // Find entities with Position that also have ephemeral DamageEvent
    /// let damage_query = Query::<Position>::new()
    ///     .with_ephemeral::<DamageEvent>();
    /// ```
    pub fn with_ephemeral<C: Component>(mut self) -> Self {
        let type_id = TypeId::of::<C>();
        self.with_ephemeral_components.insert(type_id);
        self
    }

    /// Adds a condition that entities must NOT have another ephemeral component type.
    ///
    /// Returns the same `Query<T>` type for seamless chaining and composability.
    /// Duplicate component types are automatically deduplicated.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Health { value: u32 }
    /// impl Component for Health {}
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct DeadEvent;
    /// impl Component for DeadEvent {}
    ///
    /// // Find entities with Health that don't have ephemeral DeadEvent
    /// let living_query = Query::<Health>::new()
    ///     .without_ephemeral::<DeadEvent>();
    /// ```
    pub fn without_ephemeral<C: Component>(mut self) -> Self {
        let type_id = TypeId::of::<C>();
        self.without_ephemeral_components.insert(type_id);
        self
    }

    /// Creates an iterator over all entities that have the specified component.
    ///
    /// Returns an iterator that yields `(Entity, &T)` pairs for each entity
    /// that matches all the query criteria using efficient set operations.
    ///
    /// # Performance
    /// This method uses set intersection and difference operations for filtering,
    /// providing O(size_of_smallest_set) complexity for multi-component queries
    /// instead of O(entities_with_T) * number_of_filters per-entity checking.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    /// world.add_component(entity, Position { x: 5.0, y: 10.0 }).unwrap();
    ///
    /// let query = Query::<Position>::new();
    /// let positions: Vec<_> = query.iter(&world)
    ///     .map(|(entity, pos)| (entity, pos.x, pos.y))
    ///     .collect();
    ///
    /// assert_eq!(positions.len(), 1);
    /// assert_eq!(positions[0].1, 5.0);
    /// assert_eq!(positions[0].2, 10.0);
    /// ```
    pub fn iter<'w>(&'w self, world: &'w World) -> impl Iterator<Item = (Entity, &'w T)> + 'w {
        // Start with entities that have the primary component T
        let mut result_entities = world.entities_with_component_by_type_id(TypeId::of::<T>());

        // Intersect with entities that have all required components
        for &type_id in &self.with_components {
            let entities_with_component = world.entities_with_component_by_type_id(type_id);
            result_entities = result_entities
                .intersection(&entities_with_component)
                .copied()
                .collect();

            // Early exit if intersection becomes empty
            if result_entities.is_empty() {
                break;
            }
        }

        // Remove entities that have any forbidden components using set difference
        for &type_id in &self.without_components {
            let entities_with_component = world.entities_with_component_by_type_id(type_id);
            result_entities = result_entities
                .difference(&entities_with_component)
                .copied()
                .collect();
        }

        // Intersect with entities that have all required ephemeral components
        for &type_id in &self.with_ephemeral_components {
            let entities_with_component =
                world.entities_with_ephemeral_component_by_type_id(type_id);
            result_entities = result_entities
                .intersection(&entities_with_component)
                .copied()
                .collect();

            // Early exit if intersection becomes empty
            if result_entities.is_empty() {
                break;
            }
        }

        // Remove entities that have any forbidden ephemeral components using set difference
        for &type_id in &self.without_ephemeral_components {
            let entities_with_component =
                world.entities_with_ephemeral_component_by_type_id(type_id);
            result_entities = result_entities
                .difference(&entities_with_component)
                .copied()
                .collect();
        }

        // Return iterator that maps entities to (Entity, &T) tuples
        result_entities.into_iter().filter_map(move |entity| {
            world
                .get_component::<T>(entity)
                .map(|component| (entity, component))
        })
    }

    /// Creates an iterator over all entities that have the specified ephemeral component.
    ///
    /// Returns an iterator that yields `(Entity, &T)` pairs for each entity
    /// that matches all the query criteria for ephemeral components using
    /// efficient set operations.
    ///
    /// # Performance
    /// This method uses set intersection and difference operations for filtering,
    /// providing O(size_of_smallest_set) complexity for multi-component queries
    /// instead of O(entities_with_T) * number_of_filters per-entity checking.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct DamageEvent { amount: u32 }
    /// impl Component for DamageEvent {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    /// world.add_ephemeral_component(entity, DamageEvent { amount: 50 }).unwrap();
    ///
    /// let query = Query::<DamageEvent>::new();
    /// let damage_events: Vec<_> = query.iter_ephemeral(&world)
    ///     .map(|(entity, damage)| (entity, damage.amount))
    ///     .collect();
    ///
    /// assert_eq!(damage_events.len(), 1);
    /// assert_eq!(damage_events[0].1, 50);
    /// ```
    pub fn iter_ephemeral<'w>(
        &'w self,
        world: &'w World,
    ) -> impl Iterator<Item = (Entity, &'w T)> + 'w {
        // Start with entities that have the primary ephemeral component T
        let mut result_entities =
            world.entities_with_ephemeral_component_by_type_id(TypeId::of::<T>());

        // Intersect with entities that have all required components (regular components for filters)
        for &type_id in &self.with_components {
            let entities_with_component = world.entities_with_component_by_type_id(type_id);
            result_entities = result_entities
                .intersection(&entities_with_component)
                .copied()
                .collect();

            // Early exit if intersection becomes empty
            if result_entities.is_empty() {
                break;
            }
        }

        // Remove entities that have any forbidden components using set difference
        for &type_id in &self.without_components {
            let entities_with_component = world.entities_with_component_by_type_id(type_id);
            result_entities = result_entities
                .difference(&entities_with_component)
                .copied()
                .collect();
        }

        // Intersect with entities that have all required ephemeral components
        for &type_id in &self.with_ephemeral_components {
            let entities_with_component =
                world.entities_with_ephemeral_component_by_type_id(type_id);
            result_entities = result_entities
                .intersection(&entities_with_component)
                .copied()
                .collect();

            // Early exit if intersection becomes empty
            if result_entities.is_empty() {
                break;
            }
        }

        // Remove entities that have any forbidden ephemeral components using set difference
        for &type_id in &self.without_ephemeral_components {
            let entities_with_component =
                world.entities_with_ephemeral_component_by_type_id(type_id);
            result_entities = result_entities
                .difference(&entities_with_component)
                .copied()
                .collect();
        }

        // Return iterator that maps entities to (Entity, &T) tuples
        result_entities.into_iter().filter_map(move |entity| {
            world
                .get_ephemeral_component::<T>(entity)
                .map(|component| (entity, component))
        })
    }
}

impl<T: Component> Default for Query<T> {
    /// Creates a new query using the default constructor.
    ///
    /// This is equivalent to calling `Query::new()`.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Component, World};

    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, PartialEq)]
    struct Health {
        value: u32,
    }
    impl Component for Health {}

    #[derive(Debug, Clone, PartialEq)]
    struct Dead;
    impl Component for Dead {}

    #[test]
    fn test_query_new() {
        let query = Query::<Position>::new();
        let world = World::new();

        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_default() {
        let query: Query<Position> = Query::default();
        let world = World::new();

        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_single_component_query() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        world
            .add_component(entity1, Position { x: 1.0, y: 2.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 3.0, y: 4.0 })
            .unwrap();
        // entity3 has no Position

        let query = Query::<Position>::new();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 2);

        // Results should contain both entities with Position
        let entity_ids: Vec<Entity> = results.iter().map(|(e, _)| *e).collect();
        assert!(entity_ids.contains(&entity1));
        assert!(entity_ids.contains(&entity2));
        assert!(!entity_ids.contains(&entity3));

        // Check component values
        for (entity, pos) in results {
            if entity == entity1 {
                assert_eq!(pos.x, 1.0);
                assert_eq!(pos.y, 2.0);
            } else if entity == entity2 {
                assert_eq!(pos.x, 3.0);
                assert_eq!(pos.y, 4.0);
            }
        }
    }

    #[test]
    fn test_query_builder_pattern() {
        let world = World::new();

        // Test chaining with filtering
        let complex_query = Query::<Position>::new()
            .with::<Velocity>()
            .without::<Dead>();

        // Verify it works
        let results: Vec<_> = complex_query.iter(&world).collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_with_empty_world() {
        let world = World::new();
        let query = Query::<Position>::new();

        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_deduplication() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world
            .add_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();
        world
            .add_component(entity, Velocity { x: 0.5, y: 1.0 })
            .unwrap();

        // Add the same component filter multiple times
        let query = Query::<Position>::new()
            .with::<Velocity>()
            .with::<Velocity>() // Duplicate - should be deduplicated
            .without::<Dead>()
            .without::<Dead>(); // Duplicate - should be deduplicated

        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, entity);
    }

    #[test]
    fn test_query_iterator_exhaustion_and_reuse() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world
            .add_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();

        let query = Query::<Position>::new();

        // First iteration
        let mut iter1 = query.iter(&world);
        assert!(iter1.next().is_some());
        assert!(iter1.next().is_none()); // Exhausted

        // Create new iterator (should work independently)
        let mut iter2 = query.iter(&world);
        assert!(iter2.next().is_some());
        assert!(iter2.next().is_none());

        // Can collect multiple times
        let results1: Vec<_> = query.iter(&world).collect();
        let results2: Vec<_> = query.iter(&world).collect();
        assert_eq!(results1.len(), 1);
        assert_eq!(results2.len(), 1);
    }

    #[test]
    fn test_query_iter_ephemeral_basic() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Add ephemeral components
        world
            .add_ephemeral_component(entity1, Position { x: 1.0, y: 2.0 })
            .unwrap();
        world
            .add_ephemeral_component(entity2, Position { x: 3.0, y: 4.0 })
            .unwrap();

        let query = Query::<Position>::new();
        let results: Vec<_> = query.iter_ephemeral(&world).collect();

        assert_eq!(results.len(), 2);

        // Check that both entities are present (order may vary)
        let entities: Vec<_> = results.iter().map(|(entity, _)| *entity).collect();
        assert!(entities.contains(&entity1));
        assert!(entities.contains(&entity2));

        // Check that the correct positions are present
        let positions: Vec<_> = results.iter().map(|(_, pos)| *pos).collect();
        assert!(positions.contains(&&Position { x: 1.0, y: 2.0 }));
        assert!(positions.contains(&&Position { x: 3.0, y: 4.0 }));
    }

    #[test]
    fn test_query_iter_ephemeral_empty() {
        let world = World::new();
        let query = Query::<Position>::new();
        let results: Vec<_> = query.iter_ephemeral(&world).collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_iter_ephemeral_vs_regular_separation() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Add regular component to entity1
        world
            .add_component(entity1, Position { x: 10.0, y: 20.0 })
            .unwrap();

        // Add ephemeral component to entity2
        world
            .add_ephemeral_component(entity2, Position { x: 30.0, y: 40.0 })
            .unwrap();

        let query = Query::<Position>::new();

        // Regular query should only find entity1
        let regular_results: Vec<_> = query.iter(&world).collect();
        assert_eq!(regular_results.len(), 1);
        assert_eq!(regular_results[0].0, entity1);
        assert_eq!(regular_results[0].1, &Position { x: 10.0, y: 20.0 });

        // Ephemeral query should only find entity2
        let ephemeral_results: Vec<_> = query.iter_ephemeral(&world).collect();
        assert_eq!(ephemeral_results.len(), 1);
        assert_eq!(ephemeral_results[0].0, entity2);
        assert_eq!(ephemeral_results[0].1, &Position { x: 30.0, y: 40.0 });
    }

    #[test]
    fn test_query_iter_ephemeral_with_filtering() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        // Add ephemeral Position to all entities
        world
            .add_ephemeral_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_ephemeral_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();
        world
            .add_ephemeral_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();

        // Add regular Velocity to entity1 and entity2 only
        world
            .add_component(entity1, Velocity { x: 0.1, y: 0.1 })
            .unwrap();
        world
            .add_component(entity2, Velocity { x: 0.2, y: 0.2 })
            .unwrap();

        // Add regular Health to entity2 only
        world.add_component(entity2, Health { value: 100 }).unwrap();

        // Query ephemeral Position with Velocity (should find entity1 and entity2)
        let query_with_velocity = Query::<Position>::new().with::<Velocity>();
        let results_with_velocity: Vec<_> = query_with_velocity.iter_ephemeral(&world).collect();
        assert_eq!(results_with_velocity.len(), 2);

        // Query ephemeral Position with Velocity but without Health (should find only entity1)
        let query_without_health = Query::<Position>::new()
            .with::<Velocity>()
            .without::<Health>();
        let results_without_health: Vec<_> = query_without_health.iter_ephemeral(&world).collect();
        assert_eq!(results_without_health.len(), 1);
        assert_eq!(results_without_health[0].0, entity1);
    }

    #[test]
    fn test_query_iter_ephemeral_after_cleanup() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add ephemeral component
        world
            .add_ephemeral_component(entity, Position { x: 5.0, y: 10.0 })
            .unwrap();

        let query = Query::<Position>::new();

        // Should find the ephemeral component
        let results_before: Vec<_> = query.iter_ephemeral(&world).collect();
        assert_eq!(results_before.len(), 1);

        // Clean ephemeral storage
        world.clean_ephemeral_storage();

        // Should not find any ephemeral components after cleanup
        let results_after: Vec<_> = query.iter_ephemeral(&world).collect();
        assert_eq!(results_after.len(), 0);
    }

    #[test]
    fn test_query_iter_ephemeral_deleted_entities() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Add ephemeral components
        world
            .add_ephemeral_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_ephemeral_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();

        // Delete entity1
        world.delete_entity(entity1);

        let query = Query::<Position>::new();
        let results: Vec<_> = query.iter_ephemeral(&world).collect();

        // Should only find entity2 (entity1 is deleted)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, entity2);
        assert_eq!(results[0].1, &Position { x: 2.0, y: 2.0 });
    }

    #[test]
    fn test_query_iter_ephemeral_same_entity_both_storages() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add both regular and ephemeral Position components to same entity
        world
            .add_component(entity, Position { x: 100.0, y: 200.0 })
            .unwrap();
        world
            .add_ephemeral_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();

        let query = Query::<Position>::new();

        // Regular query should return regular component
        let regular_results: Vec<_> = query.iter(&world).collect();
        assert_eq!(regular_results.len(), 1);
        assert_eq!(regular_results[0].1, &Position { x: 100.0, y: 200.0 });

        // Ephemeral query should return ephemeral component
        let ephemeral_results: Vec<_> = query.iter_ephemeral(&world).collect();
        assert_eq!(ephemeral_results.len(), 1);
        assert_eq!(ephemeral_results[0].1, &Position { x: 1.0, y: 2.0 });
    }

    #[test]
    fn test_query_with_ephemeral_components() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        // Add regular Position to all entities
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();
        world
            .add_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();

        // Add ephemeral Health to entity1 and entity2
        world
            .add_ephemeral_component(entity1, Health { value: 100 })
            .unwrap();
        world
            .add_ephemeral_component(entity2, Health { value: 50 })
            .unwrap();

        // Add ephemeral Dead to entity2 only
        world.add_ephemeral_component(entity2, Dead).unwrap();

        // Query Position with ephemeral Health (should find entity1 and entity2)
        let query_with_health = Query::<Position>::new().with_ephemeral::<Health>();
        let results_with_health: Vec<_> = query_with_health.iter(&world).collect();
        assert_eq!(results_with_health.len(), 2);

        // Query Position with ephemeral Health but without ephemeral Dead (should find only entity1)
        let query_without_dead = Query::<Position>::new()
            .with_ephemeral::<Health>()
            .without_ephemeral::<Dead>();
        let results_without_dead: Vec<_> = query_without_dead.iter(&world).collect();
        assert_eq!(results_without_dead.len(), 1);
        assert_eq!(results_without_dead[0].0, entity1);

        // Query Position without ephemeral Health (should find only entity3)
        let query_without_health = Query::<Position>::new().without_ephemeral::<Health>();
        let results_without_health: Vec<_> = query_without_health.iter(&world).collect();
        assert_eq!(results_without_health.len(), 1);
        assert_eq!(results_without_health[0].0, entity3);
    }

    #[test]
    fn test_query_mixed_regular_and_ephemeral_filtering() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        // Add regular Position to all entities
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();
        world
            .add_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();

        // Add regular Velocity to entity1 and entity2
        world
            .add_component(entity1, Velocity { x: 0.1, y: 0.1 })
            .unwrap();
        world
            .add_component(entity2, Velocity { x: 0.2, y: 0.2 })
            .unwrap();

        // Add ephemeral Health to entity1 and entity3
        world
            .add_ephemeral_component(entity1, Health { value: 100 })
            .unwrap();
        world
            .add_ephemeral_component(entity3, Health { value: 75 })
            .unwrap();

        // Query Position with regular Velocity AND ephemeral Health (should find only entity1)
        let mixed_query = Query::<Position>::new()
            .with::<Velocity>()
            .with_ephemeral::<Health>();
        let mixed_results: Vec<_> = mixed_query.iter(&world).collect();
        assert_eq!(mixed_results.len(), 1);
        assert_eq!(mixed_results[0].0, entity1);

        // Query Position with regular Velocity but without ephemeral Health (should find only entity2)
        let mixed_query2 = Query::<Position>::new()
            .with::<Velocity>()
            .without_ephemeral::<Health>();
        let mixed_results2: Vec<_> = mixed_query2.iter(&world).collect();
        assert_eq!(mixed_results2.len(), 1);
        assert_eq!(mixed_results2[0].0, entity2);
    }

    #[test]
    fn test_query_ephemeral_with_ephemeral_filtering() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        // Add ephemeral Position to all entities
        world
            .add_ephemeral_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_ephemeral_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();
        world
            .add_ephemeral_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();

        // Add ephemeral Health to entity1 and entity2
        world
            .add_ephemeral_component(entity1, Health { value: 100 })
            .unwrap();
        world
            .add_ephemeral_component(entity2, Health { value: 50 })
            .unwrap();

        // Add ephemeral Dead to entity2 only
        world.add_ephemeral_component(entity2, Dead).unwrap();

        // Query ephemeral Position with ephemeral Health (should find entity1 and entity2)
        let query_with_health = Query::<Position>::new().with_ephemeral::<Health>();
        let results_with_health: Vec<_> = query_with_health.iter_ephemeral(&world).collect();
        assert_eq!(results_with_health.len(), 2);

        // Query ephemeral Position with ephemeral Health but without ephemeral Dead (should find only entity1)
        let query_without_dead = Query::<Position>::new()
            .with_ephemeral::<Health>()
            .without_ephemeral::<Dead>();
        let results_without_dead: Vec<_> = query_without_dead.iter_ephemeral(&world).collect();
        assert_eq!(results_without_dead.len(), 1);
        assert_eq!(results_without_dead[0].0, entity1);
    }
}
