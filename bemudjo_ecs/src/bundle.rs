use crate::{Entity, World};

/// A collection of components that can be inserted together into the world.
///
/// Implement this trait to define reusable entity archetypes like players,
/// rooms, or mobs.
///
/// # Example
/// ```
/// use bemudjo_ecs::{Bundle, Component, Entity, World};
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Name(String);
/// impl Component for Name {}
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Health { value: u32 }
/// impl Component for Health {}
///
/// struct PlayerBundle {
///     name: Name,
///     health: Health,
/// }
///
/// impl Bundle for PlayerBundle {
///     fn insert_into(self, world: &mut World, entity: Entity) {
///         world.add_component(entity, self.name).unwrap();
///         world.add_component(entity, self.health).unwrap();
///     }
/// }
///
/// let mut world = World::new();
/// let player = world.spawn_with(PlayerBundle {
///     name: Name("Aragorn".to_string()),
///     health: Health { value: 100 },
/// });
///
/// assert!(world.has_component::<Name>(player));
/// assert!(world.has_component::<Health>(player));
/// ```
pub trait Bundle {
    fn insert_into(self, world: &mut World, entity: Entity);
}
