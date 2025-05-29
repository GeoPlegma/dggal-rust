#!/bin/bash
rm -rf ecere
mkdir ecere  


git clone git@github.com:ecere/dggal.git ecere/dggal #TODO: Only the fetchandbuild is needed.
cd ecere/dggal/

VERSION=$(git describe --tags --abbrev=0 | sed 's/^v//')
echo "Current version is $VERSION"


./fetchAndBuild.sh

cd ../../crates

# ECRT SYS
cd ecrt_sys
cargo clean
cp ../../ecere/dggal/dgbuild/dggal/bindings/rust/ecrt_cffi.rs ./src/lib.rs
cp ../../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libecrt_sys.rlib ./lib/.
cp ../../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libecrt_cStatic.a ./lib/.
cp ../../ecere/dggal/dgbuild/eC/obj/linux/lib/libecrtStatic.a ./lib/.
cp ../../ecere/dggal/LICENSE .
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" ./Cargo.toml

sed -i 's|/\*unsafe\*/ *extern|unsafe extern|' ./src/lib.rs
sed -i '1i\#![allow(non_upper_case_globals)]' ./src/lib.rs
sed -i '1i\#![allow(non_snake_case)]' ./src/lib.rs
sed -i '1i\#![allow(non_camel_case_types)]' ./src/lib.rs

# DGGAL SYS
cd ../dggal_sys
cargo clean
cp ../../ecere/dggal/dgbuild/dggal/bindings/rust/dggal_cffi.rs ./src/lib.rs
cp ../../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libdggal_sys.rlib ./lib/.
cp ../../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libdggal_cStatic.a ./lib/.
cp ../../ecere/dggal/dgbuild/dggal/obj/static.linux/libdggalStatic.a ./lib/.
cp ../../ecere/dggal/LICENSE .
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" ./Cargo.toml
sed -i "s/^ecrt_sys = \".*\"/ecrt_sys = \"$VERSION\"/" ./Cargo.toml

sed -i 's|/\*unsafe\*/ *extern|unsafe extern|' ./src/lib.rs

# DGGAL
cd ../dggal
cargo clean
cp ../../ecere/dggal/dgbuild/dggal/bindings/rust/dggal.rs ./src/lib.rs
cp ../../ecere/dggal/dgbuild/dggal/bindings/rust/obj/linux/libdggal.rlib ./lib/.
cp ../../ecere/dggal/LICENSE .
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" ./Cargo.toml
sed -i "s/^ecrt_sys = \".*\"/ecrt_sys = \"$VERSION\"/" ./Cargo.toml
sed -i "s/^dggal_sys = \".*\"/dggal_sys = \"$VERSION\"/" ./Cargo.toml

cd ..

