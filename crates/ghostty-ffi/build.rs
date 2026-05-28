//! Generate a C header skeleton for the Rust embedding bootstrap (`ghostty_rust.h`).

use std::path::PathBuf;

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rerun-if-changed=src/lib.rs");

    let bindings = cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("GHOSTTY_RUST_H")
        .with_documentation(true)
        .generate()
        .expect("cbindgen failed");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_dir.join("ghostty_rust.h"));
}
