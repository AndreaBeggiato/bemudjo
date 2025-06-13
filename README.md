# Bemudjo MUD

A Multi-User Dungeon (MUD) game server built in Rust, featuring a custom Entity Component System (ECS) architecture.

## 🎮 Project Overview

Bemudjo is a text-based multiplayer adventure game server that allows players to connect via telnet and explore a virtual world. The project is structured as a monorepo containing multiple related crates.

## 📁 Repository Structure

```
bemudjo/
├── bemudjo_ecs/          # Custom ECS library for game logic
├── bemudjo_server_telnet/ # Telnet server for player connections
└── README.md             # This file
```

## 🧩 Crates

### bemudjo_ecs
A custom Entity Component System library designed specifically for MUD games. This library provides:
- Entity management
- Component storage
- System execution
- Query mechanisms

**Status**: 🚧 In development - implementing core ECS functionality

### bemudjo_server_telnet
The main game server that handles:
- TCP connections via telnet
- Player command processing
- Game world management
- Real-time multiplayer interactions

**Status**: ✅ Basic telnet server functional

## 🚀 Getting Started

### Prerequisites
- Rust 1.75+ (we recommend using the latest stable version)
- Git

### Building the Project

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/bemudjo.git
cd bemudjo

# Build all crates
cargo build

# Run the telnet server
cargo run -p bemudjo_server_telnet

# Run ECS tests
cargo test -p bemudjo_ecs
```

### Connecting to the Game

Once the server is running, connect using any telnet client:

```bash
telnet localhost 2323
```

## 🎯 Development Roadmap

The ECS implementation follows a structured development plan tracked through GitHub issues. Key phases include:

- [x] **Phase 0**: Project setup and workspace configuration
- [ ] **Phase 1**: Core ECS structure (entities, components, world)
- [ ] **Phase 2**: Systems framework and queries
- [ ] **Phase 3**: Resources and event handling
- [ ] **Phase 4**: Performance optimization
- [ ] **Phase 5**: MUD-specific integration
- [ ] **Phase 6**: Documentation and release

## 🛠️ Development

### Running Tests

```bash
# Test all crates
cargo test

# Test specific crate
cargo test -p bemudjo_ecs
cargo test -p bemudjo_server_telnet
```

### Code Structure

This project follows Rust best practices:
- **Workspace**: Multiple related crates in one repository
- **ECS Architecture**: Data-oriented design for game logic
- **Async/Await**: Modern concurrent programming for networking
- **Type Safety**: Leveraging Rust's type system for reliable game state

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Commit your changes (`git commit -m 'Add some amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📝 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🎮 Game Commands

Currently available commands:
- `help` - Show available commands
- `look` - Look around the current area
- `say <message>` - Say something to other players
- `quit` - Exit the game

## 🏗️ Architecture Notes

The project uses a custom ECS implementation to handle game logic efficiently:
- **Entities**: Unique identifiers for game objects (players, items, rooms)
- **Components**: Data attached to entities (health, location, inventory)
- **Systems**: Logic that processes entities with specific components

This architecture allows for:
- Flexible game object composition
- Efficient batch processing
- Easy feature addition and modification
- Clear separation of data and logic

---

*Built with ❤️ in Rust*
