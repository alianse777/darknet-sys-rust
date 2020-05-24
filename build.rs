use failure::{format_err, Fallible};
use std::fs;
use std::{
    env,
    path::{Path, PathBuf},
};

const DARKNET_SRC_ENV: &'static str = "DARKNET_SRC";
const DARKNET_INCLUDE_PATH_ENV: &'static str = "DARKNET_INCLUDE_PATH";

lazy_static::lazy_static! {
    static ref BINDINGS_SRC_PATH: PathBuf = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR")).join("src").join("bindings.rs");
    static ref BINDINGS_TARGET_PATH: PathBuf = PathBuf::from(env::var("OUT_DIR").expect("Failed to get OUT_DIR")).join("bindings.rs");
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

fn gen_bindings<P>(include_path: P) -> Fallible<()>
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

fn is_opencv_enabled() -> bool {
    cfg!(feature = "enable-opencv")
}

fn build_with_cmake<P>(path: P) -> Fallible<()>
where
    P: AsRef<Path>,
{
    let link = if is_dynamic() { "dylib" } else { "static" };
    let path = path.as_ref();
    let dst = cmake::Config::new(path)
        .define("BUILD_SHARED_LIBS", if is_dynamic() { "ON" } else { "OFF" })
        .define("ENABLE_CUDA", if is_cuda_enabled() { "ON" } else { "OFF" })
        .define("ENABLE_CUDNN", if is_cuda_enabled() { "ON" } else { "OFF" })
        .define(
            "ENABLE_OPENCV",
            if is_opencv_enabled() { "ON" } else { "OFF" },
        )
        .build();
    println!("cargo:rustc-link-search={}", dst.join("build").display());

    // link to different target under distinct profiles
    match guess_cmake_profile() {
        "Debug" => println!("cargo:rustc-link-lib={}=darkd", link),
        _ => println!("cargo:rustc-link-lib={}=dark", link),
    }
    if !is_dynamic() {
        println!("cargo:rustc-link-lib=gomp");
        println!("cargo:rustc-link-lib=stdc++");
    }

    gen_bindings(path.join("include"))?;

    Ok(())
}

fn build_runtime() -> Fallible<()> {
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

fn build_from_source() -> Fallible<()> {
    let src_dir: PathBuf = match env::var_os(DARKNET_SRC_ENV) {
        Some(src) => src.into(),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("darknet"),
    };
    build_with_cmake(src_dir)?;

    Ok(())
}

fn main() -> Fallible<()> {
    println!("cargo:rerun-if-env-changed={}", DARKNET_SRC_ENV);
    println!("cargo:rerun-if-env-changed={}", DARKNET_INCLUDE_PATH_ENV);
    println!(
        "cargo:rerun-if-env-changed={}",
        BINDINGS_TARGET_PATH.display()
    );

    // build from source by default
    if cfg!(feature = "runtime") {
        build_runtime()?;
    } else {
        build_from_source()?;
    }
    Ok(())
}
