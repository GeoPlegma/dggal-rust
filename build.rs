use glob;
use std::env;
use std::fs;
use std::fs::{read_to_string, write};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::fs::symlink as symlink_file;

#[cfg(windows)]
use std::os::windows::fs::symlink_file;

/// Cross-platform symlink creator (for files)
fn create_symlink(src: &Path, dst: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        if dst.exists() {
            std::fs::remove_file(dst)?;
        }
        symlink_file(src, dst)
    }

    #[cfg(unix)]
    {
        if dst.exists() {
            std::fs::remove_file(dst)?;
        }
        symlink_file(src, dst)
    }
}

fn ensure_symlinks(from_dir: &Path, to_dir: &Path, filenames: &[&str]) {
    fs::create_dir_all(to_dir).unwrap();

    for name in filenames {
        let src = from_dir.join(name);
        let dst = to_dir.join(name);
        if !src.exists() {
            println!("cargo:warning=Missing source file for symlink: {:?}", src);
            continue;
        }
        if dst.exists() {
            fs::remove_file(&dst).unwrap();
        }
        symlink_file(&src, &dst).unwrap_or_else(|e| {
            panic!("Failed to symlink {:?} -> {:?}: {:?}", dst, src, e);
        });
    }
}

fn patch_ecrt_cffi_rs(lib_rs_path: &Path) {
    let mut content = read_to_string(lib_rs_path).expect("Failed to read lib.rs");

    // Patch extern
    content = content.replace("/*unsafe*/ extern", "unsafe extern");

    // Prepend allow directives
    let header = r#"#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
"#;

    if !content.starts_with(header) {
        content = format!("{}{}", header, content);
    }

    write(lib_rs_path, content).expect("Failed to write patched lib.rs");
}

fn patch_dggal_cffi_rs(lib_rs_path: &Path) {
    let mut content = read_to_string(lib_rs_path).expect("Failed to read lib.rs");

    // Patch extern
    content = content.replace("/*unsafe*/ extern", "unsafe extern");

    write(lib_rs_path, content).expect("Failed to write patched lib.rs");
}

fn patch_dggal_rs(lib_rs_path: &Path) {
    let content = read_to_string(lib_rs_path).expect("Failed to read dggal.rs");

    let replacement = r#"#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

//include!("ecrt_cffi.rs");
//include!("dggal_cffi.rs");
"#;

    let mut lines: Vec<&str> = content.lines().collect();

    // Replace the first 6 lines
    let rest = if lines.len() > 6 {
        lines.split_off(6)
    } else {
        vec![]
    };

    let mut new_content = replacement.to_string();
    new_content.push('\n');
    new_content.push_str(&rest.join("\n"));

    write(lib_rs_path, new_content).expect("Failed to write patched lib.rs");
}

fn copy_if_exists(src: &Path, dst: &Path) {
    if !src.exists() {
        panic!("Expected file not found: {:?}", src);
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::copy(src, dst).expect(&format!("Failed to copy from {:?} to {:?}", src, dst));
}

fn find_and_copy(dgbuild: &Path, filename: &str, dest_dir: &Path) {
    let mut found = false;

    for entry in walkdir::WalkDir::new(dgbuild)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && e.file_name() == filename)
    {
        let src = entry.path();
        println!("{:?}", src);
        let dst = dest_dir.join(filename);
        println!("cargo:warning=Copying {:?} to {:?}", src, dst);
        fs::create_dir_all(dest_dir).unwrap();
        fs::copy(src, dst).expect("Failed to copy library file");
        found = true;
        break;
    }

    if !found {
        panic!("Required file {:?} not found under {:?}", filename, dgbuild);
    }
}

