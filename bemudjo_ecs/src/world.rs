use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::{
    AnyStorage, Component, ComponentError, ComponentStorage, Entity, HashMapComponentStorage,
};

/// The central World container that manages entities and components.
///
/// The World provides a clean API for entity and component management, automatically
/// handling performance optimizations like deferred cleanup internally.
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
/// world.add_component(entity, Position { x: 10.0, y: 20.0 }).unwrap();
/// assert!(world.has_component::<Position>(entity));
///
/// world.delete_entity(entity);
/// assert!(!world.has_component::<Position>(entity));
/// ```
pub struct World {
    entities: HashSet<Entity>,
    soft_deleted_entities: HashSet<Entity>,
    component_storages: HashMap<TypeId, Box<dyn AnyStorage>>,
}

impl World {
    fn get_storage<T: Component>(&self) -> Option<&HashMapComponentStorage<T>> {
        let type_id = TypeId::of::<T>();

        self.component_storages
            .get(&type_id)
            .and_then(|any_storage| {
                any_storage
                    .as_any()
                    .downcast_ref::<HashMapComponentStorage<T>>()
            })
    }

    fn get_storage_mut<T: Component>(&mut self) -> &mut HashMapComponentStorage<T> {
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

    fn is_entity_active(&self, entity: Entity) -> bool {
        self.entities.contains(&entity) && !self.soft_deleted_entities.contains(&entity)
    }

    /// Creates a new empty World.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::World;
    ///
    /// let world = World::new();
    /// assert_eq!(world.entities().count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            entities: HashSet::new(),
            soft_deleted_entities: HashSet::new(),
            component_storages: HashMap::new(),
        }
    }

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

    /// Adds a component to an entity.
    ///
    /// If the entity already has a component of this type, the operation will fail
    /// with `ComponentError::ComponentAlreadyExists`. If the entity doesn't exist
    /// or has been deleted, it will fail with `ComponentError::ComponentNotFound`.
    ///
    /// # Parameters
    /// * `entity` - The entity to add the component to
    /// * `component` - The component instance to add
    ///
    /// # Returns
    /// * `Ok(())` if the component was successfully added
    /// * `Err(ComponentError)` if the operation failed
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
    /// let result = world.add_component(entity, Position { x: 10.0, y: 20.0 });
    /// assert!(result.is_ok());
    ///
    /// // Adding the same component type again fails
    /// let result = world.add_component(entity, Position { x: 5.0, y: 5.0 });
    /// assert!(result.is_err());
    /// ```
    pub fn add_component<T: Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), ComponentError> {
        if !self.is_entity_active(entity) {
            return Err(ComponentError::ComponentNotFound);
        }

        let storage = self.get_storage_mut::<T>();
        storage.insert(entity, component)
    }

    /// Gets a reference to a component attached to an entity.
    ///
    /// Returns `None` if the entity doesn't exist, has been deleted, or doesn't
    /// have a component of the specified type.
    ///
    /// # Parameters
    /// * `entity` - The entity to get the component from
    ///
    /// # Returns
    /// * `Some(&T)` if the component exists
    /// * `None` if the component doesn't exist or entity is invalid
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
    /// assert_eq!(world.get_component::<Position>(entity), None);
    ///
    /// world.add_component(entity, Position { x: 10.0, y: 20.0 }).unwrap();
    /// let position = world.get_component::<Position>(entity).unwrap();
    /// assert_eq!(position.x, 10.0);
    /// ```
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.is_entity_active(entity) {
            return None;
        }

        self.get_storage::<T>()?.get(entity)
    }

    /// Updates a component using a functional transformation.
    ///
    /// This method provides immutable component updates by taking the current component,
    /// applying a transformation function, and storing the result. This approach is
    /// safe for multi-process environments and ensures data consistency.
    ///
    /// # Type Parameters
    /// * `T` - The component type, which must implement `Component + Clone`
    /// * `F` - The transformation function type
    ///
    /// # Parameters
    /// * `entity` - The entity whose component to update
    /// * `f` - A function that takes the current component and returns the new component
    ///
    /// # Returns
    /// * `Ok(T)` - The new component value after update
    /// * `Err(ComponentError::ComponentNotFound)` - If entity or component doesn't exist
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Health { value: u32 }
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    ///
    /// world.add_component(entity, Health { value: 100 }).unwrap();
    ///
    /// // Damage the entity
    /// let new_health = world.update_component::<Health, _>(entity, |mut health| {
    ///     health.value = health.value.saturating_sub(25);
    ///     health
    /// }).unwrap();
    ///
    /// assert_eq!(new_health.value, 75);
    /// assert_eq!(world.get_component::<Health>(entity).unwrap().value, 75);
    /// ```
    pub fn update_component<T, F>(&mut self, entity: Entity, f: F) -> Result<T, ComponentError>
    where
        T: Component + Clone,
        F: FnOnce(T) -> T,
    {
        if !self.is_entity_active(entity) {
            return Err(ComponentError::ComponentNotFound);
        }

        let storage = self.get_storage_mut::<T>();
        match storage.get(entity) {
            Some(old_component) => {
                let new_component = f(old_component.clone());
                storage.insert_or_update(entity, new_component.clone());
                Ok(new_component)
            }
            None => Err(ComponentError::ComponentNotFound),
        }
    }

    /// Replaces a component with a new value, returning the old value if it existed.
    ///
    /// If the entity doesn't have the component type, the new component is added
    /// and `None` is returned. If the entity has been deleted, `None` is returned
    /// and no action is taken.
    ///
    /// # Parameters
    /// * `entity` - The entity whose component to replace
    /// * `component` - The new component value
    ///
    /// # Returns
    /// * `Some(T)` - The previous component value if it existed
    /// * `None` - If no previous component existed or entity is invalid
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
    /// // First replacement on empty entity returns None
    /// let old = world.replace_component(entity, Position { x: 1.0, y: 1.0 });
    /// assert_eq!(old, None);
    ///
    /// // Second replacement returns the old value
    /// let old = world.replace_component(entity, Position { x: 2.0, y: 2.0 });
    /// assert_eq!(old, Some(Position { x: 1.0, y: 1.0 }));
    /// ```
    pub fn replace_component<T: Component + Clone>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Option<T> {
        if !self.is_entity_active(entity) {
            return None;
        }

        let storage = self.get_storage_mut::<T>();
        let old_component = storage.get(entity).cloned();
        storage.insert_or_update(entity, component);
        old_component
    }

    /// Checks if an entity has a specific component type.
    ///
    /// Returns `false` if the entity doesn't exist, has been deleted, or doesn't
    /// have a component of the specified type.
    ///
    /// # Parameters
    /// * `entity` - The entity to check
    ///
    /// # Returns
    /// * `true` if the entity has the component type
    /// * `false` if the entity doesn't have the component or is invalid
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Health { value: u32 }
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    ///
    /// assert!(!world.has_component::<Position>(entity));
    /// assert!(!world.has_component::<Health>(entity));
    ///
    /// world.add_component(entity, Position { x: 1.0, y: 2.0 }).unwrap();
    ///
    /// assert!(world.has_component::<Position>(entity));
    /// assert!(!world.has_component::<Health>(entity));
    /// ```
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        if !self.is_entity_active(entity) {
            return false;
        }

        self.get_storage::<T>()
            .is_some_and(|storage| storage.contains(entity))
    }

    /// Removes a component from an entity and returns it.
    ///
    /// Returns `None` if the entity doesn't exist, has been deleted, or doesn't
    /// have a component of the specified type.
    ///
    /// # Parameters
    /// * `entity` - The entity to remove the component from
    ///
    /// # Returns
    /// * `Some(T)` - The removed component if it existed
    /// * `None` - If the component didn't exist or entity is invalid
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
    /// let position = Position { x: 10.0, y: 20.0 };
    /// world.add_component(entity, position.clone()).unwrap();
    ///
    /// assert!(world.has_component::<Position>(entity));
    ///
    /// let removed = world.remove_component::<Position>(entity);
    /// assert_eq!(removed, Some(position));
    /// assert!(!world.has_component::<Position>(entity));
    /// ```
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        if !self.is_entity_active(entity) {
            return None;
        }

        self.get_storage_mut::<T>().remove(entity)
    }
}

