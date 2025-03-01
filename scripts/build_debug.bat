@echo on

cargo build --target i686-pc-windows-msvc -p dll
cargo build --bin desktop
