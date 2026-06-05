use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
fn make() -> Command {
    let mut cmd = Command::new("mingw32-make");
    cmd.env_remove("SHELL");
    cmd.arg("SHELL=C:\\Windows\\System32\\cmd.exe");
    cmd
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
))]
fn make() -> Command {
    Command::new("gmake")
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
fn make() -> Command {
    Command::new("make")
}

fn platform_lib_dir() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "obj/win32/lib"
    }
    #[cfg(target_os = "macos")]
    {
        "obj/apple/lib"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        "obj/linux/lib"
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = manifest_dir.join("lib");
    if lib_dir.exists() {
        fs::remove_dir_all(&lib_dir).expect(&format!("Failed to delete {:?} directory", &lib_dir));
    }
    fs::create_dir_all(&lib_dir).expect(&format!("Failed to create {:?} directory", &lib_dir));

    let ec_dir = manifest_dir.join("ecere/eC");
    let ec_bindings_dir = ec_dir.join("bindings/c");

    // Step 0: clean ecere/eC/
    let status = make()
        .current_dir(&ec_dir)
        .arg("distclean")
        .status()
        .expect(&format!("Failed to run `make distclean` in {:?}", &ec_dir));
    if !status.success() {
        panic!("{}", &format!("make distclean in {:?} failed", &ec_dir));
    }

    // Step 1: Build eC core
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

    let ec_bindings_dir_obj = &ec_bindings_dir.join("obj");
    if ec_bindings_dir_obj.exists() {
        println!("cargo:warning=Cleaning {:?}", ec_bindings_dir_obj); // optional debug
        if let Err(err) = fs::remove_dir_all(&ec_bindings_dir_obj) {
            panic!("Failed to delete {:?}: {}", ec_bindings_dir_obj, err);
        }
    }

    // Step 2: Build Rust bindings via Makefile.ecrt-sys
    let ec_bindings_dir_output = make()
        .current_dir(&ec_bindings_dir)
        .env_remove("DEBUG")
        .output()
        .expect(&format!("Failed to run `make` in {:?}", ec_bindings_dir));

    println!(
        "cargo:warning=make stdout:\n{}",
        String::from_utf8_lossy(&ec_bindings_dir_output.stdout)
    );
    println!(
        "cargo:warning=make stderr:\n{}",
        String::from_utf8_lossy(&ec_bindings_dir_output.stderr)
    );

    if !ec_bindings_dir_output.status.success() {
        panic!("make in {:?} failed", ec_bindings_dir);
    }

    let files_to_copy = ["libecrt_cStatic.a", "libecrtStatic.a"];

    for file_name in &files_to_copy {
        let src = ec_dir.join(platform_lib_dir()).join(file_name);
        let dst = lib_dir.join(file_name);

        fs::copy(&src, &dst).expect(&format!("Failed to copy {}", file_name));
    }

    let dggal_dir = manifest_dir.join("ecere/dggal");
    let dggal_bindings_dir = dggal_dir.join("bindings/c");

    // Step 0: clean ecere/dggal/
    let status = make()
        .current_dir(&dggal_dir)
        .arg("distclean")
        .status()
        .expect(&format!(
            "Failed to run `make distclean` in {:?}",
            &dggal_dir
        ));
    if !status.success() {
        panic!("{}", &format!("make distclean in {:?} failed", &dggal_dir));
    }

    // Step 1: Build DGGAL core
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

    let dggal_bindings_dir_obj = &dggal_bindings_dir.join("obj");
    if dggal_bindings_dir_obj.exists() {
        println!("cargo:warning=Cleaning {:?}", dggal_bindings_dir_obj); // optional debug
        if let Err(err) = fs::remove_dir_all(&dggal_bindings_dir_obj) {
            panic!("Failed to delete {:?}: {}", dggal_bindings_dir_obj, err);
        }
    }

    // Step 2: Build Rust bindings via Makefile
    let dggal_bindings_dir_output = make()
        .current_dir(&dggal_bindings_dir)
        .env_remove("DEBUG")
        .output()
        .expect("Failed to run make");

    println!(
        "cargo:warning=make stdout:\n{}",
        String::from_utf8_lossy(&dggal_bindings_dir_output.stdout)
    );
    println!(
        "cargo:warning=make stderr:\n{}",
        String::from_utf8_lossy(&dggal_bindings_dir_output.stderr)
    );

    if !dggal_bindings_dir_output.status.success() {
        panic!("make for dggal Rust bindings failed");
    }

    let files_to_copy = ["libdggal_cStatic.a", "libdggalStatic.a"];

    for file_name in &files_to_copy {
        let src = dggal_dir.join(platform_lib_dir()).join(file_name);
        let dst = lib_dir.join(file_name);

        fs::copy(&src, &dst).expect(&format!("Failed to copy {}", file_name));
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=ecrt_cStatic");
    println!("cargo:rustc-link-lib=static=ecrtStatic");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=static=dggal_cStatic");
    println!("cargo:rustc-link-lib=static=dggalStatic");
    println!("cargo:rerun-if-changed=src/ffi/dggal_cffi.rs");
    println!("cargo:rerun-if-changed=src/ffi/ecrt_cffi.rs");
}
