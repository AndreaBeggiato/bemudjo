use std::collections::HashMap;

use crate::Entity;

/// A marker trait for component types in the ECS system.
///
/// Components are pure data structures that represent different aspects
/// of entities, such as position, health, inventory, etc.
///
/// # Examples
///
/// ```
/// use bemudjo_ecs::Component;
///
/// #[derive(Debug, Clone)]
/// struct Health {
///     current: u32,
///     max: u32,
/// }
///
/// impl Component for Health {}
/// ```
pub trait Component {}

/// Errors that can occur during component operations.
#[derive(Debug, PartialEq)]
pub enum ComponentError {
    /// Attempted to insert a component for an entity that already has one.
    ComponentAlreadyExists,
}

/// A storage system for components of a specific type.
///
/// This trait defines the interface for storing and retrieving components
/// associated with entities. Different implementations can provide different
/// storage strategies (HashMap, database, etc.).
///
/// # Type Parameters
///
/// * `T` - The component type that implements [`Component`]
///
/// # Examples
///
/// ```
/// use bemudjo_ecs::{Entity, Component, ComponentStorage, HashMapComponentStorage};
///
/// #[derive(Debug, PartialEq, Clone)]
/// struct Health {
///     hp: u32,
/// }
///
/// impl Component for Health {}
///
/// let mut storage = HashMapComponentStorage::<Health>::new();
/// let entity = Entity::new();
/// let health = Health { hp: 100 };
///
/// // Insert a component
/// storage.insert(&entity, health).unwrap();
///
/// // Retrieve the component
/// let retrieved = storage.get(&entity);
/// assert!(retrieved.is_some());
/// ```
pub trait ComponentStorage<T: Component> {
    /// Inserts a component for an entity, failing if one already exists.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to associate the component with
    /// * `component` - The component data to store
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the component was successfully inserted
    /// * `Err(ComponentError::ComponentAlreadyExists)` if the entity already has this component type
    ///
    /// # Examples
    ///
    /// ```
    /// use bemudjo_ecs::{Entity, Component, ComponentStorage, HashMapComponentStorage, ComponentError};
    ///
    /// #[derive(Debug, PartialEq, Clone)]
    /// struct Health { hp: u32 }
    /// impl Component for Health {}
    ///
    /// let mut storage = HashMapComponentStorage::<Health>::new();
    /// let entity = Entity::new();
    ///
    /// // First insert succeeds
    /// assert!(storage.insert(&entity, Health { hp: 100 }).is_ok());
    ///
    /// // Second insert fails
    /// assert_eq!(
    ///     storage.insert(&entity, Health { hp: 50 }),
    ///     Err(ComponentError::ComponentAlreadyExists)
    /// );
    /// ```
    fn insert(&mut self, entity: &Entity, component: T) -> Result<(), ComponentError>;

    /// Inserts or updates a component for an entity, always succeeding.
    ///
    /// If the entity already has a component of this type, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to associate the component with
    /// * `component` - The component data to store
    ///
    /// # Examples
    ///
    /// ```
    /// use bemudjo_ecs::{Entity, Component, ComponentStorage, HashMapComponentStorage};
    ///
    /// #[derive(Debug, PartialEq, Clone)]
    /// struct Health { hp: u32 }
    /// impl Component for Health {}
    ///
    /// let mut storage = HashMapComponentStorage::<Health>::new();
    /// let entity = Entity::new();
    ///
    /// // Insert initial component
    /// storage.insert_or_update(&entity, Health { hp: 100 });
    ///
    /// // Replace with new value
    /// storage.insert_or_update(&entity, Health { hp: 50 });
    ///
    /// assert_eq!(storage.get(&entity).unwrap().hp, 50);
    /// ```
    fn insert_or_update(&mut self, entity: &Entity, component: T);

    /// Removes a component for an entity, returning it if it existed.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the component from
    ///
    /// # Returns
    ///
    /// * `Some(component)` if the entity had this component type
    /// * `None` if the entity did not have this component type
    ///
    /// # Examples
    ///
    /// ```
    /// use bemudjo_ecs::{Entity, Component, ComponentStorage, HashMapComponentStorage};
    ///
    /// #[derive(Debug, PartialEq, Clone)]
    /// struct Health { hp: u32 }
    /// impl Component for Health {}
    ///
    /// let mut storage = HashMapComponentStorage::<Health>::new();
    /// let entity = Entity::new();
    ///
    /// storage.insert_or_update(&entity, Health { hp: 100 });
    ///
    /// let removed = storage.remove(&entity);
    /// assert_eq!(removed.unwrap().hp, 100);
    ///
    /// // Component no longer exists
    /// assert!(storage.get(&entity).is_none());
    /// ```
    fn remove(&mut self, entity: &Entity) -> Option<T>;

