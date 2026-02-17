# Introduction

Alembic is a cross-platform GUI and CLI Asheron's Call launcher supporting both [Decal](https://decaldev.com) and my own minimal Decal reimplementation.

## Features

- **Desktop GUI** written in [egui](https://www.egui.rs)
  - Complete Servers and Accounts management UI
  - Shows ingame chat
  - Shows game client network activity
  - Cross-platform support (fully works on Windows)
- **CLI** for launching and injecting from the command line
- **Settings system** for persisting configuration
- **RPC system** for communicating between the game client and the GUI/CLI
- **Client Hooks**
  - Chat hooks
  - Networking hooks
- **NSIS-based installer**

## Current Status

| Component | Windows | Linux | macOS |
|-----------|---------|-------|-------|
| CLI Launching | ✅ | ✅ | ✅ |
| CLI Decal | ✅ | ❌ | ❌ |
| CLI Alembic | ❌ | ❌ | ❌ |
| Desktop Launching | ✅ | ✅ | ✅ |
| Desktop Decal | ❌ | ❌ | ❌ |
| Desktop Alembic | ✅ | ❌ | ❌ |

See the [Alembic Walkthrough](https://www.youtube.com/watch?v=Q9_YcRT3qpg) for a video overview.
