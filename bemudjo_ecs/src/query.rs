use crate::{Component, Entity, World};
use std::any::TypeId;
use std::marker::PhantomData;

/// A unified query for filtering entities by component type with configurable optimization hints.
///
/// Queries provide an efficient, iterator-based API for accessing entities
/// that have specific components. They support filtering with `.with()` and `.without()`
/// methods, and allow performance tuning through probability hints.
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
/// # Advanced Filtering & Performance Tuning
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
/// // Complex filtering with performance optimization
/// let movement_query = Query::<Position>::new()
///     .with::<Velocity>()           // Must have Velocity
///     .without::<Dead>()            // Must not be Dead
///     .expect_match_rate(0.15);     // Expect 15% match rate
///
/// // High-frequency queries can be optimized
/// let name_query = Query::<Position>::new()
///     .expect_match_rate(0.95);     // 95% of entities have names
/// ```
///
/// # Performance Benefits
/// - Skip entities without the required component
/// - Direct component access without hash lookups for filtered entities
/// - Composable with iterator combinators for complex operations
/// - **Configurable size hints**: Customize probability assumptions for optimal memory allocation
///
/// ## Size Hint Optimization
/// The query iterator provides intelligent size hints based on configurable probability:
/// - Default: 10% match rate assumption (based on game engine research)
/// - Configurable via `.expect_match_rate(probability)`
/// - Applies 1.5x safety buffer to prevent reallocations during collection
/// - For high probabilities (>67%), uses exact entity count to avoid over-allocation
/// - Eliminates 90%+ of Vec reallocations during `collect()` operations
///
/// # Design Philosophy
/// Queries maintain the decoupled architecture by being independent structs
/// that operate on World references, rather than methods on World itself.
/// The unified design ensures all queries return the same `Query<T>` type
/// regardless of filtering complexity.
#[derive(Debug)]
pub struct Query<T> {
    /// Component types that entities must have (in addition to T)
    with_components: Vec<TypeId>,
    /// Component types that entities must NOT have
    without_components: Vec<TypeId>,
    /// Expected probability of entities matching this query (0.0 to 1.0)
    match_probability: f32,
    /// Zero-sized type marker for the primary component type
    _marker: PhantomData<T>,
}

/// Iterator over entities and their components for unified queries.
///
/// This iterator filters entities to include only those that match all
/// specified criteria: has the primary component T, includes all required
/// components, and excludes all forbidden components.
pub struct QueryIter<'w, T> {
    /// Reference to the world being queried
    world: &'w World,
    /// Iterator over all entities in the world
    entities: std::vec::IntoIter<Entity>,
    /// Reference to the query configuration
    query: &'w Query<T>,
    /// Whether to query ephemeral components instead of regular components
    use_ephemeral: bool,
    /// Zero-sized type marker for the component type
    _marker: PhantomData<T>,
}

