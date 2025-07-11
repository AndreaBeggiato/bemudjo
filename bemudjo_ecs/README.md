# Bemudjo ECS

A fast and flexible Entity Component System (ECS) library built in Rust, designed for game development with performance and type safety in mind.

> ⚠️ **Active Development**: This library is currently in active development. APIs may change and some features are still being implemented. Use with caution in production environments.

## 🚀 Features

- **Type-safe Components**: Leverage Rust's type system for safe component management
- **Ephemeral Components**: Revolutionary event system replacement using temporary components
- **Efficient Queries**: Powerful query system with set-based filtering for optimal performance
- **System Scheduling**: Built-in system scheduler for organized game logic execution
- **Memory Efficient**: Optimized storage with deferred cleanup and batch operations
- **Game-Optimized**: Designed for real-time game development patterns
- **Zero-Copy Queries**: Efficient iteration without unnecessary allocations
- **Exact Size Hints**: Queries provide precise entity counts for optimal memory allocation

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

// Query with filtering - uses efficient set operations
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
Systems contain the game logic that processes entities with specific components. A system should be efficient and use queries to iterate over relevant entities.

```rust
struct MovementSystem;

impl System for MovementSystem {
    fn run(&self, world: &mut World) {
        let mut updates = Vec::new();

        // Query for entities that have both a `Position` and a `Velocity` component.
        // This is much more efficient than iterating through all entities.
        let query = Query::<Position>::new().with::<Velocity>();
        for (entity, pos) in query.iter(world) {
            // We get the velocity separately. Since the query ensures it exists, we can unwrap.
            let vel = world.get_component::<Velocity>(entity).unwrap();
            updates.push((
                entity,
                Position {
                    x: pos.x + vel.x,
                    y: pos.y + vel.y,
                },
            ));
        }

        // Apply all the updates in a separate loop.
        // This avoids borrowing `world` mutably while iterating, which is a good practice.
        for (entity, new_pos) in updates {
            world.replace_component(entity, new_pos).unwrap();
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

for (entity, health) in combat_entities.iter(&world) {
    if health.current <= 0 {
        world.add_component(entity, Destroyed).unwrap();
    }
}
```

### Ephemeral Components - Event System Replacement

Bemudjo ECS includes **ephemeral components** - a superior alternative to traditional event systems for inter-system communication. Ephemeral components are temporary, frame-lifetime components that exist only for the duration of a single tick and are automatically cleaned up by the scheduler.

#### Why Ephemeral Components?

Traditional event systems have several drawbacks:
- **Complex lifecycle management**: Events need to be created, queued, dispatched, and cleaned up manually
- **Type safety issues**: Event dispatching often uses dynamic typing or string-based event names
- **Performance overhead**: Event queues and dispatchers add indirection and allocation costs
- **Temporal coupling**: Systems must coordinate on when events are processed and cleared

Ephemeral components solve these problems by leveraging the existing ECS infrastructure:

```rust
// Define ephemeral components just like regular components
#[derive(Clone, Debug, PartialEq)]
struct DamageEvent {
    amount: u32,
    source: String,
}
impl Component for DamageEvent {}

#[derive(Clone, Debug, PartialEq)]
struct MovementEvent {
    dx: f32,
    dy: f32,
}
impl Component for MovementEvent {}

// Systems can create ephemeral components for communication
struct CombatSystem;
impl System for CombatSystem {
    fn run(&self, world: &mut World) {
        // Detect combat and create damage events
        for (entity, position) in Query::<Position>::new().iter(world) {
            if position.x > 50.0 { // In combat zone
                world.add_ephemeral_component(entity, DamageEvent {
                    amount: 10,
                    source: "combat".to_string(),
                }).ok();
            }
        }
    }
}

struct HealthSystem;
impl System for HealthSystem {
    fn run(&self, world: &mut World) {
        // Process damage events
        for entity in world.entities().cloned().collect::<Vec<_>>() {
            if let Some(damage) = world.get_ephemeral_component::<DamageEvent>(entity) {
                if let Some(health) = world.get_component::<Health>(entity) {
                    let new_health = health.current.saturating_sub(damage.amount);
                    world.replace_component(entity, Health {
                        current: new_health,
                        max: health.max,
                    });
                }
            }
        }
    }
}
```

#### Ephemeral Component Lifecycle

