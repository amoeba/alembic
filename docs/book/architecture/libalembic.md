# libalembic

The libalembic crate contains common and shared functionality used across all components.

## Key Components

- **RPC System** - Communication between game client and GUI/CLI
  - Built on [tarpc](https://docs.rs/tarpc/)
  - JSON-based message transport
  - TCP networking

- **Settings System** - Persistent configuration management
  - Accounts and servers configuration
  - User preferences

- **Networking** - Network packet handling
  - Capture and filtering
  - Message parsing (work in progress)

## Architecture

The shared library abstracts common concerns so other crates can focus on their specific domain (CLI, Desktop, DLL injection).
