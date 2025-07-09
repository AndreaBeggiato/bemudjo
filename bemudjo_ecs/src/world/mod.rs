use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::{AnyStorage, Entity};

mod components;
mod entities;
mod ephemeral_component;
mod resources;
mod storage;

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
    resource_entity: Entity, // we want to store here all the resources (global state, e.g Time component)
    entities: HashSet<Entity>,
    soft_deleted_entities: HashSet<Entity>,
    component_storages: HashMap<TypeId, Box<dyn AnyStorage>>,
    ephemeral_component_storages: HashMap<TypeId, Box<dyn AnyStorage>>,
}

impl World {
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
            resource_entity: Entity::new(),
            entities: HashSet::new(),
            soft_deleted_entities: HashSet::new(),
            component_storages: HashMap::new(),
            ephemeral_component_storages: HashMap::new(),
        }
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

    #[test]
    fn test_world_default_and_new() {
        let world1 = World::new();
        let world2 = World::default();

        assert_eq!(world1.entities().count(), 0);
        assert_eq!(world2.entities().count(), 0);
    }

    #[test]
    fn test_world_integration() {
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

        let mut world = World::new();

        // Test full workflow: spawn -> add components -> operations -> cleanup
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Add different component combinations
        world
            .add_component(entity1, Position { x: 1.0, y: 2.0 })
            .unwrap();
        world.add_component(entity1, Health { value: 100 }).unwrap();
        world
            .add_component(entity2, Position { x: 3.0, y: 4.0 })
            .unwrap();

        // Verify state
        assert_eq!(world.entities().count(), 2);
        assert!(world.has_component::<Position>(entity1));
        assert!(world.has_component::<Health>(entity1));
        assert!(world.has_component::<Position>(entity2));
        assert!(!world.has_component::<Health>(entity2));

        // Update and replace operations
        world
            .update_component::<Health, _>(entity1, |mut h| {
                h.value -= 25;
                h
            })
            .unwrap();

        let old_pos = world.replace_component(entity2, Position { x: 5.0, y: 6.0 });
        assert_eq!(old_pos, Some(Position { x: 3.0, y: 4.0 }));

        // Delete one entity
        world.delete_entity(entity1);
        assert_eq!(world.entities().count(), 1);
        assert!(!world.has_component::<Position>(entity1));
        assert!(!world.has_component::<Health>(entity1));
        assert!(world.has_component::<Position>(entity2));

        // Cleanup and verify final state
        world.cleanup_deleted_entities();
        assert_eq!(world.entities().count(), 1);
        assert_eq!(world.get_component::<Position>(entity2).unwrap().x, 5.0);
    }
}
