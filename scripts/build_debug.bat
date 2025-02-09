@echo on

cargo build --target i686-pc-windows-msvc -p dll --debug
cargo build --target i686-pc-windows-msvc -p desktop --debug
