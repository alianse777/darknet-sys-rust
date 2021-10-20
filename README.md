Main crate: [darknet](https://crates.io/crates/darknet)

# darknet-sys: FFI bindings to AlexeyAB's Darknet

[![Crates.io](https://img.shields.io/crates/v/darknet-sys?style=for-the-badge)](https://crates.io/crates/darknet-sys) ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/alianse777/darknet-sys-rust/Rust?style=for-the-badge)

## Usage

Get the crate by adding the dependency to your `Cargo.toml`.

```toml
darknet-sys = "0.3"
```

If you clone the repository manually, run `git submodule init && git submodule update --recursive` to get all submodules.

## Cargo Features

- `enable-cuda`: Enable CUDA (expects CUDA 10.x and cuDNN 7.x).
- `enable-cudnn`: Enable cuDNN
- `enable-opencv`: Enable OpenCV.
- `runtime`: Link to darknet dynamic library. For example, `libdark.so` on Linux.
- `dylib`: Build dynamic library instead of static
- `buildtime-bindgen`: Generate bindings from darknet headers.

## Build

By default, darknet-sys compiles and link to darknet statically. You can control the feature flags to change the behavior.

### Method 1: Download and build from source (default)

By default, it builds and links to darknet statically.

```sh
cargo build
```

You can optionally enable CUDA and OpenCV features. Please read [Build with CUDA](#build-with-cuda) section to work to CUDA properly.

```sh
cargo build --features enable-cuda,enable-opencv
```

### Method 2: Build with custom source

If you prefer to build with your darknet source, fill the source directory to the `DARKNET_SRC` environment variable. It expects a `CMakeLists.txt` in that directory.

```sh
export DARKNET_SRC=/path/to/your/darknet/repo
cargo build
```

### Method 3: Link to darknet dynamic library

With `runtime` feature, darknet-sys will not compile the darknet source code and instead links to darknet dynamically. If you are using Linux, make sure `libdark.so` is installed on your system.


```sh
cargo build --feature runtime
```

### Re-generate bindings

With `buildtime-bindgen` feature, darknet-sys re-generates bindings from headers. It guesses the header file paths according to feature flags. The option is necessary only when darkent is updated or modified.

If you prefer to your (possibly modified) header files, fill the header directory to `DARKNET_INCLUDE_PATH` environment variable.

### Build with CUDA

Please check both CUDA 10.x and cuDNN 7.x are installed and versions are correct.

They are not installed to standard paths on most systems. `darknet-sys` build system reads `CUDA_PATH` (which defaults to `/opt/cuda` if not set) and assumes it can find cuda libraries at `${CUDA_PATH}/lib64`.

```sh
export CUDA_PATH=/usr/local/cuda-10.1
cargo build --features enable-cuda
```

## License

MIT license.

## Credits

Huge thanks to [jerry73204](https://github.com/jerry73204)
