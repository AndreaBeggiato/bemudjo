use std::collections::HashMap;

use crate::{Component, ComponentError, ComponentStorage};

use super::World;

impl World {
    /// Adds an ephemeral component to an entity.
    ///
    /// Ephemeral components are temporary components that exist only until the next
    /// cleanup cycle. They are automatically removed when `clean_ephemeral_storage()`
    /// is called, typically by the system scheduler at the end of each frame.
    ///
    /// Unlike regular components, ephemeral components can be added to the same entity
    /// multiple times, replacing the previous value. This makes them ideal for
    /// inter-system communication where multiple systems need to react to the same
    /// temporary state change.
    ///
    /// # Parameters
    /// * `entity` - The entity to add the ephemeral component to
    /// * `component` - The ephemeral component instance to add
    ///
    /// # Returns
    /// * `Ok(())` if the component was successfully added
    /// * `Err(ComponentError::ComponentNotFound)` if the entity doesn't exist or has been deleted
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct DamageReceived { amount: u32, source: String }
    /// impl Component for DamageReceived {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    ///
    /// // Add ephemeral component for other systems to react to
    /// world.add_ephemeral_component(entity, DamageReceived {
    ///     amount: 25,
    ///     source: "fire_spell".to_string()
    /// }).unwrap();
    ///
    /// // Multiple systems can read this ephemeral component
    /// assert!(world.has_ephemeral_component::<DamageReceived>(entity));
    /// ```
    pub fn add_ephemeral_component<T: Component>(
        &mut self,
        entity: crate::Entity,
        component: T,
    ) -> Result<(), ComponentError> {
        if !self.is_entity_active(entity) {
            return Err(ComponentError::ComponentNotFound);
        }

        let entities_in_reverse_index = self.get_or_create_ephemeral_reverse_index::<T>();
        entities_in_reverse_index.insert(entity);

        let storage = self.get_ephemeral_storage_mut::<T>();
        // For ephemeral components, we allow replacement (insert_or_update)
        storage.insert_or_update(entity, component);
        Ok(())
    }

    /// Gets a reference to an ephemeral component attached to an entity.
    ///
    /// Returns `None` if the entity doesn't exist, has been deleted, or doesn't
    /// have an ephemeral component of the specified type.
    ///
    /// # Parameters
    /// * `entity` - The entity to get the ephemeral component from
    ///
    /// # Returns
    /// * `Some(&T)` if the ephemeral component exists
    /// * `None` if the ephemeral component doesn't exist or entity is invalid
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct MovementIntent { direction: f32, speed: f32 }
    /// impl Component for MovementIntent {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    ///
    /// assert_eq!(world.get_ephemeral_component::<MovementIntent>(entity), None);
    ///
    /// world.add_ephemeral_component(entity, MovementIntent { direction: 90.0, speed: 5.0 }).unwrap();
    /// let intent = world.get_ephemeral_component::<MovementIntent>(entity).unwrap();
    /// assert_eq!(intent.direction, 90.0);
    /// ```
    pub fn get_ephemeral_component<T: Component>(&self, entity: crate::Entity) -> Option<&T> {
        if !self.is_entity_active(entity) {
            return None;
        }

        self.get_ephemeral_storage::<T>()?.get(entity)
    }

    /// Checks if an entity has a specific ephemeral component type.
    ///
    /// Returns `false` if the entity doesn't exist, has been deleted, or doesn't
    /// have an ephemeral component of the specified type.
    ///
    /// # Parameters
    /// * `entity` - The entity to check
    ///
    /// # Returns
    /// * `true` if the entity has the ephemeral component
    /// * `false` if the entity doesn't have the ephemeral component or is invalid
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct JumpIntent { force: f32 }
    /// impl Component for JumpIntent {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    ///
    /// assert!(!world.has_ephemeral_component::<JumpIntent>(entity));
    ///
    /// world.add_ephemeral_component(entity, JumpIntent { force: 500.0 }).unwrap();
    /// assert!(world.has_ephemeral_component::<JumpIntent>(entity));
    /// ```
    pub fn has_ephemeral_component<T: Component>(&self, entity: crate::Entity) -> bool {
        if !self.is_entity_active(entity) {
            return false;
        }

        let type_id = std::any::TypeId::of::<T>();
        self.reverse_ephemeral_component_index
            .get(&type_id)
            .map(|entities| entities.contains(&entity))
            .unwrap_or(false)
    }

