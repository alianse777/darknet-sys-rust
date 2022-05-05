use anyhow::{format_err, Result};
use std::fs;
use std::{
    env,
    path::{Path, PathBuf},
};

const DARKNET_SRC_ENV: &'static str = "DARKNET_SRC";
const DARKNET_INCLUDE_PATH_ENV: &'static str = "DARKNET_INCLUDE_PATH";
const CUDA_PATH_ENV: &'static str = "CUDA_PATH";
const CUDA_ARCHITECTURES_ENV: &'static str = "CUDA_ARCHITECTURES";

lazy_static::lazy_static! {
    static ref BINDINGS_SRC_PATH: PathBuf = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR")).join("src").join("bindings.rs");
    static ref BINDINGS_TARGET_PATH: PathBuf = PathBuf::from(env::var("OUT_DIR").expect("Failed to get OUT_DIR")).join("bindings.rs");
    static ref LIBRARY_PATH: PathBuf = PathBuf::from(env::var("OUT_DIR").expect("Failed to get OUT_DIR")).join("darknet");
}

// Recursively copy directory
// Ref: https://stackoverflow.com/a/60406693
// Modified to remove target files right before copying to circumvent permission problems.
fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));
    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();
    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();
        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }
        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        // Some `git` files are created with write protection, so replacing them
                        // directly can fail with a permissions error. Remove the destination file
                        // first. Ignore any errors, only the fs::copy() call is critical.
                        fs::remove_file(&dest_path).ok();
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }
    Ok(())
}

// Guess the cmake profile using the rule defined in the link.
// https://docs.rs/cmake/0.1.42/src/cmake/lib.rs.html#475-536
fn guess_cmake_profile() -> &'static str {
    // Determine Rust's profile, optimization level, and debug info:
    #[derive(PartialEq)]
    enum RustProfile {
        Debug,
        Release,
    }
    #[derive(PartialEq, Debug)]
    enum OptLevel {
        Debug,
        Release,
        Size,
    }

    let rust_profile = match env::var("PROFILE").unwrap().as_str() {
        "debug" => RustProfile::Debug,
        "release" | "bench" => RustProfile::Release,
        unknown => {
            eprintln!(
                "Warning: unknown Rust profile={}; defaulting to a release build.",
                unknown
            );
            RustProfile::Release
        }
    };

    let opt_level = match env::var("OPT_LEVEL").unwrap().as_str() {
        "0" => OptLevel::Debug,
        "1" | "2" | "3" => OptLevel::Release,
        "s" | "z" => OptLevel::Size,
        unknown => {
            let default_opt_level = match rust_profile {
                RustProfile::Debug => OptLevel::Debug,
                RustProfile::Release => OptLevel::Release,
            };
            eprintln!(
                "Warning: unknown opt-level={}; defaulting to a {:?} build.",
                unknown, default_opt_level
            );
            default_opt_level
        }
    };

    let debug_info: bool = match env::var("DEBUG").unwrap().as_str() {
        "false" => false,
        "true" => true,
        unknown => {
            eprintln!("Warning: unknown debug={}; defaulting to `true`.", unknown);
            true
        }
    };

    match (opt_level, debug_info) {
        (OptLevel::Debug, _) => "Debug",
        (OptLevel::Release, false) => "Release",
        (OptLevel::Release, true) => "RelWithDebInfo",
        (OptLevel::Size, _) => "MinSizeRel",
    }
}

fn gen_bindings<P>(include_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    bindgen::Builder::default()
        .header(
            include_path
                .as_ref()
                .join("darknet.h")
                .to_str()
                .ok_or_else(|| format_err!("cannot create path to darknet.h"))?,
        )
        .generate()
        .map_err(|_| format_err!("failed to generate bindings"))?
        .write_to_file(&*BINDINGS_TARGET_PATH)?;
    Ok(())
}

fn is_dynamic() -> bool {
    return cfg!(feature = "dylib");
}

fn is_cuda_enabled() -> bool {
    cfg!(feature = "enable-cuda")
}

fn is_cudnn_enabled() -> bool {
    cfg!(feature = "enable-cudnn")
}

fn is_opencv_enabled() -> bool {
    cfg!(feature = "enable-opencv")
}

fn is_openmp_enabled() -> bool {
    cfg!(feature = "enable-openmp")
}

