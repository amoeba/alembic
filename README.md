# Alembic

![Alembic Logo](./crates/desktop/assets/logo.png)

Tiny demonstration of using the [dll-syringe](https://github.com/OpenByteDev/dll-syringe/) and [retour-rs](https://github.com/Hpmason/retour-rs) to hook Asheron's Call client functions and [egui](https://www.egui.rs) to give all of that a UI.

Built for my curiosity.
This quickly got very out of hand.

## Features

- [x] Desktop GUI written in [egui](https://www.egui.rs)
- [x] CLI for launching and injecting from the command line
- [x] Settings system for persisting
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
cargo build --target i686-pc-windows-msvc -p dll
cargo run --bin desktop
```

This will start the desktop GUI which you can then use to launch a game client and inject Alembic into it.

## Contributing

I can use all the help I can get, please feel issues or reach out to me directly before sending in pull requests though.

Some possible ways to contribute are:

- I was learning Rust while I wrote most of this so if you see things that could be improved, please let me know
- File issues with ideas
- Implement more hooks or client functions