1. **Creation**: Systems add ephemeral components during any system phase (`before_run`, `run`, `after_run`)
2. **Access**: Ephemeral components persist across all system phases within the same tick
3. **Querying**: Use special ephemeral queries to iterate over entities with ephemeral components
4. **Automatic Cleanup**: The scheduler automatically removes all ephemeral components at the end of each tick

```rust
// Query ephemeral components specifically
let damage_query = Query::<DamageEvent>::new();
for (entity, damage_event) in damage_query.iter_ephemeral(&world) {
    println!("Entity took {} damage from {}", damage_event.amount, damage_event.source);
}

// Check if an entity has an ephemeral component
if world.has_ephemeral_component::<MovementEvent>(entity) {
    let movement = world.get_ephemeral_component::<MovementEvent>(entity).unwrap();
    // Process movement...
}
```

#### Benefits of Ephemeral Components

- **Type Safety**: Full compile-time type checking, no runtime type errors
- **Performance**: Zero-allocation queries, direct component access, no event queue overhead
- **Simplicity**: No manual lifecycle management, automatic cleanup
- **ECS Native**: Uses the same patterns as regular components, familiar API
- **Debugging**: Easy to inspect in debuggers, clear data ownership
- **Flexibility**: Can attach multiple ephemeral component types to the same entity

#### Common Use Cases

- **Damage/Healing Events**: Health modifications from various sources
- **Input Events**: Player actions that need to be processed by multiple systems
- **Collision Events**: Physics collision data that multiple systems need to react to
- **Animation Triggers**: Start/stop animation events
- **Audio Events**: Sound effect triggers
- **UI Events**: Interface state changes

The scheduler automatically handles cleanup, so ephemeral components never leak between frames, providing a clean and efficient event-like system that's fully integrated with the ECS architecture.

## 🔧 Advanced Usage

### System Scheduling
Use the built-in `SequentialSystemScheduler` to organize and run your systems in a defined order. The scheduler uses a **builder pattern**: you add all your systems first, and then call `build()` to finalize the execution order and dependency checks.

Once built, the scheduler is locked and no more systems can be added.

```rust
use bemudjo_ecs::SequentialSystemScheduler;

// 1. Create a new scheduler
let mut scheduler = SequentialSystemScheduler::new();

// 2. Add all your systems
scheduler.add_system(Box::new(MovementSystem)).unwrap();
scheduler.add_system(Box::new(CombatSystem)).unwrap();
scheduler.add_system(Box::new(RenderSystem)).unwrap();

// 3. Build the scheduler to resolve dependencies and lock it
scheduler.build();

// 4. Run all systems in order for each game tick
scheduler.run(&mut world);
```

### System Dependencies

You can define dependencies between systems to ensure they run in the correct order. For example, you can make sure the `MovementSystem` runs before the `CollisionSystem`.

```rust
use bemudjo_ecs::{System, SystemDependencies, World};

struct CollisionSystem;
impl System for CollisionSystem {
    fn run(&self, world: &mut World) {
        // ...
    }
}

impl SystemDependencies for CollisionSystem {
    fn dependencies(&self) -> Vec<Box<dyn System>> {
        vec![Box::new(MovementSystem)]
    }
}
```

### Performance Optimization

#### Query Optimization
The query system uses efficient set operations for filtering:

```rust
// Queries automatically use optimized set operations
let complex_query = Query::<Position>::new()
    .with::<Velocity>()
    .with::<Health>()
    .without::<Dead>();

// This uses efficient set intersection and difference operations
// instead of per-entity filtering
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
- Ephemeral component lifecycle and integration
- Query system performance
- System execution patterns
- Edge cases and error handling
- Integration scenarios

## 📈 Performance Characteristics

- **Entity Creation**: O(1) with atomic counter
- **Component Addition**: O(1) average case with HashMap storage
- **Query Iteration**: O(size_of_smallest_set) with efficient set operations for filtering
- **Memory Usage**: Minimal overhead with component-specific storage pools
- **Size Hints**: Exact entity counts for optimal memory allocation

### Benchmarks
The library is optimized for typical game development scenarios:
- 1000+ active entities
- 10,000+ total game objects (players, projectiles, items, effects)
- Real-time frame processing
- Efficient batch operations for world updates
- Set-based query filtering eliminates per-entity condition checking

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

- [bemudjo](../) - A MUD game server built using this ECS
- [bemudjo_server_telnet](../bemudjo_server_telnet) - Example telnet server implementation

---

*Built with ❤️ in Rust*
