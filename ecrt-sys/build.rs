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

fn obj_subdir(subdir: &str) -> String {
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
    format!("obj/{}{}/{}", platform, arch_suffix, subdir)
}

fn obj_lib_subdir() -> String { obj_subdir("lib") }
fn obj_bin_subdir() -> String { obj_subdir("bin") }

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
    let ec_dir = manifest_dir
        .join("..")
        .join("ecere/eC")
        .canonicalize()
        .expect("failed to canonicalize make path");
    let bindings_dir = ec_dir.join("bindings/c");
    let lib_dir = manifest_dir.join("lib");

    let bin_out_dir = ec_dir.join(obj_bin_subdir());

    // Step 1: Build eC core in ecere/eC/
    let mut ec_make = make();
    ec_make
        .arg("-B")
        .current_dir(&ec_dir)
        .env("PLATFORM", platform_name())
        .env("MSYSCON", "1")
        .env_remove("DEBUG");
    apply_shell(&mut ec_make);
    let ec_dir_output = ec_make
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

    // Post-build: ensure eC compiler tools are in obj/<platform>/bin/.
    // The 2nd-stage tools (ecp/obj/release.*) link against ecrt.dll and can
    // crash with access violations on some Windows setups.  Prefer the
    // bootstrap tools (bootstrap/obj/bin.*) which are pure-C self-contained.
    let exe = if cfg!(target_os = "windows") { ".exe" } else { "" };
    for tool in &["ecp", "ecc", "ecs"] {
        let tool_file = format!("{}{}", tool, exe);
        let dst = bin_out_dir.join(&tool_file);
        if !dst.exists() {
            let bootstrap_src = ec_dir
                .join("bootstrap")
                .join("obj")
                .join(format!("bin.{}", platform_name()))
                .join(&tool_file);
            let release_src = ec_dir
                .join(tool)
                .join("obj")
                .join(format!("release.{}", platform_name()))
                .join(&tool_file);
            let src = if bootstrap_src.exists() { bootstrap_src } else { release_src };
            if src.exists() {
                fs::copy(&src, &dst).unwrap_or_else(|e| {
                    panic!("Failed to copy {} from {:?}: {}", tool_file, src, e)
                });
            }
        }
    }

    let bindings_dir_obj = &bindings_dir.join("obj");
    if bindings_dir_obj.exists() {
        println!("cargo:warning=Cleaning {:?}", bindings_dir_obj);
        if let Err(err) = fs::remove_dir_all(&bindings_dir_obj) {
            panic!("Failed to delete {:?}: {}", bindings_dir_obj, err);
        }
    }

    // Step 2: Build Rust bindings via Makefile.ecrt-sys
    let mut bindings_make = make();
    bindings_make
        .current_dir(&bindings_dir)
        .env("PLATFORM", platform_name())
        .env("MSYSCON", "1")
        .env_remove("DEBUG");
    apply_shell(&mut bindings_make);
    let bindings_dir_output = bindings_make
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

    if lib_dir.exists() {
        fs::remove_dir_all(&lib_dir).expect(&format!("Failed to delete {:?} directory", &lib_dir));
    }
    fs::create_dir_all(&lib_dir).expect(&format!("Failed to create {:?} directory", &lib_dir));

    let files_to_copy = ["libecrt_cStatic.a", "libecrtStatic.a"];
    let obj_subdir = obj_lib_subdir();

    for file_name in &files_to_copy {
        let src = ec_dir.join(&obj_subdir).join(file_name);
        let dst = lib_dir.join(file_name);

        fs::copy(&src, &dst).unwrap_or_else(|e| {
            panic!("Failed to copy {} from {:?}: {}", file_name, src, e)
        });
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=ecrt_cStatic");
    println!("cargo:rustc-link-lib=static=ecrtStatic");

    // zlib: on Linux install zlib1g-dev; on Windows TDM-GCC ships libz.a
    println!("cargo:rustc-link-lib=z");
}

fn platform_name() -> &'static str {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "linux".to_string());
    match target_os.as_str() {
        "windows" => "win32",
        "macos" => "apple",
        _ => "linux",
    }
}
