use std::sync::atomic::{AtomicU64, Ordering};

/// A unique identifier for entities in the ECS system.
///
/// Entities represent game objects like players, monsters, items, or rooms.
/// Each entity is guaranteed to be unique.
///
/// # Examples
///
/// ```
/// use bemudjo_ecs::Entity;
///
/// let player = Entity::new();
/// let monster = Entity::new();
///
/// assert_ne!(player, monster);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entity {
    id: u64,
}

static CURRENT_ID: AtomicU64 = AtomicU64::new(0);

impl Entity {
    /// Creates a new unique entity.
    ///
    /// # Examples
    ///
    /// ```
    /// use bemudjo_ecs::Entity;
    ///
    /// let entity1 = Entity::new();
    /// let entity2 = Entity::new();
    ///
    /// assert_ne!(entity1, entity2);
    /// ```
    #[allow(clippy::new_without_default)]
    pub fn new() -> Entity {
        Entity {
            id: CURRENT_ID.fetch_add(1, Ordering::SeqCst),
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
