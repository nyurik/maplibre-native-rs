# MapLibre-native-rs

[![GitHub](https://img.shields.io/badge/github-nyurik/maplibre--native--rs-8da0cb?logo=github)](https://github.com/nyurik/maplibre-native-rs)
[![crates.io version](https://img.shields.io/crates/v/maplibre_native)](https://crates.io/crates/maplibre_native)
[![docs.rs](https://img.shields.io/docsrs/maplibre_native)](https://docs.rs/maplibre_native)
[![crates.io license](https://img.shields.io/crates/l/maplibre_native)](https://github.com/nyurik/maplibre-native-rs/blob/main/LICENSE-APACHE)
[![CI build](https://github.com/nyurik/maplibre-native-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nyurik/maplibre-native-rs/actions)

Rust bindings to the [MapLibre Native](https://github.com/maplibre/maplibre-native) map rendering engine.

## Usage

In order to compile, you must have the following dependencies (linux). No other system has been tested yet (PRs welcome). See the `.github/workflows/ci.yml` for the full list of dependencies.

* `ccache`
* `CMake` + `Ninja`

### Apt Packages
* `build-esential`
* `libcurl4-openssl-dev`
* `libuv1-dev`
* `libjpeg-dev`
* `libpng-dev`
* `libglfw3-dev`
* `libwebp-dev`
* `libopengl0`
* `mesa-vulkan-drivers`

## Development

* This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`.
  Install it with `cargo install just`.
* To get a list of available commands, run `just`.
* To run tests, use `just test`.

### Compiling MapLibre Native

This crate relies on the MapLibre Native library, which is compiled as part of the build process:
* if the `MLN_FROM_SOURCE` environment variable is set, the build script will compile the native library from that dir.
* if this repo has been cloned, it will contain the `/maplibre-native` submodule, which will be used to compile the library. Make sure to run `git submodule update --init --recursive` to fetch the submodule.
* if there is no `/maplibre-native` submodule, the build script assumes it is being run as a dependency, and will try to download the source into the `OUT_DIR`.  Note that the first might take significant time to download and compile.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
