use bindgen;
use cc;
use copy_dir;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn obtain_repo() -> PathBuf {
    let repo_dir = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("darknet");
    let out_dir = Path::new(&env::var_os("OUT_DIR").unwrap()).join("darknet");
    println!(
        "cargo:rerun-if-changed={}",
        repo_dir.join("include").join("darknet.h").to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo_dir.join("Makefile").to_str().unwrap()
    );
    println!("Output dir: {}", out_dir.to_str().unwrap());
    println!("Repo dir: {}", repo_dir.to_str().unwrap());
    /*if out_dir.exists() {
        fs::remove_dir_all(&out_dir);
    }*/
    copy_dir::copy_dir(&repo_dir.to_str().unwrap(), &out_dir).unwrap();
    out_dir.to_owned()
}

fn build_with_make(path: &PathBuf) {
    let _cc = cc::Build::new().get_compiler();
    let make_status = Command::new("make")
        .env("CC", _cc.path().to_str().unwrap())
        .current_dir(&path)
        .status()
        .expect("Failed to execute make!");
    if make_status.success() {
        println!("cargo:rustc-link-search={}", path.to_str().unwrap());
        println!("cargo:rustc-link-lib=static=darknet");
    } else {
        panic!("make retruned non-zero status!");
    }
}

fn gen_bindings(path: &PathBuf) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_path = out_path.join("bindings.rs");
    println!("Writing bindings to: {}", bindings_path.to_str().unwrap());
    bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(path.join("include").join("darknet.h").to_str().unwrap())
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        //.parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings")
        //.write_to_file(out_path.join("bindings.rs"))
        .write_to_file(bindings_path)
        .expect("Couldn't write bindings!");
}

fn main() {
    let path = obtain_repo();
    build_with_make(&path);
    gen_bindings(&path);
}
