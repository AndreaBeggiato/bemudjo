use crate::{Component, ComponentStorage, Entity};

use super::World;

impl World {
    /// Spawns a new entity in the world.
    ///
    /// Each entity is guaranteed to have a unique identifier that can be used
    /// to attach components and perform operations.
    ///
    /// # Returns
    /// The newly created `Entity` with a unique identifier.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::World;
    ///
    /// let mut world = World::new();
    /// let entity1 = world.spawn_entity();
    /// let entity2 = world.spawn_entity();
    ///
    /// assert_ne!(entity1, entity2); // Entities have unique identifiers
    /// assert_eq!(world.entities().count(), 2);
    /// ```
    pub fn spawn_entity(&mut self) -> Entity {
        let entity = Entity::new();
        self.entities.insert(entity);
        entity
    }

    /// Returns an iterator over all active entities (excludes deleted entities).
    ///
    /// The iterator yields references to `Entity` objects that are currently active
    /// in the world. Deleted entities are automatically excluded.
    ///
    /// # Returns
    /// An iterator over `&Entity` references.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::World;
    ///
    /// let mut world = World::new();
    /// let entity1 = world.spawn_entity();
    /// let entity2 = world.spawn_entity();
    ///
    /// assert_eq!(world.entities().count(), 2);
    ///
    /// world.delete_entity(entity1);
    /// assert_eq!(world.entities().count(), 1);
    /// ```
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    /// Gets all entities that have a specific component type.
    ///
    /// This is an internal method used by the query system for component-first iteration.
    /// Instead of checking all entities for a component, this returns only entities
    /// that actually have the component, providing significant performance improvements.
    ///
    /// # Performance
    /// This method has O(entities_with_component_T) complexity instead of O(total_entities),
    /// which can be 10-100x faster for sparse components.
    ///
    /// # Returns
    /// A vector of entities that have component type T. Returns empty vector if no
    /// entities have this component type or if the component storage doesn't exist.
    pub(crate) fn entities_with_component<T: Component>(&self) -> Vec<crate::Entity> {
        self.get_storage::<T>()
            .map(|storage| {
                storage
                    .entities()
                    .filter(|&entity| self.is_entity_active(entity))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Deletes an entity from the world.
    ///
    /// The entity will no longer be accessible for component operations, but actual
    /// cleanup of component data happens during the next cleanup cycle for performance.
    /// Multiple calls to delete the same entity are safe and have no additional effect.
    ///
    /// # Parameters
    /// * `entity` - The entity to delete
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    ///
    /// world.add_component(entity, Position { x: 1.0, y: 2.0 }).unwrap();
    /// assert!(world.has_component::<Position>(entity));
    ///
    /// world.delete_entity(entity);
    /// assert!(!world.has_component::<Position>(entity));
    /// ```
    pub fn delete_entity(&mut self, entity: Entity) {
        if self.entities.contains(&entity) {
            self.entities.remove(&entity);
            self.soft_deleted_entities.insert(entity);
        }
    }

    /// Performs cleanup of deleted entities.
    ///
    /// This method removes component data for all deleted entities from storage.
    /// It's typically called automatically by the ECS at appropriate times
    /// (end of frame, maintenance cycles, etc.) but can be called manually if needed.
    /// Multiple calls are safe and efficient.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    ///
    /// world.add_component(entity, Position { x: 1.0, y: 2.0 }).unwrap();
    /// world.delete_entity(entity);
    ///
    /// // Cleanup removes component data from storage
    /// world.cleanup_deleted_entities();
    /// ```
    pub fn cleanup_deleted_entities(&mut self) {
        for entity in self.soft_deleted_entities.iter() {
            for storage in self.component_storages.values_mut() {
                storage.remove_entity(*entity);
            }
        }
        self.soft_deleted_entities.clear();
    }

    /// Checks if an entity is active (exists and hasn't been soft-deleted).
    pub(super) fn is_entity_active(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Component;

    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, PartialEq)]
    struct Health {
        value: u32,
    }
    impl Component for Health {}

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }
    impl Component for Velocity {}

    #[test]
    fn test_entity_active_status() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Newly spawned entity should be active
        assert!(world.is_entity_active(entity));

        // Entity should become inactive after deletion
        world.delete_entity(entity);
        assert!(!world.is_entity_active(entity));

        // Entity should remain inactive after cleanup
        world.cleanup_deleted_entities();
        assert!(!world.is_entity_active(entity));

        // Different entity should be unaffected
        let other_entity = world.spawn_entity();
        assert!(world.is_entity_active(other_entity));
    }

