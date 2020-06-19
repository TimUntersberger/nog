cargo wix init --force
rcedit ./target/release/wwm.exe --set-icon "./logo.ico"
cargo wix --bin-path "C:\Program Files (x86)\WiX Toolset v3.11\bin"