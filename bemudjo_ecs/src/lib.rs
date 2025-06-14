//! # Bemudjo ECS
//!
//! A flexible Entity-Component-System (ECS) library for game development,
//! specifically designed for MUD (Multi-User Dungeon) games.
//!
//! ## Core Concepts
//!
//! - **Entity**: A unique identifier that represents a game object
//! - **Component**: Pure data that defines attributes of entities
//! - **System**: Logic that processes entities with specific components (coming soon)
//!
//! ## Quick Start
//!
//! ```rust
//! use bemudjo_ecs::{Entity, Component, ComponentStorage, HashMapComponentStorage};
//!
//! // Define a component
//! #[derive(Debug, Clone)]
//! struct Health {
//!     current: u32,
//!     max: u32,
//! }
//!
//! impl Component for Health {}
//!
//! // Create entities and storage
//! let player = Entity::new();
//! let monster = Entity::new();
//! let mut health_storage = HashMapComponentStorage::<Health>::new();
//!
//! // Add components to entities
//! health_storage.insert_or_update(&player, Health { current: 100, max: 100 });
//! health_storage.insert_or_update(&monster, Health { current: 50, max: 50 });
//!
//! // Query components
//! if let Some(player_health) = health_storage.get(&player) {
//!     println!("Player health: {}/{}", player_health.current, player_health.max);
//! }
//! ```
//!
//! ## Features
//!
//! - **Type-safe**: Components are strongly typed
//! - **Flexible storage**: Pluggable storage backends via traits
//! - **Error handling**: Proper error types instead of panics
//! - **Thread-safe entities**: Atomic ID generation
//! - **Zero-cost abstractions**: Efficient runtime performance

pub mod component;
pub mod entity;

pub use entity::Entity;

pub use component::Component;
pub use component::ComponentError;
pub use component::ComponentStorage;
pub use component::HashMapComponentStorage;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
