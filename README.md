
# Nog

## Status

I have very litte free time in October 2020, so the development will slow down until at least November. Hopefully I'll have more free time this November and onwards.

(I currently have to work a lot)

The current focus is on these two PRs:

* [Cross platform foundation](https://github.com/TimUntersberger/nog/pull/165)
* [Tile layout algorithm refactor](https://github.com/TimUntersberger/nog/pull/164)

## [WIP] Documentation

https://timuntersberger.github.io/nog

## Download

In almost all cases the [development](https://github.com/TimUntersberger/nog/releases/tag/development-release) release is the way to go.

## Development

### Create Executable

```
$env:NOG_VERSION="<version>"
cargo build --release
./rcedit ./target/release/nog.exe --set-icon ./assets/logo.ico
```