    /// Clears all ephemeral component storages.
    ///
    /// This implements the "nuclear cleanup" pattern - an O(1) operation that
    /// replaces the entire ephemeral storage HashMap with a new one, letting
    /// Rust's Drop trait handle all memory cleanup automatically.
    ///
    /// This function is typically called by the system scheduler at the end of
    /// each frame to ensure ephemeral components only live for one frame cycle.
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TempEffect { duration: f32 }
    /// impl Component for TempEffect {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_entity();
    ///
    /// world.add_ephemeral_component(entity, TempEffect { duration: 1.0 }).unwrap();
    /// assert!(world.has_ephemeral_component::<TempEffect>(entity));
    ///
    /// // System scheduler calls this at end of frame
    /// world.clean_ephemeral_storage();
    /// assert!(!world.has_ephemeral_component::<TempEffect>(entity));
    /// ```
    pub fn clean_ephemeral_storage(&mut self) {
        // Nuclear cleanup - O(1) operation
        self.ephemeral_component_storages = HashMap::new();
        self.reverse_ephemeral_component_index = HashMap::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Component;

    #[derive(Debug, Clone, PartialEq)]
    struct DamageReceived {
        amount: u32,
        damage_type: String,
        source: crate::Entity,
    }
    impl Component for DamageReceived {}

    #[derive(Debug, Clone, PartialEq)]
    struct MovementIntent {
        direction: f32,
        speed: f32,
    }
    impl Component for MovementIntent {}

    #[derive(Debug, Clone, PartialEq)]
    struct JumpPerformed {
        force: f32,
        timestamp: u64,
    }
    impl Component for JumpPerformed {}

    #[test]
    fn test_add_ephemeral_component() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        let result = world.add_ephemeral_component(
            entity,
            DamageReceived {
                amount: 50,
                damage_type: "fire".to_string(),
                source: entity,
            },
        );

        assert!(result.is_ok());
        assert!(world.has_ephemeral_component::<DamageReceived>(entity));
    }

    #[test]
    fn test_add_ephemeral_component_to_nonexistent_entity() {
        let mut world = World::new();
        let nonexistent_entity = crate::Entity::new();

        let result = world.add_ephemeral_component(
            nonexistent_entity,
            DamageReceived {
                amount: 50,
                damage_type: "fire".to_string(),
                source: nonexistent_entity,
            },
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ComponentError::ComponentNotFound);
    }

    #[test]
    fn test_ephemeral_component_replacement() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add first ephemeral component
        world
            .add_ephemeral_component(
                entity,
                MovementIntent {
                    direction: 0.0,
                    speed: 1.0,
                },
            )
            .unwrap();

        let first_intent = world
            .get_ephemeral_component::<MovementIntent>(entity)
            .unwrap();
        assert_eq!(first_intent.direction, 0.0);

        // Add second ephemeral component (should replace the first)
        world
            .add_ephemeral_component(
                entity,
                MovementIntent {
                    direction: 90.0,
                    speed: 2.0,
                },
            )
            .unwrap();

