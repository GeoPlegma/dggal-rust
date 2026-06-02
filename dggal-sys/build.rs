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

fn obj_lib_subdir() -> String {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "linux".to_string());
    let platform = match target_os.as_str() {
        "windows" => "win32",
        "macos" => "apple",
        _ => "linux",
    };
    // On 32-bit Windows, crossplatform.mk sets ARCH=x32 → COMPILER_SUFFIX=.x32
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let arch_suffix = if target_os == "windows" && target_arch == "x86" {
        ".x32"
    } else {
        ""
    };
    format!("obj/{}{}/lib", platform, arch_suffix)
}

fn platform_name() -> &'static str {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "linux".to_string());
    match target_os.as_str() {
        "windows" => "win32",
        "macos" => "apple",
        _ => "linux",
    }
}

fn find_sh_dir() -> Option<PathBuf> {
    if let Ok(out) = Command::new("where.exe").arg("sh.exe").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = s.lines().next() {
                return PathBuf::from(line.trim()).parent().map(|p| p.to_path_buf());
            }
        }
    }
    let out = Command::new("where.exe").arg("git.exe").output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    let git = PathBuf::from(s.lines().next()?.trim());
    let root = git.parent()?.parent()?;
    let usr_bin = root.join("usr").join("bin");
    if usr_bin.join("sh.exe").exists() { return Some(usr_bin); }
    let bin = root.join("bin");
    if bin.join("sh.exe").exists() { return Some(bin); }
    None
}

fn apply_shell(cmd: &mut Command) {
    if let Some(sh_dir) = find_sh_dir() {
        let existing = env::var("PATH").unwrap_or_default();
        let new_path = format!("{};{}", sh_dir.display(), existing);
        cmd.env("PATH", new_path);
        cmd.env("SHELL", "sh");
    }
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

    // Step 1: Build dggal core in ecere/dggal/
    let mut dggal_make = make();
    dggal_make
        .arg("-B")
        .current_dir(&dggal_dir)
        .env("PLATFORM", platform_name())
        .env("MSYSCON", "1")
        .env_remove("DEBUG");
    apply_shell(&mut dggal_make);
    let dggal_dir_output = dggal_make
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
        println!("cargo:warning=Cleaning {:?}", bindings_dir_obj);
        if let Err(err) = fs::remove_dir_all(&bindings_dir_obj) {
            panic!("Failed to delete {:?}: {}", bindings_dir_obj, err);
        }
    }

    // Step 2: Build Rust bindings via Makefile
    let mut bindings_make = make();
    bindings_make
        .current_dir(&bindings_dir)
        .env("PLATFORM", platform_name())
        .env("MSYSCON", "1")
        .env_remove("DEBUG");
    apply_shell(&mut bindings_make);
    let bindings_dir_output = bindings_make
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

    if lib_dir.exists() {
        fs::remove_dir_all(&lib_dir).expect(&format!("Failed to delete {:?} directory", &lib_dir));
    }
    fs::create_dir_all(&lib_dir).expect(&format!("Failed to create {:?} directory", &lib_dir));

    let files_to_copy = ["libdggal_cStatic.a", "libdggalStatic.a"];
    let obj_subdir = obj_lib_subdir();

    for file_name in &files_to_copy {
        let src = dggal_dir.join(&obj_subdir).join(file_name);
        let dst = lib_dir.join(file_name);

        fs::copy(&src, &dst).unwrap_or_else(|e| {
            panic!("Failed to copy {} from {:?}: {}", file_name, src, e)
        });
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=dggal_cStatic");
    println!("cargo:rustc-link-lib=static=dggalStatic");
}
