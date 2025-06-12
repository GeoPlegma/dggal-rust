use make_cmd::make;
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

    let ec_obj = manifest_dir.join("eC/obj");
    let ec_bindings_obj = manifest_dir.join("eC/bindings/rust/obj");
    let dggal_obj = manifest_dir.join("dggal/obj");
    let dggal_bindings_obj = manifest_dir.join("dggal/bindings/rust/obj");

    if ec_obj.exists() {
        println!("cargo:warning=Cleaning eC/obj");
        fs::remove_dir_all(&ec_obj).expect("Failed to clean eC/obj");
    }

    if ec_bindings_obj.exists() {
        println!("cargo:warning=Cleaning eC/bindings/rust/obj");
        fs::remove_dir_all(&ec_bindings_obj).expect("Failed to clean eC/bindings/rust/obj");
    }

    if dggal_obj.exists() {
        println!("cargo:warning=Cleaning dggal/obj");
        fs::remove_dir_all(&dggal_obj).expect("Failed to clean dggal/obj");
    }

    if dggal_bindings_obj.exists() {
        println!("cargo:warning=Cleaning dggal/bindings/rust/obj");
        fs::remove_dir_all(&dggal_bindings_obj).expect("Failed to clean dggal/bindings/rust/obj");
    }

    // Clean and recreate dgbuild
    if lib_dir.exists() {
        println!("cargo:warning=Removing existing lib");
        fs::remove_dir_all(&lib_dir).expect("Failed to remove lib");
    }
    fs::create_dir(&lib_dir).expect("Failed to create lib");

    make()
        .current_dir(&manifest_dir.join("eC/"))
        .status()
        .expect("Failed to run `make` in eC/");

    let output = make()
        .current_dir(&manifest_dir.join("eC/bindings/rust/"))
        .output()
        .expect("Failed to run make");

    println!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        panic!("make for eC Rust bindings failed");
    }

    make()
        .current_dir(&manifest_dir.join("dggal/"))
        .status()
        .expect("Failed to run `make` in dggal/");
    make()
        .current_dir(&manifest_dir.join("dggal/bindings/rust/"))
        .status()
        .expect("Failed to run `make` in dggal/bindings/rust/");

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
