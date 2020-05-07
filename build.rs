use failure::{format_err, Fallible};
use std::{
    env,
    path::{Path, PathBuf},
};

const DARKNET_SRC_ENV: &'static str = "DARKNET_SRC";
const DARKNET_INCLUDE_PATH_ENV: &'static str = "DARKNET_INCLUDE_PATH";

lazy_static::lazy_static! {
    static ref BINDINGS_TARGET_PATH: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("bindings.rs");
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

fn build_with_cmake<P>(path: P) -> Fallible<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let dst = cmake::Config::new(path).build();
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").display()
    );
    println!("cargo:rustc-link-lib=darkd");

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
