use std::any::TypeId;

use crate::{Component, HashMapComponentStorage};

use super::World;

impl World {
    /// Gets an immutable reference to the storage for a specific component type.
    ///
    /// Returns `None` if no storage exists for this component type yet.
    pub(super) fn get_storage<T: Component>(&self) -> Option<&HashMapComponentStorage<T>> {
        let type_id = TypeId::of::<T>();

        self.component_storages
            .get(&type_id)
            .and_then(|any_storage| {
                any_storage
                    .as_any()
                    .downcast_ref::<HashMapComponentStorage<T>>()
            })
    }

    /// Gets a mutable reference to the storage for a specific component type.
    ///
    /// Creates the storage if it doesn't exist yet.
    pub(super) fn get_storage_mut<T: Component>(&mut self) -> &mut HashMapComponentStorage<T> {
        let type_id = TypeId::of::<T>();

        // Use entry API to create storage if it doesn't exist
        let any_storage = self
            .component_storages
            .entry(type_id)
            .or_insert_with(|| Box::new(HashMapComponentStorage::<T>::new()));

        any_storage
            .as_any_mut()
            .downcast_mut::<HashMapComponentStorage<T>>()
            .expect("Failed to downcast storage for component type")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnyStorage, Component, ComponentStorage};

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
    fn test_get_storage_nonexistent() {
        let world = World::new();

        // Storage for component type that was never used should return None
        let storage = world.get_storage::<Position>();
        assert!(storage.is_none());
    }

    #[test]
    fn test_get_storage_mut_creates_storage() {
        let mut world = World::new();

        // First access should create the storage
        let storage = world.get_storage_mut::<Position>();
        assert!(storage.component_type_name().contains("Position"));

        // Now get_storage should return Some
        let storage_ref = world.get_storage::<Position>();
        assert!(storage_ref.is_some());
    }

    #[test]
    fn test_get_storage_after_component_operations() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Initially no storage
        assert!(world.get_storage::<Position>().is_none());

        // Add component - this should create storage via get_storage_mut
        world
            .add_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();

