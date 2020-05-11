Main crate: [darknet](https://crates.io/crates/darknet)
# darknet-sys: FFI bindings to AlexeyAB's Darknet

Version 0.2.0 changes:

- Replace the unmaintained pjreddie's darknet to AlexeyAB's darknet fork.
- Users can optionally link at runtime without compiling the source code.
- Configurable source code path and include path.

## Usage

To ensure the git submodules are tracked, please run `git submodule init && git submodule update --recursive` after you clone this repository.

There are two ways to generate the bindings:

- Building from source (default)
- Runtime linking

By default, it builds the darknet from submodule if there is no additional environment variables and features.

### Method 1: Build from source

It is the default behavior. If you would like to build your own source, you can set the `DARKNET_SRC` environment variables to the path of your repository. Note that it expects a `CMakeLists.txt` in your repository.

```sh
export DARKNET_SRC=/path/to/your/darknet/repo # if you would like to build your own source
cargo build
```

### Method 2: Runtime linking

If you prefer not to build the source code and use the generated bindings already built in our repository, add the `runtime` feature to take effect.


```sh
cargo build --feature runtime
```

The `buildtime-bindgen` allows you to re-generate bindings from headers without compiling the source code. By default, it finds the header files from the shipped darknet submodule. If you would like to provide your own header files, please set `DARKNET_INCLUDE_PATH` the environment variable.

```sh
export DARKNET_INCLUDE_PATH=/path/to/your/header/dir
cargo build --feature runtime,buildtime-bindgen
```
Huge thanks to [jerry73204](https://github.com/jerry73204)
