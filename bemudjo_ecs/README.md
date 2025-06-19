# Bemudjo ECS

A fast and flexible Entity Component System (ECS) library built in Rust, designed for game development with performance and type safety in mind.

> ⚠️ **Active Development**: This library is currently in active development. APIs may change and some features are still being implemented. Use with caution in production environments.

## 🚀 Features

- **Type-safe Components**: Leverage Rust's type system for safe component management
- **Flexible Queries**: Powerful query system with filtering and optimization hints
- **System Scheduling**: Built-in system scheduler for organized game logic execution
- **Memory Efficient**: Optimized storage with deferred cleanup and batch operations
- **Game-Optimized**: Designed for real-time game development patterns
- **Zero-Copy Queries**: Efficient iteration without unnecessary allocations

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bemudjo_ecs = "0.1"
```

## 🎮 Quick Start

```rust
use bemudjo_ecs::{Component, World, Query, System};

// Define your components
#[derive(Clone, Debug, PartialEq)]
struct Position { x: f32, y: f32 }
impl Component for Position {}

#[derive(Clone, Debug, PartialEq)]
struct Velocity { x: f32, y: f32 }
impl Component for Velocity {}

#[derive(Clone, Debug, PartialEq)]
struct Health { current: u32, max: u32 }
impl Component for Health {}

// Create a world and spawn entities
let mut world = World::new();

let player = world.spawn_entity();
world.add_component(player, Position { x: 0.0, y: 0.0 }).unwrap();
world.add_component(player, Health { current: 100, max: 100 }).unwrap();

let enemy = world.spawn_entity();
world.add_component(enemy, Position { x: 10.0, y: 10.0 }).unwrap();
world.add_component(enemy, Velocity { x: -1.0, y: 0.0 }).unwrap();

// Query entities with specific components
let query = Query::<Position>::new();
for (entity, position) in query.iter(&world) {
    println!("Entity {:?} at ({}, {})", entity, position.x, position.y);
}

// Query with filtering
let moving_entities = Query::<Position>::new()
    .with::<Velocity>(); // Only entities with both Position and Velocity

for (entity, position) in moving_entities.iter(&world) {
    println!("Moving entity {:?} at ({}, {})", entity, position.x, position.y);
}
```

## 🏗️ Core Concepts

### Entities
Entities are unique identifiers for game objects. They're lightweight handles that tie components together:

```rust
let player = world.spawn_entity();
let projectile = world.spawn_entity();
let pickup = world.spawn_entity();
```

### Components
Components are data structures that can be attached to entities. Any type implementing the `Component` trait can be used:

```rust
#[derive(Clone, Debug, PartialEq)]
struct GameStats {
    level: u32,
    experience: u64,
    score: u32,
}
impl Component for GameStats {}

world.add_component(player, GameStats {
    level: 15,
    experience: 12500,
    score: 98750,
}).unwrap();
```

### Systems
Systems contain the game logic that processes entities with specific components:

```rust
struct MovementSystem;

impl System for MovementSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if let (Some(pos), Some(vel)) = (
                world.get_component::<Position>(entity),
                world.get_component::<Velocity>(entity)
            ) {
                let new_pos = Position {
                    x: pos.x + vel.x,
                    y: pos.y + vel.y,
                };
                world.replace_component(entity, new_pos);
            }
        }
    }
}
```

### Queries
Queries provide an efficient way to iterate over entities with specific component combinations:

```rust
// Basic query
let positions = Query::<Position>::new();

// Query with additional requirements
let combat_entities = Query::<Health>::new()
    .with::<Position>()        // Must have Position
    .without::<Invulnerable>();    // Must not have Invulnerable component

// Optimized query with probability hints
let rare_entities = Query::<Collectible>::new()
    .with::<Rare>()
    .with_probability(0.01);   // Hint: only 1% of entities match

