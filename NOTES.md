# Notes

When changing to a new version, do this:

* update submodules for ecere/dggal and ecere/eC using `git submodule update --remote --merge`
* copy these files
```
cp ecere/eC/bindings/rust/ecrt.rs src/bindings/ecrt.rs
cp ecere/eC/bindings/rust/ecrt_cffi.rs src/ffi/ecrt_cffi.rs
cp ecere/dggal/bindings/rust/dggal.rs src/bindings/dggal.rs 
cp ecere/dggal/bindings/rust/dggal_cffi.rs src/ffi/dggal_cffi.rs 
````
* add `unsafe impl` Send and Sync for Application {}, DGGRS {}, and DGGAL {}
* make sure to adjust the `use` statements
* replace the README.md
* run `cargo fmt`
