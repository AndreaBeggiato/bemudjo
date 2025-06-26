pub mod component;
pub mod entity;
pub mod query;
pub mod sequential_system_scheduler;
pub mod system;
pub mod world;

// Re-export commonly used types
pub use component::{Component, ComponentError};
pub use entity::Entity;
pub use query::{Query, QueryIter};
pub use sequential_system_scheduler::SequentialSystemScheduler;
pub use system::System;
pub use world::World;

// Re-export internal types that advanced users might need
#[doc(hidden)]
pub use component::{AnyStorage, ComponentStorage, HashMapComponentStorage};