impl Default for World {
    /// Creates a new empty World using the default constructor.
    ///
    /// This is equivalent to calling `World::new()`.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::World;
    ///
    /// let world = World::default();
    /// assert_eq!(world.entities().count(), 0);
    /// ```
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_add_and_get_component() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let position = Position { x: 10.0, y: 20.0 };
        world.add_component(entity, position.clone()).unwrap();

        let retrieved_position = world.get_component::<Position>(entity);
        assert_eq!(retrieved_position, Some(&position));
    }

    #[test]
    fn test_multiple_component_types() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let position = Position { x: 5.0, y: 15.0 };
        let health = Health { value: 100 };

        world.add_component(entity, position.clone()).unwrap();
        world.add_component(entity, health.clone()).unwrap();

        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Health>(entity));

        assert_eq!(world.get_component::<Position>(entity), Some(&position));
        assert_eq!(world.get_component::<Health>(entity), Some(&health));
    }

    #[test]
    fn test_component_mutation() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let health = Health { value: 100 };
        world.add_component(entity, health).unwrap();

        // Use functional update instead of mutable reference
        world
            .update_component::<Health, _>(entity, |mut health| {
                health.value = 50;
                health
            })
            .unwrap();

        assert_eq!(world.get_component::<Health>(entity).unwrap().value, 50);
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let position = Position { x: 1.0, y: 2.0 };
        world.add_component(entity, position.clone()).unwrap();

        let removed = world.remove_component::<Position>(entity);
        assert_eq!(removed, Some(position));

        assert!(!world.has_component::<Position>(entity));
        assert_eq!(world.get_component::<Position>(entity), None);
    }

    #[test]
    fn test_delete_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let position = Position { x: 1.0, y: 2.0 };
        let health = Health { value: 100 };
        world.add_component(entity, position).unwrap();
        world.add_component(entity, health).unwrap();

        // Initially entity has components
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Health>(entity));

        // Delete the entity
        world.delete_entity(entity);

        // Component access should return None/false for deleted entities
        assert!(!world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));
        assert_eq!(world.get_component::<Position>(entity), None);

        // After cleanup, the underlying storage is also cleaned
        world.cleanup_deleted_entities();

        // Still no components
        assert!(!world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));
    }

    #[test]
    fn test_clean_entity_lifecycle() {
        let mut world = World::new();

        // Spawn multiple entities
        let player = world.spawn_entity();
        let monster = world.spawn_entity();
        let item = world.spawn_entity();

        // Add components
        world
            .add_component(player, Position { x: 0.0, y: 0.0 })
            .unwrap();
        world.add_component(player, Health { value: 100 }).unwrap();
        world
            .add_component(monster, Position { x: 10.0, y: 10.0 })
            .unwrap();
        world.add_component(monster, Health { value: 50 }).unwrap();
        world
            .add_component(item, Position { x: 5.0, y: 5.0 })
            .unwrap();

        // Verify all entities have components
        assert!(world.has_component::<Position>(player));
        assert!(world.has_component::<Health>(player));
        assert!(world.has_component::<Position>(monster));
        assert!(world.has_component::<Health>(monster));
        assert!(world.has_component::<Position>(item));

        // Count active entities
        let entity_count = world.entities().count();
        assert_eq!(entity_count, 3);

        // Delete monster
        world.delete_entity(monster);

        // Monster should no longer have components accessible
        assert!(!world.has_component::<Position>(monster));
        assert!(!world.has_component::<Health>(monster));

        // Other entities should still have components
        assert!(world.has_component::<Position>(player));
        assert!(world.has_component::<Health>(player));
        assert!(world.has_component::<Position>(item));

        // Only 2 active entities now
        let active_count = world.entities().count();
        assert_eq!(active_count, 2);

        // Cleanup happens automatically (or when called)
        world.cleanup_deleted_entities();

        // Everything still works the same from user perspective
        assert!(!world.has_component::<Position>(monster));
        assert!(!world.has_component::<Health>(monster));
        assert!(world.has_component::<Position>(player));
        assert!(world.has_component::<Position>(item));
        assert_eq!(world.entities().count(), 2);
    }

    #[test]
    fn test_world_default_and_new() {
        let world1 = World::new();
        let world2 = World::default();

        assert_eq!(world1.entities().count(), 0);
        assert_eq!(world2.entities().count(), 0);
    }

    #[test]
    fn test_empty_world_operations() {
        let mut world = World::new();
        let mut other_world = World::new();
        let other_entity = other_world.spawn_entity(); // Entity from different world

        // All operations on entity from different world should fail gracefully
        assert!(!world.has_component::<Position>(other_entity));
        assert_eq!(world.get_component::<Position>(other_entity), None);
        assert_eq!(world.remove_component::<Position>(other_entity), None);
        assert_eq!(
            world.replace_component(other_entity, Position { x: 1.0, y: 1.0 }),
            None
        );

        let result = world.add_component(other_entity, Position { x: 1.0, y: 1.0 });
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

        let result = world.update_component::<Position, _>(other_entity, |pos| pos);
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

        // Delete entity from different world should be safe
        world.delete_entity(other_entity);
        world.cleanup_deleted_entities();

        assert_eq!(world.entities().count(), 0);
    }

    #[test]
    fn test_component_error_cases() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Adding component to valid entity should work
        let result = world.add_component(entity, Position { x: 1.0, y: 1.0 });
        assert!(result.is_ok());

        // Adding same component type again should fail
        let result = world.add_component(entity, Position { x: 2.0, y: 2.0 });
        assert!(matches!(
            result,
            Err(ComponentError::ComponentAlreadyExists)
        ));

        // Update non-existent component should fail
        let result = world.update_component::<Health, _>(entity, |h| h);
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

        // Delete entity and try operations
        world.delete_entity(entity);

        let result = world.add_component(entity, Health { value: 100 });
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

        let result = world.update_component::<Position, _>(entity, |pos| pos);
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));
    }

    #[test]
    fn test_entity_uniqueness_and_spawning() {
        let mut world = World::new();
        let mut entities = HashSet::new();

        // Spawn many entities and ensure they're all unique
        for _ in 0..1000 {
            let entity = world.spawn_entity();
            assert!(!entities.contains(&entity), "Entity should be unique");
            entities.insert(entity);
        }

        assert_eq!(world.entities().count(), 1000);
        assert_eq!(entities.len(), 1000);
    }

    #[test]
    fn test_multiple_component_storages() {
        let mut world = World::new();

        // Create entities with different component combinations
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        // Entity1: Position only
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();

        // Entity2: Health only
        world.add_component(entity2, Health { value: 100 }).unwrap();

        // Entity3: Both components
        world
            .add_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();
        world.add_component(entity3, Health { value: 200 }).unwrap();

        // Verify component existence
        assert!(world.has_component::<Position>(entity1));
        assert!(!world.has_component::<Health>(entity1));

        assert!(!world.has_component::<Position>(entity2));
        assert!(world.has_component::<Health>(entity2));

        assert!(world.has_component::<Position>(entity3));
        assert!(world.has_component::<Health>(entity3));

        // Verify component values
        assert_eq!(world.get_component::<Position>(entity1).unwrap().x, 1.0);
        assert_eq!(world.get_component::<Health>(entity2).unwrap().value, 100);
        assert_eq!(world.get_component::<Position>(entity3).unwrap().x, 3.0);
        assert_eq!(world.get_component::<Health>(entity3).unwrap().value, 200);
    }

    #[test]
    fn test_update_component_scenarios() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add initial health
        world.add_component(entity, Health { value: 100 }).unwrap();

        // Test successful update
        let new_health = world
            .update_component::<Health, _>(entity, |mut h| {
                h.value += 50;
                h
            })
            .unwrap();
        assert_eq!(new_health.value, 150);
        assert_eq!(world.get_component::<Health>(entity).unwrap().value, 150);

        // Test update that doesn't change anything
        let same_health = world.update_component::<Health, _>(entity, |h| h).unwrap();
        assert_eq!(same_health.value, 150);

        // Test complex update
        let result = world
            .update_component::<Health, _>(entity, |mut h| {
                h.value = h.value.saturating_sub(200); // Should go to 0
                h
            })
            .unwrap();
        assert_eq!(result.value, 0);
        assert_eq!(world.get_component::<Health>(entity).unwrap().value, 0);
    }

    #[test]
    fn test_replace_component_scenarios() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Replace on entity without component should return None
        let old = world.replace_component(entity, Position { x: 1.0, y: 1.0 });
        assert_eq!(old, None);
        assert!(world.has_component::<Position>(entity));

        // Replace existing component should return old value
        let old = world.replace_component(entity, Position { x: 2.0, y: 2.0 });
        assert_eq!(old, Some(Position { x: 1.0, y: 1.0 }));
        assert_eq!(world.get_component::<Position>(entity).unwrap().x, 2.0);

        // Replace after entity deletion should return None
        world.delete_entity(entity);
        let old = world.replace_component(entity, Position { x: 3.0, y: 3.0 });
        assert_eq!(old, None);
    }

    #[test]
    fn test_soft_deletion_edge_cases() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add components
        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world.add_component(entity, Health { value: 100 }).unwrap();

        // Delete entity multiple times (should be safe)
        world.delete_entity(entity);
        world.delete_entity(entity); // Second delete should be safe
        world.delete_entity(entity); // Third delete should be safe

        // Entity should still be inaccessible
        assert!(!world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));

        // Multiple cleanups should be safe
        world.cleanup_deleted_entities();
        world.cleanup_deleted_entities();
        world.cleanup_deleted_entities();

        // Entity should still be inaccessible
        assert!(!world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));
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

        // Spawn new entity (might get same ID due to atomic counter implementation)
        let entity2 = world.spawn_entity();

        // New entity should be clean (no components from previous entity)
        assert!(!world.has_component::<Position>(entity2));

        // Should be able to add components to new entity
        world.add_component(entity2, Health { value: 200 }).unwrap();
        assert!(world.has_component::<Health>(entity2));
    }

    #[test]
    fn test_stress_many_entities() {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Create many entities with components
        for i in 0..100 {
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

            if i % 2 == 0 {
                world
                    .add_component(
                        entity,
                        Health {
                            value: (i * 10) as u32,
                        },
                    )
                    .unwrap();
            }
        }

        assert_eq!(world.entities().count(), 100);

        // Verify all positions are correct
        for (i, &entity) in entities.iter().enumerate() {
            let pos = world.get_component::<Position>(entity).unwrap();
            assert_eq!(pos.x, i as f32);
            assert_eq!(pos.y, (i * 2) as f32);

            if i % 2 == 0 {
                let health = world.get_component::<Health>(entity).unwrap();
                assert_eq!(health.value, (i * 10) as u32);
            } else {
                assert!(!world.has_component::<Health>(entity));
            }
        }

        // Delete half the entities
        for i in (0..entities.len()).step_by(2) {
            world.delete_entity(entities[i]);
        }

        assert_eq!(world.entities().count(), 50);

        // Cleanup and verify
        world.cleanup_deleted_entities();
        assert_eq!(world.entities().count(), 50);

        // Remaining entities should still have correct data
        for (i, &entity) in entities.iter().enumerate() {
            if i % 2 == 0 {
                // These were deleted
                assert!(!world.has_component::<Position>(entity));
                assert!(!world.has_component::<Health>(entity));
            } else {
                // These should still exist
                assert!(world.has_component::<Position>(entity));
                let pos = world.get_component::<Position>(entity).unwrap();
                assert_eq!(pos.x, i as f32);
            }
        }
    }

    #[test]
    fn test_component_operations_ordering() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Test various operation sequences
        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world.add_component(entity, Health { value: 100 }).unwrap();

        // Remove and re-add
        let removed_pos = world.remove_component::<Position>(entity).unwrap();
        assert!(!world.has_component::<Position>(entity));
        assert!(world.has_component::<Health>(entity));

        world.add_component(entity, removed_pos).unwrap();
        assert!(world.has_component::<Position>(entity));

        // Update, then replace, then remove
        world
            .update_component::<Health, _>(entity, |mut h| {
                h.value = 200;
                h
            })
            .unwrap();

        let old_health = world.replace_component(entity, Health { value: 300 });
        assert_eq!(old_health.unwrap().value, 200);

        let final_health = world.remove_component::<Health>(entity);
        assert_eq!(final_health.unwrap().value, 300);
        assert!(!world.has_component::<Health>(entity));
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, PartialEq)]
    struct Name {
        value: String,
    }
    impl Component for Name {}

    #[test]
    fn test_many_component_types() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add many different component types
        world
            .add_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();
        world.add_component(entity, Health { value: 100 }).unwrap();
        world
            .add_component(entity, Velocity { dx: 0.5, dy: -0.3 })
            .unwrap();
        world
            .add_component(
                entity,
                Name {
                    value: "TestEntity".to_string(),
                },
            )
            .unwrap();

        // Verify all components exist
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Health>(entity));
        assert!(world.has_component::<Velocity>(entity));
        assert!(world.has_component::<Name>(entity));

        // Verify component values
        assert_eq!(world.get_component::<Position>(entity).unwrap().x, 1.0);
        assert_eq!(world.get_component::<Health>(entity).unwrap().value, 100);
        assert_eq!(world.get_component::<Velocity>(entity).unwrap().dx, 0.5);
        assert_eq!(
            world.get_component::<Name>(entity).unwrap().value,
            "TestEntity"
        );

        // Remove some components
        world.remove_component::<Velocity>(entity);
        world.remove_component::<Name>(entity);

        // Verify selective removal
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Health>(entity));
        assert!(!world.has_component::<Velocity>(entity));
        assert!(!world.has_component::<Name>(entity));

        // Delete entity should clean up remaining components
        world.delete_entity(entity);
        world.cleanup_deleted_entities();

        assert!(!world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));
    }

    #[test]
    fn test_entities_iterator_consistency() {
        let mut world = World::new();
        let mut spawned_entities = Vec::new();

        // Start with empty iterator
        assert_eq!(world.entities().count(), 0);

        // Add entities one by one and check iterator
        for i in 0..10 {
            let entity = world.spawn_entity();
            spawned_entities.push(entity);
            assert_eq!(world.entities().count(), i + 1);
        }

        // Verify all spawned entities are in iterator
        let iter_entities: HashSet<_> = world.entities().cloned().collect();
        let spawned_set: HashSet<_> = spawned_entities.iter().cloned().collect();
        assert_eq!(iter_entities, spawned_set);

        // Delete some entities and check iterator updates
        world.delete_entity(spawned_entities[0]);
        world.delete_entity(spawned_entities[5]);
        world.delete_entity(spawned_entities[9]);

        assert_eq!(world.entities().count(), 7);

        // Verify deleted entities are not in iterator
        let remaining_entities: HashSet<_> = world.entities().cloned().collect();
        assert!(!remaining_entities.contains(&spawned_entities[0]));
        assert!(!remaining_entities.contains(&spawned_entities[5]));
        assert!(!remaining_entities.contains(&spawned_entities[9]));

        // Verify non-deleted entities are still in iterator
        for (i, &entity) in spawned_entities.iter().enumerate() {
            if i != 0 && i != 5 && i != 9 {
                assert!(remaining_entities.contains(&entity));
            }
        }
    }
}
