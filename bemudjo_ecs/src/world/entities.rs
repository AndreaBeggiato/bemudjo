use crate::Entity;

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
        self.entities.contains(&entity) && !self.soft_deleted_entities.contains(&entity)
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
}