        let second_intent = world
            .get_ephemeral_component::<MovementIntent>(entity)
            .unwrap();
        assert_eq!(second_intent.direction, 90.0);
        assert_eq!(second_intent.speed, 2.0);
    }

    #[test]
    fn test_get_ephemeral_component() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Initially no ephemeral component
        assert_eq!(
            world.get_ephemeral_component::<MovementIntent>(entity),
            None
        );

        // Add ephemeral component
        world
            .add_ephemeral_component(
                entity,
                MovementIntent {
                    direction: 45.0,
                    speed: 3.0,
                },
            )
            .unwrap();

        let intent = world
            .get_ephemeral_component::<MovementIntent>(entity)
            .unwrap();
        assert_eq!(intent.direction, 45.0);
        assert_eq!(intent.speed, 3.0);
    }

    #[test]
    fn test_has_ephemeral_component() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Initially no ephemeral component
        assert!(!world.has_ephemeral_component::<JumpPerformed>(entity));

        // Add ephemeral component
        world
            .add_ephemeral_component(
                entity,
                JumpPerformed {
                    force: 500.0,
                    timestamp: 12345,
                },
            )
            .unwrap();

        assert!(world.has_ephemeral_component::<JumpPerformed>(entity));
    }

    #[test]
    fn test_clean_ephemeral_storage() {
        let mut world = World::new();
        let entity1 = world.spawn_entity();
        let entity2 = world.spawn_entity();

        // Add multiple ephemeral components to multiple entities
        world
            .add_ephemeral_component(
                entity1,
                DamageReceived {
                    amount: 25,
                    damage_type: "ice".to_string(),
                    source: entity2,
                },
            )
            .unwrap();

        world
            .add_ephemeral_component(
                entity2,
                MovementIntent {
                    direction: 180.0,
                    speed: 4.0,
                },
            )
            .unwrap();

        world
            .add_ephemeral_component(
                entity1,
                JumpPerformed {
                    force: 750.0,
                    timestamp: 67890,
                },
            )
            .unwrap();

        // Verify all ephemeral components exist
        assert!(world.has_ephemeral_component::<DamageReceived>(entity1));
        assert!(world.has_ephemeral_component::<MovementIntent>(entity2));
        assert!(world.has_ephemeral_component::<JumpPerformed>(entity1));

        // Clean ephemeral storage
        world.clean_ephemeral_storage();

        // All ephemeral components should be gone
        assert!(!world.has_ephemeral_component::<DamageReceived>(entity1));
        assert!(!world.has_ephemeral_component::<MovementIntent>(entity2));
        assert!(!world.has_ephemeral_component::<JumpPerformed>(entity1));
    }

    #[test]
    fn test_ephemeral_components_independent_of_regular_components() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add regular component
        #[derive(Debug, Clone, PartialEq)]
        struct Position {
            x: f32,
            y: f32,
        }
        impl Component for Position {}

        world
            .add_component(entity, Position { x: 10.0, y: 20.0 })
            .unwrap();

        // Add ephemeral component with same type
        world
            .add_ephemeral_component(entity, Position { x: 100.0, y: 200.0 })
            .unwrap();

        // Both should exist independently
        let regular_pos = world.get_component::<Position>(entity).unwrap();
        let ephemeral_pos = world.get_ephemeral_component::<Position>(entity).unwrap();

        assert_eq!(regular_pos, &Position { x: 10.0, y: 20.0 });
        assert_eq!(ephemeral_pos, &Position { x: 100.0, y: 200.0 });

        // Clean ephemeral storage
        world.clean_ephemeral_storage();

        // Regular component should remain, ephemeral should be gone
        assert!(world.has_component::<Position>(entity));
        assert!(!world.has_ephemeral_component::<Position>(entity));
    }

    #[test]
    fn test_ephemeral_component_deleted_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        // Add ephemeral component
        world
            .add_ephemeral_component(
                entity,
                MovementIntent {
                    direction: 270.0,
                    speed: 1.5,
                },
            )
            .unwrap();

        assert!(world.has_ephemeral_component::<MovementIntent>(entity));

        // Delete entity
        world.delete_entity(entity);

        // Should not be able to access ephemeral component of deleted entity
        assert!(!world.has_ephemeral_component::<MovementIntent>(entity));
        assert_eq!(
            world.get_ephemeral_component::<MovementIntent>(entity),
            None
        );

        // Should not be able to add ephemeral component to deleted entity
        let result = world.add_ephemeral_component(
            entity,
            MovementIntent {
                direction: 0.0,
                speed: 0.0,
            },
        );
        assert!(result.is_err());
    }
}
