# Architecture

Alembic is managed as a single Cargo Workspace with six crates:

- **cli** - Command-line interface (native architecture)
- **desktop** - Cross-platform egui desktop application (native architecture)
- **tui** - Terminal UI (native architecture)
- **cork** - DLL injector utility (32-bit Windows only)
- **dll** - The injectable DLL (32-bit Windows only)
- **libalembic** - Common and shared functionality

This section covers the architecture and design of each component.
