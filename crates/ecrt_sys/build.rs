use std::env;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    println!("cargo:rustc-link-search=native=lib");

    // Link against static ecrt libraries
    println!("cargo:rustc-link-lib=static=ecrt_cStatic");
    println!("cargo:rustc-link-lib=static=ecrtStatic");

    // Rebuild if these files change
    println!("cargo:rerun-if-changed=libecrt_cStatic.a");
    println!("cargo:rerun-if-changed=libecrtStatic.a");

    // zlib (install via sudo apt install zlib1g-dev)
    println!("cargo:rustc-link-lib=z");
}
