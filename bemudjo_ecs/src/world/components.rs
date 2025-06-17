use crate::{Component, ComponentError, ComponentStorage};

use super::World;

impl World {
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
        entity: crate::Entity,
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
    pub fn get_component<T: Component>(&self, entity: crate::Entity) -> Option<&T> {
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
    pub fn update_component<T, F>(
        &mut self,
        entity: crate::Entity,
        f: F,
    ) -> Result<T, ComponentError>
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
        entity: crate::Entity,
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
    pub fn has_component<T: Component>(&self, entity: crate::Entity) -> bool {
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
    pub fn remove_component<T: Component>(&mut self, entity: crate::Entity) -> Option<T> {
        if !self.is_entity_active(entity) {
            return None;
        }

        self.get_storage_mut::<T>().remove(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Component, ComponentError};

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
    fn test_add_component_success() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let position = Position { x: 10.0, y: 20.0 };
        let result = world.add_component(entity, position.clone());

        assert!(result.is_ok());
        assert!(world.has_component::<Position>(entity));
        assert_eq!(world.get_component::<Position>(entity), Some(&position));
    }

    #[test]
    fn test_add_component_already_exists() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add component first time - should succeed
        let result = world.add_component(entity, Position { x: 1.0, y: 1.0 });
        assert!(result.is_ok());

        // Add same component type again - should fail
        let result = world.add_component(entity, Position { x: 2.0, y: 2.0 });
        assert!(matches!(
            result,
            Err(ComponentError::ComponentAlreadyExists)
        ));
    }

    #[test]
    fn test_add_component_invalid_entity() {
        let mut world = World::new();
        let mut other_world = World::new();
        let other_entity = other_world.spawn_entity();

        // Try to add component to entity from different world
        let result = world.add_component(other_entity, Position { x: 1.0, y: 1.0 });
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));
    }

    #[test]
    fn test_add_component_deleted_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world.delete_entity(entity);

        let result = world.add_component(entity, Position { x: 1.0, y: 1.0 });
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));
    }

    #[test]
    fn test_get_component_success() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let position = Position { x: 5.0, y: 15.0 };
        world.add_component(entity, position.clone()).unwrap();

        let retrieved = world.get_component::<Position>(entity);
        assert_eq!(retrieved, Some(&position));
    }

    #[test]
    fn test_get_component_not_exists() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let result = world.get_component::<Position>(entity);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_component_invalid_entity() {
        let world = World::new();
        let mut other_world = World::new();
        let other_entity = other_world.spawn_entity();

        let result = world.get_component::<Position>(other_entity);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_component_deleted_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world.delete_entity(entity);

        let result = world.get_component::<Position>(entity);
        assert_eq!(result, None);
    }

    #[test]
    fn test_update_component_success() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world.add_component(entity, Health { value: 100 }).unwrap();

        let result = world.update_component::<Health, _>(entity, |mut health| {
            health.value -= 25;
            health
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 75);
        assert_eq!(world.get_component::<Health>(entity).unwrap().value, 75);
    }

    #[test]
    fn test_update_component_not_exists() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let result = world.update_component::<Health, _>(entity, |health| health);
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));
    }

    #[test]
    fn test_update_component_invalid_entity() {
        let mut world = World::new();
        let mut other_world = World::new();
        let other_entity = other_world.spawn_entity();

        let result = world.update_component::<Health, _>(other_entity, |health| health);
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));
    }

    #[test]
    fn test_update_component_deleted_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world.add_component(entity, Health { value: 100 }).unwrap();
        world.delete_entity(entity);

        let result = world.update_component::<Health, _>(entity, |health| health);
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));
    }

    #[test]
    fn test_replace_component_existing() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let old_position = Position { x: 1.0, y: 1.0 };
        world.add_component(entity, old_position.clone()).unwrap();

        let new_position = Position { x: 2.0, y: 2.0 };
        let result = world.replace_component(entity, new_position.clone());

        assert_eq!(result, Some(old_position));
        assert_eq!(world.get_component::<Position>(entity), Some(&new_position));
    }

    #[test]
    fn test_replace_component_not_existing() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let position = Position { x: 1.0, y: 1.0 };
        let result = world.replace_component(entity, position.clone());

        assert_eq!(result, None);
        assert_eq!(world.get_component::<Position>(entity), Some(&position));
    }

    #[test]
    fn test_replace_component_invalid_entity() {
        let mut world = World::new();
        let mut other_world = World::new();
        let other_entity = other_world.spawn_entity();

        let result = world.replace_component(other_entity, Position { x: 1.0, y: 1.0 });
        assert_eq!(result, None);
    }

    #[test]
    fn test_replace_component_deleted_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world.delete_entity(entity);

        let result = world.replace_component(entity, Position { x: 2.0, y: 2.0 });
        assert_eq!(result, None);
    }

    #[test]
    fn test_has_component_exists() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        assert!(!world.has_component::<Position>(entity));

        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        assert!(world.has_component::<Position>(entity));
    }

    #[test]
    fn test_has_component_multiple_types() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world.add_component(entity, Health { value: 100 }).unwrap();

        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Health>(entity));
        assert!(!world.has_component::<Velocity>(entity));
    }

    #[test]
    fn test_has_component_invalid_entity() {
        let world = World::new();
        let mut other_world = World::new();
        let other_entity = other_world.spawn_entity();

        assert!(!world.has_component::<Position>(other_entity));
    }

    #[test]
    fn test_has_component_deleted_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        assert!(world.has_component::<Position>(entity));

        world.delete_entity(entity);
        assert!(!world.has_component::<Position>(entity));
    }

    #[test]
    fn test_remove_component_success() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let position = Position { x: 10.0, y: 20.0 };
        world.add_component(entity, position.clone()).unwrap();

        assert!(world.has_component::<Position>(entity));

        let removed = world.remove_component::<Position>(entity);
        assert_eq!(removed, Some(position));
        assert!(!world.has_component::<Position>(entity));
    }

    #[test]
    fn test_remove_component_not_exists() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let result = world.remove_component::<Position>(entity);
        assert_eq!(result, None);
    }

    #[test]
    fn test_remove_component_invalid_entity() {
        let mut world = World::new();
        let mut other_world = World::new();
        let other_entity = other_world.spawn_entity();

        let result = world.remove_component::<Position>(other_entity);
        assert_eq!(result, None);
    }

    #[test]
    fn test_remove_component_deleted_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world.delete_entity(entity);

        let result = world.remove_component::<Position>(entity);
        assert_eq!(result, None);
    }

    #[test]
    fn test_component_lifecycle_integration() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Start with no components
        assert!(!world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));

        // Add position
        world
            .add_component(entity, Position { x: 1.0, y: 2.0 })
            .unwrap();
        assert!(world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));

        // Add health
        world.add_component(entity, Health { value: 100 }).unwrap();
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Health>(entity));

        // Update health
        world
            .update_component::<Health, _>(entity, |mut h| {
                h.value -= 50;
                h
            })
            .unwrap();
        assert_eq!(world.get_component::<Health>(entity).unwrap().value, 50);

        // Replace position
        let old_pos = world.replace_component(entity, Position { x: 5.0, y: 10.0 });
        assert_eq!(old_pos, Some(Position { x: 1.0, y: 2.0 }));
        assert_eq!(world.get_component::<Position>(entity).unwrap().x, 5.0);

        // Remove health
        let removed_health = world.remove_component::<Health>(entity);
        assert_eq!(removed_health, Some(Health { value: 50 }));
        assert!(world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));

        // Remove position
        world.remove_component::<Position>(entity);
        assert!(!world.has_component::<Position>(entity));
        assert!(!world.has_component::<Health>(entity));
    }

    #[test]
    fn test_multiple_entities_same_component_type() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();
        let entity3 = world.spawn_entity();

        // Add same component type to multiple entities
        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();
        world
            .add_component(entity3, Position { x: 3.0, y: 3.0 })
            .unwrap();

        // Verify each entity has its own component data
        assert_eq!(world.get_component::<Position>(entity1).unwrap().x, 1.0);
        assert_eq!(world.get_component::<Position>(entity2).unwrap().x, 2.0);
        assert_eq!(world.get_component::<Position>(entity3).unwrap().x, 3.0);

        // Remove from middle entity
        world.remove_component::<Position>(entity2);
        assert!(world.has_component::<Position>(entity1));
        assert!(!world.has_component::<Position>(entity2));
        assert!(world.has_component::<Position>(entity3));
    }

    #[test]
    fn test_component_operations_after_cleanup() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add component, delete entity, cleanup
        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world.delete_entity(entity);
        world.cleanup_deleted_entities();

        // All operations should safely return None/false/error
        assert!(!world.has_component::<Position>(entity));
        assert_eq!(world.get_component::<Position>(entity), None);
        assert_eq!(world.remove_component::<Position>(entity), None);
        assert_eq!(
            world.replace_component(entity, Position { x: 2.0, y: 2.0 }),
            None
        );

        let result = world.add_component(entity, Position { x: 3.0, y: 3.0 });
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));

        let result = world.update_component::<Position, _>(entity, |pos| pos);
        assert!(matches!(result, Err(ComponentError::ComponentNotFound)));
    }
}