fn run(cmd: &mut Command) {
    println!("cargo:warning=Running: {:?}", cmd);
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to execute command");
    if !status.success() {
        panic!("Command failed: {:?}", cmd);
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = manifest_dir.join("lib");
    let dgbuild = manifest_dir.join("dgbuild");

    // Clean and recreate dgbuild
    if lib_dir.exists() {
        println!("cargo:warning=Removing existing lib");
        fs::remove_dir_all(&lib_dir).expect("Failed to remove lib");
    }
    fs::create_dir(&lib_dir).expect("Failed to create lib");

    // Clean and recreate dgbuild
    if dgbuild.exists() {
        println!("cargo:warning=Removing existing dgbuild");
        fs::remove_dir_all(&dgbuild).expect("Failed to remove dgbuild");
    }
    fs::create_dir(&dgbuild).expect("Failed to create dgbuild");

    // Clone eC
    run(Command::new("git")
        .arg("clone")
        .arg("-b")
        .arg("main")
        .arg("--single-branch")
        .arg("https://github.com/ecere/eC.git")
        .current_dir(&dgbuild));

    // Clone dggal
    run(Command::new("git")
        .arg("clone")
        .arg("-b")
        .arg("eC-core")
        .arg("--single-branch")
        .arg("https://github.com/ecere/dggal.git")
        .current_dir(&dgbuild));

    // make -j4 in eC
    println!("make -j4 in eC");
    run(Command::new("make")
        .arg("-j4")
        //.arg("ecrt_static")
        .current_dir(&dgbuild.join("eC")));

    let ecrt_debug_lib = dgbuild.join("eC/obj/linux.debug/lib");
    let ecrt_link_lib = dgbuild.join("eC/obj/linux/lib");

    ensure_symlinks(
        &ecrt_debug_lib,
        &ecrt_link_lib,
        &[
            "libecrtStatic.a",
            "libecrt.so",
            "libecrt.so.0",
            "libecrt.so.0.0",
            "libecrt.so.0.0.1",
        ],
    );

    // make -j4 in dggal
    println!("make -j4 in dggal");
    run(Command::new("make")
        .arg("-j4")
        .arg("-f")
        .arg("Makefile.dggal.static")
        .current_dir(&dgbuild.join("dggal")));

    // 4. Symlink libdggalStatic.a after it's built
    let dggal_debug_lib = dgbuild.join("dggal/obj/static.linux.debug"); //NOTE:this is not in a lib/ folder
    let dggal_link_lib = dgbuild.join("dggal/obj/static.linux/lib");
    ensure_symlinks(&dggal_debug_lib, &dggal_link_lib, &["libdggalStatic.a"]);

    // make in dggal/bindings/rust
    run(Command::new("make").current_dir(&dgbuild.join("dggal/bindings/rust")));

    // TODO: on windows these names should be different or not?
    let filenames = [
        "libecrt_sys.rlib",
        //"libecrt_cStatic.a",
        "libecrtStatic.a",
        "libdggal_sys.rlib",
        //"libdggal_cStatic.a",
        "libdggalStatic.a",
        "libdggal.rlib",
    ];

    for fname in &filenames {
        find_and_copy(&dgbuild, fname, &lib_dir);
    }

    // ecrt_sys
    copy_if_exists(
        &dgbuild.join("dggal/bindings/rust/ecrt_cffi.rs"),
        &manifest_dir.join("src/ecrt_cffi.rs"),
    );
    let ecrt_cffi_path = manifest_dir.join("src/ecrt_cffi.rs");
    patch_ecrt_cffi_rs(&ecrt_cffi_path);

    // dggal_sys
    copy_if_exists(
        &dgbuild.join("dggal/bindings/rust/dggal_cffi.rs"),
        &manifest_dir.join("src/dggal_cffi.rs"),
    );
    let dggal_cffi_path = manifest_dir.join("src/dggal_cffi.rs");
    patch_dggal_cffi_rs(&dggal_cffi_path);

    // dggal
    copy_if_exists(
        &dgbuild.join("dggal/bindings/rust/dggal.rs"),
        &manifest_dir.join("src/lib.rs"),
    );
    let dggal_path = manifest_dir.join("src/lib.rs");
    patch_dggal_rs(&dggal_path);

    // Copy LICENSE file
    copy_if_exists(
        &dgbuild.join("dggal/LICENSE"), // adjust path if needed
        &manifest_dir.join("LICENSE"),
    );

    // Copy LICENSE file
    copy_if_exists(
        &dgbuild.join("dggal/README.md"), // adjust path if needed
        &manifest_dir.join("README.md"),
    );

    // Link all needed libraries
    println!("cargo:rustc-link-search=native=lib");

    // Link against static ecrt libraries
    println!("cargo:rustc-link-lib=static=ecrt_cStatic");
    println!("cargo:rustc-link-lib=static=ecrtStatic");
    println!("cargo:rustc-link-lib=static=dggal_cStatic");
    println!("cargo:rustc-link-lib=static=dggalStatic");

    // Rebuild if these files change
    println!("cargo:rerun-if-changed=libecrt_cStatic.a");
    println!("cargo:rerun-if-changed=libecrtStatic.a");
    println!("cargo:rerun-if-changed=libdggal_cStatic.a");
    println!("cargo:rerun-if-changed=libdggalStatic.a");

    // zlib (install via sudo apt install zlib1g-dev)
    println!("cargo:rustc-link-lib=z");
}
