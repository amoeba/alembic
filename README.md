# Alembic

![Alembic Logo](./crates/desktop/assets/logo.png)

Minimal demo of using the [dll-syringe](https://github.com/OpenByteDev/dll-syringe/) and [retour-rs](https://github.com/Hpmason/retour-rs) to hook Asheron's Call client functions.
Built for my curiosity.

This quickly got very out of hand.

## Plan

- [ ] Hooks: Build out a minimal set of hooks, e.g.,
  - [ ] Chat
  - [ ] Network send/recv
  - [ ] DirectX (i.e., draw textures into the 3d frame)
- [x] GUI: Create a GUI or CLI to handle injection

## Status

- Hooks: Some of these work but it needs more polish and more hooks
- UI (mostly GUI): There's a very basic UI but a lot needs to be done

## Features

- [x] Desktop GUI written in egui
- [x] Very basic CLI for launching and injecting
- [x] Settings for both GUI and CLI
- [x] RPC system for communicating between the game client and the GUI/CLI

## Future Plans

- Improved network handling (i.e., convert/reassmble packets into game messages)
- Plugin system, e.g.,
  - Discord relay
  - Web server

## Building

The project is divided into subcrates,

- `desktop`: Desktop GUI (written in egui)
- `cli`: Alternative option to the GUI
- `dll`: The actual DLL that gets injected
- `libalembic`: Common or shared functionality

If you just want to get started, run:

```sh
cargo run --bin desktop
```

This will start the desktop GUI which you can then use to launch a game client and inject Alembic into it.

## Contributing

I can use all the help I can get, please feel issues or reach out to me directly before sending in pull requests though.

Some possible ways to contribute are:

- I was learning Rust while I wrote most of this so if you see things that could be improved, please let me know
- File issues with ideas
- Implement more hooks or client functions
