use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ec_dir = manifest_dir.join("..").join("ecere/ec");
    let bindings_dir = ec_dir.join("bindings/rust");
    let lib_dir = manifest_dir.join("lib");
    // Step 1: Build eC core in eC/
    let status = Command::new("make")
        .current_dir(&ec_dir)
        .status()
        .expect("Failed to run `make` in ecere/ec/");
    if !status.success() {
        panic!("make in ec/ failed");
    }

    // Step 2: Build Rust bindings via Makefile.ecrt-sys
    let output = Command::new("make")
        .current_dir(&bindings_dir)
        .arg("-f")
        .arg("Makefile.ecrt-sys")
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
        panic!("make for eC Rust bindings failed");
    }

    fs::create_dir_all(&lib_dir).expect("Failed to create lib/ directory");

    fs::copy(
        ec_dir.join("/obj/linux/lib/libecrt_cStatic.a"),
        lib_dir.join("libecrt_cStatic.a"),
    )
    .expect("Failed to copy libecrt_cStatic.a");

    fs::copy(
        ec_dir.join("bindings/rust/obj/linux/libecrt_sys.rlib"),
        lib_dir.join("libecrt_sys.rlib"),
    )
    .expect("Failed to copy libecrt_sys.rlib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=ecrt_cStatic");
    //println!("cargo:rustc-link-lib=static=ecrt_sys");
}
