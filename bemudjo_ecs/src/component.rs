use crate::Entity;
use std::any::Any;
use std::collections::HashMap;

/// Marker trait for components.
/// All component types must implement this trait.
pub trait Component: 'static {}

/// Trait for component storage operations on a specific component type.
pub trait ComponentStorage<T: Component> {
    /// Adds a component to an entity.
    fn insert(&mut self, entity: Entity, component: T) -> Result<(), ComponentError>;

    /// Adds a component to an entity, replacing any existing component.
    fn insert_or_update(&mut self, entity: Entity, component: T) -> Option<T>;

    /// Removes a component from an entity.
    fn remove(&mut self, entity: Entity) -> Option<T>;

    /// Gets a reference to a component for an entity.
    fn get(&self, entity: Entity) -> Option<&T>;

    /// Gets a mutable reference to a component for an entity.
    fn get_mut(&mut self, entity: Entity) -> Option<&mut T>;

    /// Checks if an entity has this component.
    fn contains(&self, entity: Entity) -> bool;
}

/// Type-erased storage trait for storing different component types in the same collection.
/// This is the key trait that enables storing different component storages in a HashMap.
pub trait AnyStorage {
    /// Returns a reference to the storage as `&dyn Any` for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Returns a mutable reference to the storage as `&mut dyn Any` for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Removes all components for the given entity from this storage.
    fn remove_entity(&mut self, entity: Entity);

    /// Removes all components from this storage.
    fn clear(&mut self);

    /// Returns the type name of the component this storage handles.
    fn component_type_name(&self) -> &'static str;
}

/// A HashMap-based implementation of ComponentStorage.
#[derive(Debug, Default)]
pub struct HashMapComponentStorage<T: Component> {
    data: HashMap<Entity, T>,
}

impl<T: Component> HashMapComponentStorage<T> {
    /// Creates a new empty storage.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<T: Component> ComponentStorage<T> for HashMapComponentStorage<T> {
    fn insert(&mut self, entity: Entity, component: T) -> Result<(), ComponentError> {
        match self.data.entry(entity) {
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(component);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                Err(ComponentError::ComponentAlreadyExists)
            }
        }
    }

    fn insert_or_update(&mut self, entity: Entity, component: T) -> Option<T> {
        self.data.insert(entity, component)
    }

    fn remove(&mut self, entity: Entity) -> Option<T> {
        self.data.remove(&entity)
    }

    fn get(&self, entity: Entity) -> Option<&T> {
        self.data.get(&entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.data.get_mut(&entity)
    }

    fn contains(&self, entity: Entity) -> bool {
        self.data.contains_key(&entity)
    }
}

impl<T: Component> AnyStorage for HashMapComponentStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.data.remove(&entity);
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn component_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

/// Errors that can occur when working with components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentError {
    /// The component already exists for this entity.
    ComponentAlreadyExists,
    /// The component storage for this type is not registered.
    StorageNotRegistered,
    /// The component does not exist for this entity.
    ComponentNotFound,
}
