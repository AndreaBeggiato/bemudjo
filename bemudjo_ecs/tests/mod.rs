//! Integration Test Organization
//!
//! This module organizes integration tests into focused areas:
//!
//! - `core/`: Core ECS functionality (entities, components, world operations)
//! - `systems/`: System execution, scheduling, and interactions
//! - `queries/`: Query system performance and complex filtering
//! - `resources/`: Resource management and sharing between systems
//! - `performance/`: Stress testing and benchmarks
//! - `scenarios/`: Realistic game scenarios and edge cases

pub mod core;
pub mod performance;
pub mod queries;
pub mod resources;
pub mod scenarios;
pub mod systems;
