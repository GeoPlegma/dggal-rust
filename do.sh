#!/bin/bash
echo test

rm -rf ecrt_sys

cargo new --lib ecrt_sys

cp ../ecere/dggal/dgbuild/dggal/bindings/rust/ecrt_cffi.rs ./ecrt_sys/src/lib.rs
cp ../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libecrt_sys.rlib ./ecrt_sys/.
cp ../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libecrt_cStatic.a ./ecrt_sys/.
cp ../ecere/dggal/dgbuild/eC/obj/linux/lib/libecrtStatic.a ./ecrt_sys/.

sed -i 's|/\*unsafe\*/ *extern|unsafe extern|' ./ecrt_sys/src/lib.rs
sed -i '1i\#![allow(non_upper_case_globals)]' ./ecrt_sys/src/lib.rs
sed -i '1i\#![allow(non_snake_case)]' ./ecrt_sys/src/lib.rs
sed -i '1i\#![allow(non_camel_case_types)]' ./ecrt_sys/src/lib.rs

cat <<EOF > ./ecrt_sys/build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    println!("cargo:rustc-link-search=native={}", dir.display());

    // Link against static ecrt libraries
    println!("cargo:rustc-link-lib=static=ecrt_cStatic");
    println!("cargo:rustc-link-lib=static=ecrtStatic");

    // Rebuild if these files change
    println!("cargo:rerun-if-changed=libecrt_cStatic.a");
    println!("cargo:rerun-if-changed=libecrtStatic.a");

    // zlib (install via sudo apt install zlib1g-dev)
    println!("cargo:rustc-link-lib=z");
}
EOF


rm -rf dggal_sys

cargo new --lib dggal_sys

cp ../ecere/dggal/dgbuild/dggal/bindings/rust/dggal_cffi.rs ./dggal_sys/src/lib.rs
cp ../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libdggal_sys.rlib ./dggal_sys/.
cp ../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libdggal_cStatic.a ./dggal_sys/.
cp ../ecere/dggal/dgbuild/dggal/obj/static.linux/libdggalStatic.a ./dggal_sys/.

sed -i 's|/\*unsafe\*/ *extern|unsafe extern|' ./dggal_sys/src/lib.rs
echo 'ecrt_sys = { path = "/home/mj/CODE/gh/dggal-rust/ecrt_sys" }' >> ./dggal_sys/Cargo.toml

cat <<EOF > ./dggal_sys/build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Link against static dggal libraries
    println!("cargo:rustc-link-search=native={}", dir.display());
    println!("cargo:rustc-link-lib=static=dggal_cStatic");
    println!("cargo:rustc-link-lib=static=dggalStatic");

    // Rebuild if these files change
    println!("cargo:rerun-if-changed=libdggal_cStatic.a");
    println!("cargo:rerun-if-changed=libdggalStatic.a");
}
EOF

rm -rf dggal

cargo new --lib dggal

cp ../ecere/dggal/dgbuild/dggal/bindings/rust/dggal.rs ./dggal/src/lib.rs
cp ../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libdggal.rlib ./dggal/.
echo 'ecrt_sys = { path = "/home/mj/CODE/gh/dggal-rust/ecrt_sys" }' >> ./dggal/Cargo.toml
echo 'dggal_sys = { path = "/home/mj/CODE/gh/dggal-rust/dggal_sys" }' >> ./dggal/Cargo.toml

