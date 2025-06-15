use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let dggal_dir = manifest_dir.join("..").join("ecere/dggal");
    let bindings_dir = dggal_dir.join("bindings/rust");
    let lib_dir = manifest_dir.join("lib");

    // Step 0: clean ecere/eC/
    // let status = Command::new("make")
    //     .current_dir(&dggal_dir)
    //     .arg("distclean")
    //     .status()
    //     .expect("Failed to run `make distclean` in ecere/ec/");
    // if !status.success() {
    //     panic!("make distclean in ecere/eC/ failed");
    // }

    // Step 1: Build dggal core in ecere/dggal/
    let status = Command::new("make")
        .current_dir(&dggal_dir)
        .status()
        .expect("Failed to run `make` in ecere/dggal/");
    if !status.success() {
        panic!("make in ecere/dggal/ failed");
    }

    // Step 2: Build Rust bindings via Makefile.ecrt-sys
    let output = Command::new("make")
        .current_dir(&bindings_dir)
        .arg("-f")
        .arg("Makefile")
        .output()
        .expect("Failed to run make");

    println!(
        "cargo:warning=make stdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "cargo:warning=make stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    if !output.status.success() {
        panic!("make for dggal Rust bindings failed");
    }

    fs::create_dir_all(&lib_dir).expect("Failed to create lib/ directory");

    fs::copy(
        dggal_dir.join("obj/linux/lib/libdggal_cStatic.a"),
        lib_dir.join("libdggal_cStatic.a"),
    )
    .expect("Failed to copy libecrt_cStatic.a");

    fs::copy(
        dggal_dir.join("bindings/rust/obj/linux/libdggal_sys.rlib"),
        lib_dir.join("libdggal_sys.rlib"),
    )
    .expect("Failed to copy libdggal_sys.rlib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=dggal_cStatic");
    //println!("cargo:rustc-link-lib=static=ecrt_sys");
}