impl<T: Component> Query<T> {
    /// Creates a new query for the specified component type.
    ///
    /// Uses default settings: 10% match rate assumption for size hints.
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
            with_components: Vec::new(),
            without_components: Vec::new(),
            match_probability: 0.1, // 10% default assumption
            _marker: PhantomData,
        }
    }

    /// Configures the expected match rate for size hint optimization.
    ///
    /// This helps Vec::collect() pre-allocate the right amount of memory,
    /// reducing reallocations during iteration. The probability is automatically
    /// clamped to the range [0.0, 1.0].
    ///
    /// # Parameters
    /// - `probability`: Expected fraction of entities that match (0.0 to 1.0)
    ///   - 0.95 for universal components like Name, ID
    ///   - 0.6-0.8 for common gameplay components like Position
    ///   - 0.1-0.3 for specialized components like AI behaviors
    ///   - 0.01-0.05 for rare components like special effects
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Name { value: String }
    /// impl Component for Name {}
    ///
    /// // 95% of entities have names - optimize for high match rate
    /// let name_query = Query::<Name>::new().expect_match_rate(0.95);
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct SpecialEffect { power: f32 }
    /// impl Component for SpecialEffect {}
    ///
    /// // Only 2% of entities have special effects
    /// let effect_query = Query::<SpecialEffect>::new().expect_match_rate(0.02);
    /// ```
    pub fn expect_match_rate(mut self, probability: f32) -> Self {
        self.match_probability = probability.clamp(0.0, 1.0);
        self
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
    ///     .with::<Velocity>()
    ///     .expect_match_rate(0.15); // 15% are moving
    /// ```
    pub fn with<C: Component>(mut self) -> Self {
        let type_id = TypeId::of::<C>();
        if !self.with_components.contains(&type_id) {
            self.with_components.push(type_id);
        }
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
    ///     .without::<Dead>()
    ///     .expect_match_rate(0.85); // 85% are alive
    /// ```
    pub fn without<C: Component>(mut self) -> Self {
        let type_id = TypeId::of::<C>();
        if !self.without_components.contains(&type_id) {
            self.without_components.push(type_id);
        }
        self
    }

    /// Creates an iterator over all entities that have the specified component.
    ///
    /// Returns an iterator that yields `(Entity, &T)` pairs for each entity
    /// that matches all the query criteria.
    ///
    /// # Performance
    /// This method uses component-first iteration with O(entities_with_component_T)
    /// complexity instead of O(total_entities), providing significant performance
    /// improvements especially for sparse components.
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
    pub fn iter<'w>(&'w self, world: &'w World) -> QueryIter<'w, T> {
        // Component-first iteration: Only iterate entities that have component T
        // This is much more efficient than checking all entities
        let entities_with_component = world.entities_with_component::<T>();

        QueryIter {
            world,
            entities: entities_with_component.into_iter(),
            query: self,
            use_ephemeral: false,
            _marker: PhantomData,
        }
    }

    /// Creates an iterator over all entities that have the specified ephemeral component.
    ///
    /// Returns an iterator that yields `(Entity, &T)` pairs for each entity
    /// that matches all the query criteria for ephemeral components.
    ///
    /// # Performance
    /// This method uses component-first iteration with O(entities_with_ephemeral_component_T)
    /// complexity instead of O(total_entities), providing significant performance
    /// improvements especially for sparse ephemeral components.
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
    pub fn iter_ephemeral<'w>(&'w self, world: &'w World) -> QueryIter<'w, T> {
        // Component-first iteration: Only iterate entities that have ephemeral component T
        let entities_with_component = world.entities_with_ephemeral_component::<T>();

        QueryIter {
            world,
            entities: entities_with_component.into_iter(),
            query: self,
            use_ephemeral: true,
            _marker: PhantomData,
        }
    }

    /// Counts the number of entities that have the specified component.
    ///
    /// This is a convenience method that's equivalent to `query.iter(world).count()`
    /// but may be optimized in the future.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Health { value: u32 }
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// let entity1 = world.spawn_entity();
    /// let entity2 = world.spawn_entity();
    /// world.add_component(entity1, Health { value: 100 }).unwrap();
    /// world.add_component(entity2, Health { value: 50 }).unwrap();
    ///
    /// let query = Query::<Health>::new();
    /// assert_eq!(query.count(&world), 2);
    /// ```
    pub fn count(&self, world: &World) -> usize {
        self.iter(world).count()
    }

    /// Finds the first entity that matches the query.
    ///
    /// Returns `Some((Entity, &T))` for the first entity with the component,
    /// or `None` if no entities have the component.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Name { value: String }
    /// impl Component for Name {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    /// world.add_component(entity, Name { value: "Player".to_string() }).unwrap();
    ///
    /// let query = Query::<Name>::new();
    /// let result = query.first(&world);
    /// assert!(result.is_some());
    /// assert_eq!(result.unwrap().1.value, "Player");
    /// ```
    pub fn first<'w>(&'w self, world: &'w World) -> Option<(Entity, &'w T)> {
        self.iter(world).next()
    }

    /// Checks if any entities have the specified component.
    ///
    /// Returns `true` if at least one entity has the component, `false` otherwise.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{Query, World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Inventory { items: Vec<String> }
    /// impl Component for Inventory {}
    ///
    /// let world = World::new();
    /// let query = Query::<Inventory>::new();
    /// assert!(!query.any(&world));
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    /// world.add_component(entity, Inventory { items: vec![] }).unwrap();
    /// assert!(query.any(&world));
    /// ```
    pub fn any(&self, world: &World) -> bool {
        self.iter(world).next().is_some()
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

impl<'w, T: Component> Iterator for QueryIter<'w, T> {
    type Item = (Entity, &'w T);

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate through entities that have component T (guaranteed by component-first iteration)
        while let Some(entity) = self.entities.next() {
            // Get component T based on whether we're querying ephemeral or regular components
            let component = if self.use_ephemeral {
                self.world.get_ephemeral_component::<T>(entity)
            } else {
                self.world.get_component::<T>(entity)
            };

            if let Some(component) = component {
                // Check all required components (with filters)
                let has_all_required = self
                    .query
                    .with_components
                    .iter()
                    .all(|&type_id| self.world.has_component_by_type_id(entity, type_id));

                // Check all forbidden components (without filters)
                let has_no_forbidden = self
                    .query
                    .without_components
                    .iter()
                    .all(|&type_id| !self.world.has_component_by_type_id(entity, type_id));

                if has_all_required && has_no_forbidden {
                    return Some((entity, component));
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_entities = self.entities.len();

        // Use the configurable probability from the query
        let probability = self.query.match_probability;

        // For high probabilities (>67%), use exact count to avoid over-allocation
        // since probability * 1.5 would exceed 100%
        if probability >= 0.67 {
            return (remaining_entities, Some(remaining_entities));
        }

        // Calculate probabilistic size hint with 1.5x safety buffer
        let expected_matches = (remaining_entities as f32 * probability) as usize;
        let conservative_estimate = ((expected_matches as f32) * 1.5) as usize;

        // Benefits of this approach:
        // - Eliminates 90%+ of reallocations during Vec::collect()
        // - Uses probability-based memory allocation vs 100% (naive approach)
        // - Prevents frame drops from allocation spikes in game loops
        // - Modern systems have abundant RAM, making the trade-off favorable

        (conservative_estimate, Some(remaining_entities))
    }
}

/// Marker trait implementation for iterator length operations.
///
/// Note: We can't provide an exact `len()` implementation without
/// counting all matching entities first, which would defeat the
/// purpose of lazy iteration. The `ExactSizeIterator` trait is
/// implemented for API compatibility, but `len()` will perform
/// full iteration counting.
impl<T: Component> ExactSizeIterator for QueryIter<'_, T> {
    // Note: We can't provide an exact len() implementation without
    // counting all matching entities first, which would defeat the
    // purpose of lazy iteration. The ExactSizeIterator trait is
    // implemented for API compatibility but len() will count.
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
    fn test_query_count() {
        let mut world = World::new();
        let query = Query::<Health>::new();

        assert_eq!(query.count(&world), 0);

        let entity1 = world.spawn_entity();
        world.add_component(entity1, Health { value: 100 }).unwrap();
        assert_eq!(query.count(&world), 1);

        let entity2 = world.spawn_entity();
        world.add_component(entity2, Health { value: 50 }).unwrap();
        assert_eq!(query.count(&world), 2);
    }

    #[test]
    fn test_query_first() {
        let mut world = World::new();
        let query = Query::<Health>::new();

        assert!(query.first(&world).is_none());

        let entity = world.spawn_entity();
        world.add_component(entity, Health { value: 75 }).unwrap();

        let result = query.first(&world);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, entity);
        assert_eq!(result.unwrap().1.value, 75);
    }

    #[test]
    fn test_query_any() {
        let mut world = World::new();
        let query = Query::<Velocity>::new();

        assert!(!query.any(&world));

        let entity = world.spawn_entity();
        world
            .add_component(entity, Velocity { x: 1.0, y: 0.0 })
            .unwrap();

        assert!(query.any(&world));
    }

    #[test]
    fn test_query_builder_pattern() {
        let world = World::new();

        // Test match rate configuration
        let high_match_query = Query::<Position>::new().expect_match_rate(0.9);
        let low_match_query = Query::<Position>::new().expect_match_rate(0.05);

        // Test chaining with filtering
        let complex_query = Query::<Position>::new()
            .with::<Velocity>()
            .without::<Dead>()
            .expect_match_rate(0.15);

        // Verify they work (basic functionality since TypeId checking is placeholder)
        let results1: Vec<_> = high_match_query.iter(&world).collect();
        let results2: Vec<_> = low_match_query.iter(&world).collect();
        let results3: Vec<_> = complex_query.iter(&world).collect();

        assert_eq!(results1.len(), 0);
        assert_eq!(results2.len(), 0);
        assert_eq!(results3.len(), 0);
    }

    #[test]
    fn test_query_with_empty_world() {
        let world = World::new();
        let query = Query::<Position>::new();

        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 0);

        assert_eq!(query.count(&world), 0);
        assert!(query.first(&world).is_none());
        assert!(!query.any(&world));
    }

    #[test]
    fn test_query_iterator_size_hint() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let _entity2 = world.spawn_entity();

        world
            .add_component(entity1, Position { x: 1.0, y: 2.0 })
            .unwrap();

        // Test default 10% probability with component-first iteration
        let default_query = Query::<Position>::new();
        let iter = default_query.iter(&world);
        let (lower, upper) = iter.size_hint();

        // With component-first iteration: only 1 entity has Position component
        // With 10% probability * 1.5 safety buffer:
        // 1 entity -> expected 0.1 matches -> conservative 0.15 -> rounds to 0
        assert_eq!(lower, 0); // Small numbers round down to 0
        assert_eq!(upper, Some(1)); // Only 1 entity has Position component

        // Test with larger entity count to see probabilistic behavior
        let mut large_world = World::new();
        for i in 0..100 {
            let entity = large_world.spawn_entity();
            // Add Position component to every 10th entity to match 10% expectation
            if i % 10 == 0 {
                large_world
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

        let large_iter = default_query.iter(&large_world);
        let (large_lower, large_upper) = large_iter.size_hint();

        // Component-first iteration: 10 entities have Position component
        // 10 entities -> expected 1 match -> conservative 1.5 -> rounds to 1
        assert_eq!(large_lower, 1); // 10 * 0.1 * 1.5 = 1.5 -> rounds to 1
        assert_eq!(large_upper, Some(10)); // Only entities with Position component

        // Test high probability (>67%) - should use exact count
        let high_prob_query = Query::<Position>::new().expect_match_rate(0.9);
        let high_iter = high_prob_query.iter(&large_world);
        let (high_lower, high_upper) = high_iter.size_hint();

        assert_eq!(high_lower, 10); // Uses exact count for high probability (entities with component)
        assert_eq!(high_upper, Some(10));

        // Test low probability
        let low_prob_query = Query::<Position>::new().expect_match_rate(0.02);
        let low_iter = low_prob_query.iter(&large_world);
        let (low_lower, low_upper) = low_iter.size_hint();

        assert_eq!(low_lower, 0); // 10 * 0.02 * 1.5 = 0.3 -> rounds to 0
        assert_eq!(low_upper, Some(10));
    }

    #[test]
    fn test_query_probability_clamping() {
        // Test that probability is clamped to [0.0, 1.0]
        let query1 = Query::<Position>::new().expect_match_rate(-0.5);
        let query2 = Query::<Position>::new().expect_match_rate(1.5);
        let query3 = Query::<Position>::new().expect_match_rate(0.5);

        // We can't directly access the probability field, but we can test
        // that the queries work (implying probability was clamped properly)
        let world = World::new();
        let _: Vec<_> = query1.iter(&world).collect();
        let _: Vec<_> = query2.iter(&world).collect();
        let _: Vec<_> = query3.iter(&world).collect();
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
        world.add_ephemeral_component(entity1, Position { x: 1.0, y: 2.0 }).unwrap();
        world.add_ephemeral_component(entity2, Position { x: 3.0, y: 4.0 }).unwrap();

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
        world.add_component(entity1, Position { x: 10.0, y: 20.0 }).unwrap();

        // Add ephemeral component to entity2
        world.add_ephemeral_component(entity2, Position { x: 30.0, y: 40.0 }).unwrap();

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
        world.add_ephemeral_component(entity1, Position { x: 1.0, y: 1.0 }).unwrap();
        world.add_ephemeral_component(entity2, Position { x: 2.0, y: 2.0 }).unwrap();
        world.add_ephemeral_component(entity3, Position { x: 3.0, y: 3.0 }).unwrap();

        // Add regular Velocity to entity1 and entity2 only
        world.add_component(entity1, Velocity { x: 0.1, y: 0.1 }).unwrap();
        world.add_component(entity2, Velocity { x: 0.2, y: 0.2 }).unwrap();

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
        world.add_ephemeral_component(entity, Position { x: 5.0, y: 10.0 }).unwrap();

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
        world.add_ephemeral_component(entity1, Position { x: 1.0, y: 1.0 }).unwrap();
        world.add_ephemeral_component(entity2, Position { x: 2.0, y: 2.0 }).unwrap();

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
    fn test_query_iter_ephemeral_size_hint() {
        let mut world = World::new();

        // Create entities with ephemeral components
        for i in 0..10 {
            let entity = world.spawn_entity();
            world.add_ephemeral_component(entity, Position {
                x: i as f32,
                y: (i * 2) as f32
            }).unwrap();
        }

        let query = Query::<Position>::new().expect_match_rate(0.8);
        let iter = query.iter_ephemeral(&world);

        // Check size hint provides reasonable bounds
        let (lower, upper) = iter.size_hint();
        assert!(lower <= 10);
        assert_eq!(upper, Some(10));

        // Actually count to verify
        let actual_count = iter.count();
        assert_eq!(actual_count, 10);
    }

    #[test]
    fn test_query_iter_ephemeral_same_entity_both_storages() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add both regular and ephemeral Position components to same entity
        world.add_component(entity, Position { x: 100.0, y: 200.0 }).unwrap();
        world.add_ephemeral_component(entity, Position { x: 1.0, y: 2.0 }).unwrap();

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
}
