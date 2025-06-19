pub mod component;
pub mod entity;
pub mod query;
pub mod system;
pub mod world;

// Re-export commonly used types
pub use component::{Component, ComponentError};
pub use entity::Entity;
pub use query::{Query, QueryIter};
pub use system::{SequentialSystemScheduler, System};
pub use world::World;

// Re-export internal types that advanced users might need
#[doc(hidden)]
pub use component::{AnyStorage, ComponentStorage, HashMapComponentStorage};
