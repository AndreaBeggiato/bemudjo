use std::sync::atomic::{AtomicU64, Ordering};

/// A unique identifier for entities in the ECS system.
///
/// Entities represent game objects like players, monsters, items, or rooms.
/// Each entity is guaranteed to be unique and should only be created through
/// the [`World::spawn_entity()`](crate::World::spawn_entity) method.
///
/// # Examples
///
/// ```
/// use bemudjo_ecs::World;
///
/// let mut world = World::new();
/// let player = world.spawn_entity();
/// let monster = world.spawn_entity();
///
/// assert_ne!(player, monster);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    id: u64,
}

static CURRENT_ID: AtomicU64 = AtomicU64::new(0);

impl Entity {
    /// Creates a new unique entity.
    ///
    /// This method is internal to the crate and should only be called by [`World::spawn_entity()`].
    /// Users should create entities through the World interface to ensure proper tracking.
    ///
    /// [`World::spawn_entity()`]: crate::World::spawn_entity
    #[allow(clippy::new_without_default)]
    pub(crate) fn new() -> Entity {
        Entity {
            id: CURRENT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_should_be_unique() {
        let entity_1 = Entity::new();
        let entity_2 = Entity::new();

        assert_ne!(entity_1, entity_2);
    }

    #[test]
    fn test_entity_should_be_equal_to_themself() {
        let entity = Entity::new();

        assert_eq!(entity, entity);
    }
}
