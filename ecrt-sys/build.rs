use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// make is an alias for gnu_make.
pub use gnu_make as make;

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub fn gnu_make() -> Command {
    Command::new("gmake")
}

#[cfg(not(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
pub fn gnu_make() -> Command {
    Command::new("make")
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub fn bsd_make() -> Command {
    Command::new("make")
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ec_dir = manifest_dir
        .join("..")
        .join("ecere/eC")
        .canonicalize()
        .expect("failed to canonicalize make path");
    let bindings_dir = ec_dir.join("bindings/c");
    let lib_dir = manifest_dir.join("lib");

    // Step 0: clean ecere/eC/
    let status = make()
        .current_dir(&ec_dir)
        .arg("distclean")
        .status()
        .expect("Failed to run `make distclean` in ecere/ec/");
    if !status.success() {
        panic!("make distclean in ecere/eC/ failed");
    }

    // let ec_dir_obj = &ec_dir.join("obj");
    // if ec_dir_obj.exists() {
    //     println!("cargo:warning=Cleaning {:?}", ec_dir_obj); // optional debug
    //     if let Err(err) = fs::remove_dir_all(&ec_dir_obj) {
    //         panic!("Failed to delete {:?}: {}", ec_dir, err);
    //     }
    // }

    // Step 1: Build eC core in ecere/eC/
    let ec_dir_output = make()
        .current_dir(&ec_dir)
        .env_remove("DEBUG")
        .output()
        .expect(&format!("Failed to run make in {:?}", ec_dir));

    println!(
        "cargo:warning=make stdout:\n{}",
        String::from_utf8_lossy(&ec_dir_output.stdout)
    );
    println!(
        "cargo:warning=make stderr:\n{}",
        String::from_utf8_lossy(&ec_dir_output.stderr)
    );

    if !ec_dir_output.status.success() {
        panic!("make in {:?} failed", ec_dir);
    }

    let bindings_dir_obj = &bindings_dir.join("obj");
    if bindings_dir_obj.exists() {
        println!("cargo:warning=Cleaning {:?}", bindings_dir_obj); // optional debug
        if let Err(err) = fs::remove_dir_all(&bindings_dir_obj) {
            panic!("Failed to delete {:?}: {}", bindings_dir_obj, err);
        }
    }

    // Step 2: Build Rust bindings via Makefile.ecrt-sys
    let bindings_dir_output = make()
        .current_dir(&bindings_dir)
        .env_remove("DEBUG")
        .output()
        .expect(&format!("Failed to run `make` in {:?}", bindings_dir));

    println!(
        "cargo:warning=make stdout:\n{}",
        String::from_utf8_lossy(&bindings_dir_output.stdout)
    );
    println!(
        "cargo:warning=make stderr:\n{}",
        String::from_utf8_lossy(&bindings_dir_output.stderr)
    );

    if !bindings_dir_output.status.success() {
        panic!("make in {:?} failed", bindings_dir);
    }

    fs::remove_dir_all(&lib_dir).expect("Failed to delete lib/ directory");
    fs::create_dir_all(&lib_dir).expect("Failed to create lib/ directory");

    fs::copy(
        ec_dir.join("obj/linux/lib/libecrt_cStatic.a"),
        lib_dir.join("libecrt_cStatic.a"),
    )
    .expect("Failed to copy libecrt_cStatic.a");

    // fs::copy(
    //     ec_dir.join("bindings/rust/obj/linux/libecrt_sys.rlib"),
    //     lib_dir.join("libecrt_sys.rlib"),
    // )
    // .expect("Failed to copy libecrt_sys.rlib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=ecrt_cStatic");
    //println!("cargo:rustc-link-lib=static=ecrt_sys");
}
