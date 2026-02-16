# DLL

The DLL crate is the injectable DLL that hooks into the Asheron's Call client.

## Technologies

- [dll-syringe](https://github.com/OpenByteDev/dll-syringe/) - DLL injection
- [retour-rs](https://github.com/Hpmason/retour-rs) - Function hooking
- [egui](https://www.egui.rs) - User interface

## Features

- Hooks Asheron's Call client functions
- Chat interception
- Network packet capture
- In-game UI rendering

## Platform

- ⚠️ **Windows only** - Builds as 32-bit (i686-pc-windows-msvc)