    #[test]
    fn test_spawn_entity_unique_ids() {
        let mut world = World::new();

        // Spawn multiple entities and ensure they're all unique
        let mut entities = Vec::new();
        for _ in 0..100 {
            let entity = world.spawn_entity();
            assert!(!entities.contains(&entity), "Entity should be unique");
            entities.push(entity);
        }

        assert_eq!(entities.len(), 100);
        assert_eq!(world.entities().count(), 100);
    }

    #[test]
    fn test_spawn_entity_basic() {
        let mut world = World::new();

        // Empty world should have no entities
        assert_eq!(world.entities().count(), 0);

        // Spawn first entity
        let entity1 = world.spawn_entity();
        assert_eq!(world.entities().count(), 1);
        assert!(world.is_entity_active(entity1));

        // Spawn second entity
        let entity2 = world.spawn_entity();
        assert_eq!(world.entities().count(), 2);
        assert!(world.is_entity_active(entity1));
        assert!(world.is_entity_active(entity2));
        assert_ne!(entity1, entity2);
    }

    #[test]
    fn test_entities_iterator_empty() {
        let world = World::new();

        let entities: Vec<_> = world.entities().cloned().collect();
        assert!(entities.is_empty());
        assert_eq!(world.entities().count(), 0);
    }

    #[test]
    fn test_entities_iterator_with_entities() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        let entities: Vec<_> = world.entities().cloned().collect();
        assert_eq!(entities.len(), 3);
        assert!(entities.contains(&entity1));
        assert!(entities.contains(&entity2));
        assert!(entities.contains(&entity3));
    }

    #[test]
    fn test_entities_iterator_excludes_deleted() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        // Delete middle entity
        world.delete_entity(entity2);

        let entities: Vec<_> = world.entities().cloned().collect();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&entity1));
        assert!(!entities.contains(&entity2));
        assert!(entities.contains(&entity3));
    }

    #[test]
    fn test_delete_entity_valid() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Entity should exist initially
        assert_eq!(world.entities().count(), 1);
        assert!(world.is_entity_active(entity));

        // Delete the entity
        world.delete_entity(entity);

        // Entity should no longer be in active entities
        assert_eq!(world.entities().count(), 0);
        assert!(!world.is_entity_active(entity));
    }

    #[test]
    fn test_delete_entity_nonexistent() {
        let mut world = World::new();
        let mut other_world = World::new();
        let other_entity = other_world.spawn_entity();

        // Deleting entity from different world should be safe
        world.delete_entity(other_entity);
        assert_eq!(world.entities().count(), 0);
    }

    #[test]
    fn test_delete_entity_multiple_times() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Delete entity multiple times - should be safe
        world.delete_entity(entity);
        world.delete_entity(entity);
        world.delete_entity(entity);

        assert_eq!(world.entities().count(), 0);
        assert!(!world.is_entity_active(entity));
    }

    #[test]
    fn test_delete_entity_with_components() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add component to entity
        world
            .add_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();
        assert!(world.has_component::<Position>(entity));

        // Delete entity
        world.delete_entity(entity);

        // Component should no longer be accessible
        assert!(!world.has_component::<Position>(entity));
        assert_eq!(world.get_component::<Position>(entity), None);
    }

    #[test]
    fn test_cleanup_deleted_entities_empty() {
        let mut world = World::new();

        // Cleanup on empty world should be safe
        world.cleanup_deleted_entities();
        assert_eq!(world.entities().count(), 0);
    }

    #[test]
    fn test_cleanup_deleted_entities_no_deleted() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Cleanup with no deleted entities should be safe
        world.cleanup_deleted_entities();
        assert_eq!(world.entities().count(), 2);
        assert!(world.is_entity_active(entity1));
        assert!(world.is_entity_active(entity2));
    }

    #[test]
    fn test_cleanup_deleted_entities_with_components() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Add components
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();

        // Delete one entity
        world.delete_entity(entity1);

        // Before cleanup - component still not accessible but storage might retain data
        assert!(!world.has_component::<Position>(entity1));
        assert!(world.has_component::<Position>(entity2));

        // After cleanup - storage should be cleaned
        world.cleanup_deleted_entities();
        assert!(!world.has_component::<Position>(entity1));
        assert!(world.has_component::<Position>(entity2));
        assert!(!world.is_entity_active(entity1));
        assert!(world.is_entity_active(entity2));
    }

    #[test]
    fn test_cleanup_deleted_entities_multiple_calls() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world.delete_entity(entity);

        // Multiple cleanup calls should be safe
        world.cleanup_deleted_entities();
        world.cleanup_deleted_entities();
        world.cleanup_deleted_entities();

        assert!(!world.is_entity_active(entity));
        assert!(!world.has_component::<Position>(entity));
    }

    #[test]
    fn test_entity_lifecycle_integration() {
        let mut world = World::new();

        // Start with empty world
        assert_eq!(world.entities().count(), 0);

        // Spawn entities
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();
        assert_eq!(world.entities().count(), 3);

        // Add components
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();
        world
            .add_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();

        // Delete middle entity
        world.delete_entity(entity2);
        assert_eq!(world.entities().count(), 2);
        assert!(world.is_entity_active(entity1));
        assert!(!world.is_entity_active(entity2));
        assert!(world.is_entity_active(entity3));

        // Components should reflect deletion
        assert!(world.has_component::<Position>(entity1));
        assert!(!world.has_component::<Position>(entity2));
        assert!(world.has_component::<Position>(entity3));

        // Cleanup
        world.cleanup_deleted_entities();
        assert_eq!(world.entities().count(), 2);
        assert!(world.has_component::<Position>(entity1));
        assert!(!world.has_component::<Position>(entity2));
        assert!(world.has_component::<Position>(entity3));

        // Spawn new entity after cleanup
        let entity4 = world.spawn_entity();
        assert_eq!(world.entities().count(), 3);
        assert!(world.is_entity_active(entity4));

        // New entity should be clean
        assert!(!world.has_component::<Position>(entity4));
    }

    #[test]
    fn test_entity_reuse_after_cleanup() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();

        // Add component and delete
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world.delete_entity(entity1);
        world.cleanup_deleted_entities();

        // Spawn new entity (might reuse ID due to atomic counter)
        let entity2 = world.spawn_entity();

        // New entity should be clean even if it has same ID
        assert!(!world.has_component::<Position>(entity2));
        assert!(world.is_entity_active(entity2));

        // Should be able to add components to new entity
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();
        assert!(world.has_component::<Position>(entity2));
    }

    #[test]
    fn test_massive_entity_operations() {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn many entities
        for i in 0..1000 {
            let entity = world.spawn_entity();
            entities.push(entity);
            world
                .add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: (i * 2) as f32,
                    },
                )
                .unwrap();
        }

        assert_eq!(world.entities().count(), 1000);

        // Delete every other entity
        for (i, &entity) in entities.iter().enumerate() {
            if i % 2 == 0 {
                world.delete_entity(entity);
            }
        }

        assert_eq!(world.entities().count(), 500);

        // Verify remaining entities have correct data
        for (i, &entity) in entities.iter().enumerate() {
            if i % 2 == 0 {
                // Deleted entities
                assert!(!world.is_entity_active(entity));
                assert!(!world.has_component::<Position>(entity));
            } else {
                // Active entities
                assert!(world.is_entity_active(entity));
                assert!(world.has_component::<Position>(entity));
                let pos = world.get_component::<Position>(entity).unwrap();
                assert_eq!(pos.x, i as f32);
                assert_eq!(pos.y, (i * 2) as f32);
            }
        }

        // Cleanup and verify
        world.cleanup_deleted_entities();
        assert_eq!(world.entities().count(), 500);

        // All remaining entities should still work correctly
        for (i, &entity) in entities.iter().enumerate() {
            if i % 2 != 0 {
                assert!(world.is_entity_active(entity));
                assert!(world.has_component::<Position>(entity));
            }
        }
    }

    #[test]
    fn test_cross_world_entity_safety() {
        let mut world1 = World::new();
        let mut world2 = World::new();

        let entity1 = world1.spawn_entity();
        let entity2 = world2.spawn_entity();

        // Entities should be isolated between worlds
        assert!(!world1.is_entity_active(entity2));
        assert!(!world2.is_entity_active(entity1));

        // Operations on cross-world entities should be safe
        world1.delete_entity(entity2); // Should be no-op
        world2.delete_entity(entity1); // Should be no-op

        // Original entities should be unaffected
        assert!(world1.is_entity_active(entity1));
        assert!(world2.is_entity_active(entity2));

        // Cleanup should be safe
        world1.cleanup_deleted_entities();
        world2.cleanup_deleted_entities();

        assert!(world1.is_entity_active(entity1));
        assert!(world2.is_entity_active(entity2));
    }

    #[test]
    fn test_entities_with_component_empty_world() {
        let world = World::new();

        // Empty world should have no entities with any component
        let entities = world.entities_with_component::<Position>();
        assert!(entities.is_empty());

        let entities = world.entities_with_component::<Health>();
        assert!(entities.is_empty());
    }

    #[test]
    fn test_entities_with_component_no_matching_entities() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Add Position components
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();

        // Query for Health components (which don't exist)
        let entities = world.entities_with_component::<Health>();
        assert!(entities.is_empty());
    }

    #[test]
    fn test_entities_with_component_single_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world
            .add_component(entity, Position { x: 10.0, y: 20.0 })
            .unwrap();

        let entities = world.entities_with_component::<Position>();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0], entity);
    }

    #[test]
    fn test_entities_with_component_multiple_entities() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        // Add Position to entities 1 and 3
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();

        // Add Health to entity 2 only
        world.add_component(entity2, Health { value: 100 }).unwrap();

        let position_entities = world.entities_with_component::<Position>();
        assert_eq!(position_entities.len(), 2);
        assert!(position_entities.contains(&entity1));
        assert!(position_entities.contains(&entity3));
        assert!(!position_entities.contains(&entity2));

        let health_entities = world.entities_with_component::<Health>();
        assert_eq!(health_entities.len(), 1);
        assert_eq!(health_entities[0], entity2);
    }

    #[test]
    fn test_entities_with_component_excludes_deleted() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        // Add Position to all entities
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();
        world
            .add_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();

        // All entities should be found initially
        let entities = world.entities_with_component::<Position>();
        assert_eq!(entities.len(), 3);

        // Delete middle entity
        world.delete_entity(entity2);

        // Should now exclude deleted entity
        let entities = world.entities_with_component::<Position>();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&entity1));
        assert!(!entities.contains(&entity2));
        assert!(entities.contains(&entity3));
    }

    #[test]
    fn test_entities_with_component_after_cleanup() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();

        // Delete entity and cleanup
        world.delete_entity(entity1);
        world.cleanup_deleted_entities();

        // Should still exclude deleted entity after cleanup
        let entities = world.entities_with_component::<Position>();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0], entity2);
    }

    #[test]
    fn test_entities_with_component_mixed_components() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();
        // Entity 4: No components
        let _entity4 = world.spawn_entity();

        // Entity 1: Position only
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();

        // Entity 2: Health only
        world.add_component(entity2, Health { value: 100 }).unwrap();

        // Entity 3: Both Position and Health
        world
            .add_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();
        world.add_component(entity3, Health { value: 200 }).unwrap();

        // Entity 4: No components

        let position_entities = world.entities_with_component::<Position>();
        assert_eq!(position_entities.len(), 2);
        assert!(position_entities.contains(&entity1));
        assert!(position_entities.contains(&entity3));

        let health_entities = world.entities_with_component::<Health>();
        assert_eq!(health_entities.len(), 2);
        assert!(health_entities.contains(&entity2));
        assert!(health_entities.contains(&entity3));

        let velocity_entities = world.entities_with_component::<Velocity>();
        assert!(velocity_entities.is_empty());
    }

    #[test]
    fn test_entities_with_component_component_removal() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();

        // Initially both entities have Position
        let entities = world.entities_with_component::<Position>();
        assert_eq!(entities.len(), 2);

        // Remove component from one entity
        world.remove_component::<Position>(entity1);

        // Should now only find one entity
        let entities = world.entities_with_component::<Position>();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0], entity2);
    }

    #[test]
    fn test_entities_with_component_large_scale() {
        let mut world = World::new();
        let mut position_entities = Vec::new();
        let mut health_entities = Vec::new();

        // Create 1000 entities with mixed components
        for i in 0..1000 {
            let entity = world.spawn_entity();

            if i % 3 == 0 {
                // Every 3rd entity gets Position
                world
                    .add_component(
                        entity,
                        Position {
                            x: i as f32,
                            y: 0.0,
                        },
                    )
                    .unwrap();
                position_entities.push(entity);
            }

            if i % 5 == 0 {
                // Every 5th entity gets Health
                world
                    .add_component(entity, Health { value: i as u32 })
                    .unwrap();
                health_entities.push(entity);
            }
        }

        // Verify counts
        let found_position = world.entities_with_component::<Position>();
        let found_health = world.entities_with_component::<Health>();

        assert_eq!(found_position.len(), position_entities.len());
        assert_eq!(found_health.len(), health_entities.len());

        // Verify all expected entities are found
        for &entity in &position_entities {
            assert!(found_position.contains(&entity));
        }

        for &entity in &health_entities {
            assert!(found_health.contains(&entity));
        }

        // Verify no unexpected entities are found
        for &entity in &found_position {
            assert!(position_entities.contains(&entity));
        }

        for &entity in &found_health {
            assert!(health_entities.contains(&entity));
        }
    }

    #[test]
    fn test_entities_with_component_performance_characteristic() {
        let mut world = World::new();

        // Create many entities, but only a few with our target component
        for i in 0..10000 {
            let entity = world.spawn_entity();

            // Add Health to all entities (common component)
            world
                .add_component(entity, Health { value: i as u32 })
                .unwrap();

            // Add Position to only 1% of entities (sparse component)
            if i % 100 == 0 {
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

        // This test demonstrates the performance benefit:
        // - entities_with_component::<Position>() only checks ~100 entities
        // - versus checking all 10,000 entities in the old approach

        let position_entities = world.entities_with_component::<Position>();
        assert_eq!(position_entities.len(), 100); // 10000 / 100 = 100

        let health_entities = world.entities_with_component::<Health>();
        assert_eq!(health_entities.len(), 10000); // All entities

        // Verify all Position entities are multiples of 100
        for &entity in &position_entities {
            // We can't directly check the entity ID, but we can verify
            // that all returned entities actually have the Position component
            assert!(world.has_component::<Position>(entity));
        }
    }
}