fn build_with_cmake<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let link = if is_dynamic() { "dylib" } else { "static" };
    let path = path.as_ref();
    copy(path, LIBRARY_PATH.as_path())?;
    let path = LIBRARY_PATH.as_path();

    let mut config = cmake::Config::new(path);
    config
        .uses_cxx11()
        .define("BUILD_SHARED_LIBS", if is_dynamic() { "ON" } else { "OFF" })
        .define("ENABLE_CUDA", if is_cuda_enabled() { "ON" } else { "OFF" })
        .define(
            "ENABLE_CUDNN",
            if is_cudnn_enabled() { "ON" } else { "OFF" },
        )
        .define(
            "ENABLE_OPENCV",
            if is_opencv_enabled() { "ON" } else { "OFF" },
        )
        .define(
            "CUDA_ARCHITECTURES",
            env::var_os(CUDA_ARCHITECTURES_ENV).unwrap_or_else(|| "Auto".into()),
        );
    if is_openmp_enabled() {
        // https://cmake.org/cmake/help/latest/variable/CMAKE_REQUIRE_FIND_PACKAGE_PackageName.html
        config.define("CMAKE_REQUIRE_FIND_PACKAGE_OpenMP", "ON");
    } else {
        // https://cmake.org/cmake/help/latest/variable/CMAKE_DISABLE_FIND_PACKAGE_PackageName.html
        config.define("CMAKE_DISABLE_FIND_PACKAGE_OpenMP", "ON");
    }
    let dst = config.build();

    // link to darknet
    println!("cargo:rustc-link-search={}", dst.join("build").display());
    match guess_cmake_profile() {
        "Debug" => println!("cargo:rustc-link-lib={}=darknetd", link),
        _ => println!("cargo:rustc-link-lib={}=darknet", link),
    }

    // link dependent libraries if linking to static library
    if !is_dynamic() {
        if cfg!(target_os = "macos") {
            if is_openmp_enabled() {
                println!("cargo:rustc-link-lib=omp");
            }
            println!("cargo:rustc-link-lib=c++");
        } else {
            if is_openmp_enabled() {
                println!("cargo:rustc-link-lib=gomp");
            }
            println!("cargo:rustc-link-lib=stdc++");
        }
        if is_cuda_enabled() {
            println!("cargo:rustc-link-lib=cudart");
            println!("cargo:rustc-link-lib=cublas");
            println!("cargo:rustc-link-lib=curand");
            let cuda_lib_path = env::var_os(CUDA_PATH_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| "/opt/cuda".into())
                .join("lib64");
            println!("cargo:rustc-link-search={}", cuda_lib_path.display());
        }
        if is_cudnn_enabled() {
            println!("cargo:rustc-link-lib=cudnn");
        }
    }

    gen_bindings(path.join("include"))?;

    Ok(())
}

fn build_runtime() -> Result<()> {
    if cfg!(feature = "buildtime-bindgen") {
        let include_path = env::var_os(DARKNET_INCLUDE_PATH_ENV)
            .map(|value| PathBuf::from(value))
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("darknet")
                    .join("include")
            });
        gen_bindings(include_path)?;
    } else {
        fs::copy(&*BINDINGS_SRC_PATH, &*BINDINGS_TARGET_PATH)
            .expect("Failed to copy bindings.rs to OUT_DIR");
    }

    Ok(())
}

fn build_from_source() -> Result<()> {
    let src_dir: PathBuf = match env::var_os(DARKNET_SRC_ENV) {
        Some(src) => src.into(),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("darknet"),
    };
    build_with_cmake(src_dir)?;

    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-env-changed={}", DARKNET_SRC_ENV);
    println!("cargo:rerun-if-env-changed={}", DARKNET_INCLUDE_PATH_ENV);
    println!("cargo:rerun-if-env-changed={}", CUDA_PATH_ENV);
    println!("cargo:rerun-if-env-changed={}", CUDA_ARCHITECTURES_ENV);
    println!(
        "cargo:rerun-if-env-changed={}",
        BINDINGS_TARGET_PATH.display()
    );
    if cfg!(feature = "docs-rs") {
        return Ok(());
    }
    // build from source by default
    if cfg!(feature = "runtime") {
        build_runtime()?;
    } else {
        build_from_source()?;
    }
    Ok(())
}
