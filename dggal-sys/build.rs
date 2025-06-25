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
    target_os = "openbsd",
))]
pub fn gnu_make() -> Command {
    Command::new("gmake")
}

#[cfg(not(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub fn gnu_make() -> Command {
    Command::new("make")
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
))]
pub fn bsd_make() -> Command {
    Command::new("make")
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let dggal_dir = manifest_dir
        .join("..")
        .join("ecere/dggal")
        .canonicalize()
        .expect("failed to canonicalize make path");
    let bindings_dir = dggal_dir.join("bindings/c");
    let lib_dir = manifest_dir.join("lib");

    // Step 0: clean ecere/eC/
    let status = make()
        .current_dir(&dggal_dir)
        .arg("distclean")
        .status()
        .expect("Failed to run `make distclean` in ecere/dggal/");
    if !status.success() {
        panic!("make distclean in ecere/dggal/ failed");
    }

    // let dggal_dir_obj = &dggal_dir.join("obj");
    // if dggal_dir_obj.exists() {
    //     println!("cargo:warning=Cleaning {:?}", dggal_dir_obj); // optional debug
    //     if let Err(err) = fs::remove_dir_all(&dggal_dir_obj) {
    //         panic!("Failed to delete {:?}: {}", dggal_dir_obj, err);
    //     }
    // }

    // Step 1: Build dggal core in ecere/dggal/
    let dggal_dir_output = make()
        .current_dir(&dggal_dir)
        .env_remove("DEBUG")
        .output()
        .expect(&format!("Failed to run make in {:?}", dggal_dir));

    println!(
        "cargo:warning=make stdout:\n{}",
        String::from_utf8_lossy(&dggal_dir_output.stdout)
    );
    println!(
        "cargo:warning=make stderr:\n{}",
        String::from_utf8_lossy(&dggal_dir_output.stderr)
    );

    if !dggal_dir_output.status.success() {
        panic!("make in {:?} failed", dggal_dir);
    }

    let bindings_dir_obj = &bindings_dir.join("obj");
    if bindings_dir_obj.exists() {
        println!("cargo:warning=Cleaning {:?}", bindings_dir_obj); // optional debug
        if let Err(err) = fs::remove_dir_all(&bindings_dir_obj) {
            panic!("Failed to delete {:?}: {}", bindings_dir_obj, err);
        }
    }

    // Step 2: Build Rust bindings via Makefile
    let bindings_dir_output = make()
        .current_dir(&bindings_dir)
        .env_remove("DEBUG")
        .output()
        .expect("Failed to run make");

    println!(
        "cargo:warning=make stdout:\n{}",
        String::from_utf8_lossy(&bindings_dir_output.stdout)
    );
    println!(
        "cargo:warning=make stderr:\n{}",
        String::from_utf8_lossy(&bindings_dir_output.stderr)
    );

    if !bindings_dir_output.status.success() {
        panic!("make for dggal Rust bindings failed");
    }

    fs::remove_dir_all(&lib_dir).expect("Failed to delete lib/ directory");
    fs::create_dir_all(&lib_dir).expect("Failed to create lib/ directory");

    fs::copy(
        dggal_dir.join("obj/linux/lib/libdggal_cStatic.a"),
        lib_dir.join("libdggal_cStatic.a"),
    )
    .expect("Failed to copy libdggal_cStatic.a");

    // fs::copy(
    //     dggal_dir.join("bindings/rust/obj/linux/libdggal_sys.rlib"),
    //     lib_dir.join("libdggal_sys.rlib"),
    // )
    // .expect("Failed to copy libdggal_sys.rlib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=dggal_cStatic");
    //println!("cargo:rustc-link-lib=static=ecrt_sys");
}
