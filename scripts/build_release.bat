@echo on

cargo build --target i686-pc-windows-msvc -p dll --release
cargo build --bin desktop --release
