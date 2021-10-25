# darknet-sys: FFI bindings to AlexeyAB's Darknet

[![Crates.io](https://img.shields.io/crates/v/darknet-sys?style=for-the-badge)](https://crates.io/crates/darknet-sys) ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/alianse777/darknet-sys-rust/Rust?style=for-the-badge)

## Usage

To be used with [darknet](https://crates.io/crates/darknet) crate.

If you want to build from this repo, run `git submodule init && git submodule update --recursive` to get all submodules.

## Build

Terms used:

darknet-sys, darknet = Rust wrappers

libdarknet = C/C++ darknet implementation

By default, darknet-sys will compile and link libdarknet statically. You can control the feature flags to change the behavior.

## Cargo Features

- `enable-cuda`: Enable CUDA (expects CUDA 10.x and cuDNN 7.x).
- `enable-cudnn`: Enable cuDNN
- `enable-opencv`: Enable OpenCV.
- `runtime`: Link to libdarknet dynamic library. For example, `libdark.so` on Linux.
- `dylib`: Build dynamic library instead of static
- `buildtime-bindgen`: Generate bindings from libdarknet headers.


### Method 1: Download and build from source (default)

```sh
cargo build
```

You can optionally enable CUDA and OpenCV features. Please read [Build with CUDA](#build-with-cuda) for more info.

```sh
cargo build --features enable-cuda,enable-opencv
```

### Method 2: Build with custom source

If you want to build with custom libdarknet source, point `DARKNET_SRC` environment variable to your source path. It should contain `CMakeLists.txt`.

```sh
export DARKNET_SRC=/path/to/your/darknet/repo
cargo build
```

### Method 3: Link to libdarknet dynamic library

With `runtime` feature, darknet-sys will not compile libdarknet source code and instead links to libdarknet dynamically. If you are using Linux, make sure `libdark.so` is installed on your system.

```sh
cargo build --feature runtime
```

### Re-generate bindings

With `buildtime-bindgen` feature, darknet-sys re-generates bindings from headers. The option is necessary only when darkent is updated or modified.

If you want to use your (possibly modified) header files, point `DARKNET_INCLUDE_PATH` environment variable to your header dir.

### Build with CUDA

Please check that both CUDA 10.x and cuDNN 7.x are installed.

darknet reads `CUDA_PATH` environment variable (which defaults to `/opt/cuda` if not set) and assumes it can find cuda libraries at `${CUDA_PATH}/lib64`.

```sh
export CUDA_PATH=/usr/local/cuda-10.1
cargo build --features enable-cuda
```

You can also set `CUDA_ARCHITECTURES` which is passed to libdarknet's cmake. It defaults to `Auto`, which auto-detects GPU architecture based on card present in the system during build.

## License

MIT license.

## Credits

Huge thanks to [jerry73204](https://github.com/jerry73204)