    /// Gets a reference to a component for an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to get the component for
    ///
    /// # Returns
    ///
    /// * `Some(&component)` if the entity has this component type
    /// * `None` if the entity does not have this component type
    ///
    /// # Examples
    ///
    /// ```
    /// use bemudjo_ecs::{Entity, Component, ComponentStorage, HashMapComponentStorage};
    ///
    /// #[derive(Debug, PartialEq, Clone)]
    /// struct Health { hp: u32 }
    /// impl Component for Health {}
    ///
    /// let mut storage = HashMapComponentStorage::<Health>::new();
    /// let entity = Entity::new();
    ///
    /// storage.insert_or_update(&entity, Health { hp: 100 });
    ///
    /// let health = storage.get(&entity);
    /// assert_eq!(health.unwrap().hp, 100);
    /// ```
    fn get(&self, entity: &Entity) -> Option<&T>;
}

/// A HashMap-based implementation of component storage.
///
/// This provides efficient storage and retrieval of components using a HashMap
/// with entities as keys. This is the default implementation provided by the library.
///
/// # Type Parameters
///
/// * `T` - The component type that implements [`Component`]
///
/// # Examples
///
/// ```
/// use bemudjo_ecs::{Entity, Component, ComponentStorage, HashMapComponentStorage};
///
/// #[derive(Debug, PartialEq, Clone)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// impl Component for Position {}
///
/// let mut storage = HashMapComponentStorage::<Position>::new();
/// let entity = Entity::new();
///
/// storage.insert_or_update(&entity, Position { x: 10.0, y: 20.0 });
///
/// let pos = storage.get(&entity).unwrap();
/// assert_eq!(pos.x, 10.0);
/// assert_eq!(pos.y, 20.0);
/// ```
pub struct HashMapComponentStorage<T: Component> {
    hash_map: HashMap<Entity, T>,
}

impl<T: Component> HashMapComponentStorage<T> {
    /// Creates a new empty component storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use bemudjo_ecs::{Component, HashMapComponentStorage};
    ///
    /// #[derive(Debug)]
    /// struct Health { hp: u32 }
    /// impl Component for Health {}
    ///
    /// // Both ways create the same empty storage
    /// let storage1 = HashMapComponentStorage::<Health>::new();
    /// let storage2 = HashMapComponentStorage::<Health>::default();
    /// ```
    pub fn new() -> Self {
        HashMapComponentStorage {
            hash_map: HashMap::new(),
        }
    }
}

impl<T: Component> Default for HashMapComponentStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> ComponentStorage<T> for HashMapComponentStorage<T> {
    fn insert(&mut self, entity: &Entity, component: T) -> Result<(), ComponentError> {
        if self.hash_map.contains_key(entity) {
            return Err(ComponentError::ComponentAlreadyExists);
        }
        self.hash_map.insert(*entity, component);
        Ok(())
    }

    fn insert_or_update(&mut self, entity: &Entity, component: T) {
        self.hash_map.insert(*entity, component);
    }

    fn remove(&mut self, entity: &Entity) -> Option<T> {
        self.hash_map.remove(entity)
    }

    fn get(&self, entity: &Entity) -> Option<&T> {
        self.hash_map.get(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    struct TestComponent {
        pub value: i32,
    }

    impl Component for TestComponent {}

    #[test]
    fn test_hash_map_storage_should_insert_entity_and_retrieve_it() {
        let mut storage = HashMapComponentStorage::<TestComponent>::new();
        let entity = Entity::new();
        let component = TestComponent { value: 42 };

        let result = storage.insert(&entity, component);
        assert!(result.is_ok());

        let retrieved = storage.get(&entity);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 42);
    }

    #[test]
    fn test_insert_should_fail_when_component_already_exists() {
        let mut storage = HashMapComponentStorage::<TestComponent>::new();
        let entity = Entity::new();

        let result1 = storage.insert(&entity, TestComponent { value: 42 });
        assert!(result1.is_ok());

        let result2 = storage.insert(&entity, TestComponent { value: 100 });
        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err(), ComponentError::ComponentAlreadyExists);
    }

    #[test]
    fn test_insert_or_update_should_overwrite_existing_component() {
        let mut storage = HashMapComponentStorage::<TestComponent>::new();
        let entity = Entity::new();

        // Insert first component
        storage
            .insert(&entity, TestComponent { value: 42 })
            .unwrap();

        // Overwrite with new value
        storage.insert_or_update(&entity, TestComponent { value: 100 });

        // Should have the new value
        let retrieved = storage.get(&entity);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 100);
    }

    #[test]
    fn test_remove_should_return_component() {
        let mut storage = HashMapComponentStorage::<TestComponent>::new();
        let entity = Entity::new();
        let component = TestComponent { value: 42 };

        storage.insert_or_update(&entity, component);

        let removed = storage.remove(&entity);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, 42);

        assert!(storage.get(&entity).is_none());
    }

    #[test]
    fn test_default_creates_empty_storage() {
        let storage = HashMapComponentStorage::<TestComponent>::default();
        let entity = Entity::new();

        // Storage should be empty
        assert!(storage.get(&entity).is_none());
    }
}