        // Now storage should exist
        let storage = world.get_storage::<Position>();
        assert!(storage.is_some());
        assert!(storage.unwrap().contains(entity));
    }

    #[test]
    fn test_multiple_component_types_separate_storages() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add different component types
        world
            .add_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();
        world.add_component(entity, Health { value: 100 }).unwrap();
        world
            .add_component(entity, Velocity { dx: 0.5, dy: -0.3 })
            .unwrap();

        // Each component type should have its own storage
        let pos_storage = world.get_storage::<Position>();
        let health_storage = world.get_storage::<Health>();
        let vel_storage = world.get_storage::<Velocity>();

        assert!(pos_storage.is_some());
        assert!(health_storage.is_some());
        assert!(vel_storage.is_some());

        // Each storage should contain the entity
        assert!(pos_storage.unwrap().contains(entity));
        assert!(health_storage.unwrap().contains(entity));
        assert!(vel_storage.unwrap().contains(entity));
    }

    #[test]
    fn test_get_storage_mut_returns_same_storage() {
        let mut world = World::new();

        // Get storage multiple times
        let storage1 = world.get_storage_mut::<Position>();
        let type_name1 = storage1.component_type_name();

        // Verify we can get immutable reference after mutable
        let storage_ref = world.get_storage::<Position>();
        assert!(storage_ref.is_some());
        assert_eq!(storage_ref.unwrap().component_type_name(), type_name1);

        // Get mutable storage again
        let storage2 = world.get_storage_mut::<Position>();
        let type_name2 = storage2.component_type_name();

        // Should be the same storage
        assert_eq!(type_name1, type_name2);
    }

    #[test]
    fn test_storage_operations_with_entity_lifecycle() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Add components to entities
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();

        let storage = world.get_storage::<Position>();
        assert!(storage.is_some());
        let storage = storage.unwrap();

        // Both entities should be in storage
        assert!(storage.contains(entity1));
        assert!(storage.contains(entity2));
        assert_eq!(storage.get(entity1).unwrap().x, 1.0);
        assert_eq!(storage.get(entity2).unwrap().x, 2.0);

        // Delete one entity
        world.delete_entity(entity1);

        // Storage still exists and has both entities (cleanup hasn't run)
        let storage = world.get_storage::<Position>();
        assert!(storage.is_some());
        // Note: entity data might still be in storage until cleanup

        // After cleanup, deleted entity should be removed from storage
        world.cleanup_deleted_entities();

        let storage = world.get_storage::<Position>();
        assert!(storage.is_some());
        let storage = storage.unwrap();

        // Only entity2 should remain
        assert!(!storage.contains(entity1));
        assert!(storage.contains(entity2));
        assert_eq!(storage.get(entity2).unwrap().x, 2.0);
    }

    #[test]
    fn test_storage_isolation_between_component_types() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add position component
        world
            .add_component(entity, Position { x: 5.0, y: 10.0 })
            .unwrap();

        // Position storage should exist, Health storage should not
        assert!(world.get_storage::<Position>().is_some());
        assert!(world.get_storage::<Health>().is_none());

        // Add health component
        world.add_component(entity, Health { value: 75 }).unwrap();

        // Both storages should exist
        assert!(world.get_storage::<Position>().is_some());
        assert!(world.get_storage::<Health>().is_some());

        // Remove position component
        world.remove_component::<Position>(entity);

        // Position storage still exists (empty), Health storage has data
        assert!(world.get_storage::<Position>().is_some());
        assert!(world.get_storage::<Health>().is_some());

        let pos_storage = world.get_storage::<Position>().unwrap();
        let health_storage = world.get_storage::<Health>().unwrap();

        assert!(!pos_storage.contains(entity));
        assert!(health_storage.contains(entity));
        assert_eq!(health_storage.get(entity).unwrap().value, 75);
    }

    #[test]
    fn test_get_storage_mut_with_existing_data() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add component through normal API
        world
            .add_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();

        // Get mutable storage and modify directly
        {
            let storage = world.get_storage_mut::<Position>();
            let pos = storage.get_mut(entity).unwrap();
            pos.x = 10.0;
            pos.y = 20.0;
        }

        // Changes should be reflected
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_storage_type_safety() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add components of different types
        world
            .add_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();
        world.add_component(entity, Health { value: 100 }).unwrap();

        // Each storage should only contain its specific type
        let pos_storage = world.get_storage::<Position>().unwrap();
        let health_storage = world.get_storage::<Health>().unwrap();

        // Verify type names are different
        assert!(pos_storage.component_type_name().contains("Position"));
        assert!(health_storage.component_type_name().contains("Health"));
        assert_ne!(
            pos_storage.component_type_name(),
            health_storage.component_type_name()
        );

        // Verify data integrity
        assert_eq!(pos_storage.get(entity).unwrap().x, 1.0);
        assert_eq!(health_storage.get(entity).unwrap().value, 100);
    }

    #[test]
    fn test_storage_persistence_across_operations() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Add component to first entity
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();

        // Storage should exist
        assert!(world.get_storage::<Position>().is_some());

        // Add component to second entity
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();

        // Same storage should be used
        let storage = world.get_storage::<Position>().unwrap();
        assert!(storage.contains(entity1));
        assert!(storage.contains(entity2));

        // Remove component from first entity
        world.remove_component::<Position>(entity1);

        // Storage still exists with second entity
        let storage = world.get_storage::<Position>().unwrap();
        assert!(!storage.contains(entity1));
        assert!(storage.contains(entity2));

        // Remove component from second entity
        world.remove_component::<Position>(entity2);

        // Storage still exists but is empty
        let storage = world.get_storage::<Position>().unwrap();
        assert!(!storage.contains(entity1));
        assert!(!storage.contains(entity2));
    }

    #[test]
    fn test_multiple_worlds_separate_storages() {
        let mut world1 = World::new();
        let mut world2 = World::new();

        let entity1 = world1.spawn_entity();
        let entity2 = world2.spawn_entity();

        // Add same component type to both worlds
        world1
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world2
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();

        // Each world should have its own storage
        let storage1 = world1.get_storage::<Position>().unwrap();
        let storage2 = world2.get_storage::<Position>().unwrap();

        // Storages should be independent
        assert!(storage1.contains(entity1));
        assert!(!storage1.contains(entity2));
        assert!(!storage2.contains(entity1));
        assert!(storage2.contains(entity2));

        assert_eq!(storage1.get(entity1).unwrap().x, 1.0);
        assert_eq!(storage2.get(entity2).unwrap().x, 2.0);
    }

    #[test]
    fn test_storage_after_world_operations() {
        let mut world = World::new();

        // Initial state - no storage
        assert!(world.get_storage::<Position>().is_none());

        // Create entity and add component
        let entity = world.spawn_entity();
        world
            .add_component(entity, Position { x: 5.0, y: 5.0 })
            .unwrap();

        // Storage should exist
        let storage = world.get_storage::<Position>();
        assert!(storage.is_some());
        assert!(storage.unwrap().contains(entity));

        // Update component
        world
            .update_component::<Position, _>(entity, |mut pos| {
                pos.x += 1.0;
                pos
            })
            .unwrap();

        // Storage should still exist with updated data
        let storage = world.get_storage::<Position>();
        assert!(storage.is_some());
        assert_eq!(storage.unwrap().get(entity).unwrap().x, 6.0);

        // Replace component
        world.replace_component(entity, Position { x: 10.0, y: 10.0 });

        // Storage should still exist with replaced data
        let storage = world.get_storage::<Position>();
        assert!(storage.is_some());
        assert_eq!(storage.unwrap().get(entity).unwrap().x, 10.0);

        // Delete entity and cleanup
        world.delete_entity(entity);
        world.cleanup_deleted_entities();

        // Storage should still exist but be empty
        let storage = world.get_storage::<Position>();
        assert!(storage.is_some());
        assert!(!storage.unwrap().contains(entity));
    }
}
