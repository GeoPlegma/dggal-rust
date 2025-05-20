#!/bin/bash
echo test


mkdir src

git clone git@github.com:ecere/dggal.git src/dggal
cd src/dggal/
./fetchAndBuild.sh

cd ../../dst

cp ../src/dggal/dgbuild/dggal/bindings/rust/ecrt_cffi.rs ./ecrt_sys/src/lib.rs
cp ../src/dggal/dgbuild/dggal/bindings/rust/obj/linux/libecrt_sys.rlib ./ecrt_sys/.
cp ../src/dggal/dgbuild/dggal/bindings/rust/obj/linux/libecrt_cStatic.a ./ecrt_sys/.
cp ../src/dggal/dgbuild/eC/obj/linux/lib/libecrtStatic.a ./ecrt_sys/.

sed -i 's|/\*unsafe\*/ *extern|unsafe extern|' ./ecrt_sys/src/lib.rs
sed -i '1i\#![allow(non_upper_case_globals)]' ./ecrt_sys/src/lib.rs
sed -i '1i\#![allow(non_snake_case)]' ./ecrt_sys/src/lib.rs
sed -i '1i\#![allow(non_camel_case_types)]' ./ecrt_sys/src/lib.rs


cp ../src/dggal/dgbuild/dggal/bindings/rust/dggal_cffi.rs ./dggal_sys/src/lib.rs
cp ../src/dggal/dgbuild/dggal/bindings/rust/obj/linux/libdggal_sys.rlib ./dggal_sys/.
cp ../src/dggal/dgbuild/dggal/bindings/rust/obj/linux/libdggal_cStatic.a ./dggal_sys/.
cp ../src/dggal/dgbuild/dggal/obj/static.linux/libdggalStatic.a ./dggal_sys/.

sed -i 's|/\*unsafe\*/ *extern|unsafe extern|' ./dggal_sys/src/lib.rs


cp ../src/dggal/dgbuild/dggal/bindings/rust/dggal.rs ./dggal/src/lib.rs
cp ../src/dggal/dgbuild/dggal/bindings/rust/obj/linux/libdggal.rlib ./dggal/.

