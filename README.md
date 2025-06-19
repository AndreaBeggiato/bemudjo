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
A fast and flexible Entity Component System (ECS) library designed for game development. See the [ECS README](bemudjo_ecs/README.md) for detailed documentation and examples.

**Status**: 🚧 In active development

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

## 🏗️ Architecture

The project uses a custom ECS (Entity Component System) implementation for efficient game logic. For detailed information about the ECS architecture, see the [ECS documentation](bemudjo_ecs/README.md).

Key benefits:
- Flexible game object composition
- Efficient batch processing
- Easy feature addition and modification
- Clear separation of data and logic

---

*Built with ❤️ in Rust*
