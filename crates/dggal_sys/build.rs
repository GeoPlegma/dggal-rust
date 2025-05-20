use std::env;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Link against static dggal libraries
    println!("cargo:rustc-link-search=native={}", dir.display());
    println!("cargo:rustc-link-lib=static=dggal_cStatic");
    println!("cargo:rustc-link-lib=static=dggalStatic");

    // Rebuild if these files change
    println!("cargo:rerun-if-changed=libdggal_cStatic.a");
    println!("cargo:rerun-if-changed=libdggalStatic.a");
}
