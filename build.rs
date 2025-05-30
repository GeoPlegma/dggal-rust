use std::env;
use std::fs;
use std::fs::{read_to_string, write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

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

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = manifest_dir.join("lib");

    //Choose script based on platform
    let script = if cfg!(target_os = "windows") {
        println!("cargo:rerun-if-changed=fetchAndBuild.bat");
        "fetchAndBuild.bat"
    } else {
        println!("cargo:rerun-if-changed=fetchAndBuild.sh");
        "fetchAndBuild.sh"
    };

    let script_path = manifest_dir.join(script);

    if !script_path.exists() {
        panic!("Build script not found: {:?}", script_path);
    }

    let status = Command::new(&script_path)
        .current_dir(&manifest_dir)
        .status()
        .expect("Failed to run fetchAndBuild script");

    if !status.success() {
        panic!("fetchandbuild failed");
    }

    // Copy compiled libraries to ./lib
    let dgbuild = manifest_dir.join("dgbuild");

    //    let copy_targets = [
    //        "dggal/obj/linux/lib/libecrt_sys.rlib", // TODO: we need to handle windows and linux and debug versions
    //        "dggal/obj/linux/lib/libecrt_cStatic.a",
    //        "eC/obj/linux/lib/libecrtStatic.a",
    //        "dggal/obj/linux/lib/libdggal_sys.rlib",
    //        "dggal/obj/linux/lib/libdggal_cStatic.a",
    //        "dggal/obj/static.linux/libdggalStatic.a",
    //        "dggal/obj/linux/lib/libdggal.rlib",
    //    ];
    //
    //    for rel_path in &copy_targets {
    //        let src = dgbuild.join(rel_path);
    //        let dst = lib_dir.join(src.file_name().unwrap());
    //        copy_if_exists(&src, &dst);
    //    }

    // TODO: on windows these names should b different?
    let filenames = [
        "libecrt_sys.rlib",
        "libecrt_cStatic.a",
        "libecrtStatic.a",
        "libdggal_sys.rlib",
        "libdggal_cStatic.a",
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
