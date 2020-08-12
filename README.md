
# Nog

## Documentation

https://timuntersberger.github.io/nog

## Development

### Create installer

```
cargo build --release
./rcedit ./target/release/wwm.exe --set-icon ./logo.ico
cargo wix init --force
cargo wix --bin-path "C:\Program Files (x86)\WiX Toolset v3.11\bin" --no-build
```