for (entity, health) in combat_entities.iter(&world) {
    if health.current <= 0 {
        world.add_component(entity, Destroyed).unwrap();
    }
}
```

## 🔧 Advanced Usage

### System Scheduling
Use the built-in scheduler to organize your game systems:

```rust
use bemudjo_ecs::SequentialSystemScheduler;

let mut scheduler = SequentialSystemScheduler::new();
scheduler.add_system(Box::new(MovementSystem));
scheduler.add_system(Box::new(CombatSystem));
scheduler.add_system(Box::new(RenderSystem));

// Run all systems in order
scheduler.run(&mut world);
```

### Performance Optimization

#### Query Optimization
Use probability hints for better performance on large game worlds:

```rust
// If you know only ~5% of entities have a rare component
let rare_query = Query::<RareComponent>::new()
    .with_probability(0.05);
```

#### Batch Operations
Process multiple entities efficiently:

```rust
// Collect entities first, then process
let entities: Vec<_> = world.entities().cloned().collect();
for entity in entities {
    // Process each entity
}
```

#### Deferred Cleanup
The ECS automatically handles cleanup of deleted entities and components efficiently.

## 🎮 Common Game Patterns

### Spatial Systems
```rust
#[derive(Clone, Debug, PartialEq)]
struct Transform { x: f32, y: f32, rotation: f32 }
impl Component for Transform {}

#[derive(Clone, Debug, PartialEq)]
struct Collider { radius: f32 }
impl Component for Collider {}

// Find all entities within a certain area
let spatial_query = Query::<Transform>::new()
    .with::<Collider>();

for (entity, transform) in spatial_query.iter(&world) {
    // Check collision boundaries, update spatial partitioning, etc.
}
```

### Animation and Rendering
```rust
#[derive(Clone, Debug, PartialEq)]
struct Sprite { texture_id: u32, frame: u32 }
impl Component for Sprite {}

#[derive(Clone, Debug, PartialEq)]
struct Animation {
    frames: Vec<u32>,
    current_frame: usize,
    timer: f32,
    speed: f32
}
impl Component for Animation {}

// Animation system
struct AnimationSystem;
impl System for AnimationSystem {
    fn run(&self, world: &mut World) {
        // Update animation frames, handle sprite changes, etc.
        // Implementation details...
    }
}
```

### Inventory and Items
```rust
#[derive(Clone, Debug, PartialEq)]
struct Item { name: String, value: u32 }
impl Component for Item {}

#[derive(Clone, Debug, PartialEq)]
struct Inventory { items: Vec<Entity>, capacity: u32 }
impl Component for Inventory {}

#[derive(Clone, Debug, PartialEq)]
struct Collectible { points: u32 }
impl Component for Collectible {}
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

The library includes comprehensive tests covering:
- Core ECS functionality
- Query system performance
- System execution patterns
- Edge cases and error handling
- Integration scenarios

## 📈 Performance Characteristics

- **Entity Creation**: O(1) with atomic counter
- **Component Addition**: O(1) average case with HashMap storage
- **Query Iteration**: O(n) where n is the number of matching entities
- **Memory Usage**: Minimal overhead with component-specific storage pools

### Benchmarks
The library is optimized for typical game development scenarios:
- 1000+ active entities
- 10,000+ total game objects (players, projectiles, items, effects)
- Real-time frame processing
- Efficient batch operations for world updates

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes with comprehensive tests
4. Ensure all tests pass: `cargo test`
5. Run clippy: `cargo clippy -- -D warnings`
6. Format code: `cargo fmt`
7. Commit your changes: `git commit -m 'Add amazing feature'`
8. Push to the branch: `git push origin feature/amazing-feature`
9. Open a Pull Request

## 📄 License

This project is dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../LICENSE-MIT))

at your option.

## 🔗 Related Projects

- [bemudjo](https://github.com/your-username/bemudjo) - A MUD game server built using this ECS
- [bemudjo_server_telnet](../bemudjo_server_telnet) - Example telnet server implementation

---

*Built with ❤️ in Rust*
