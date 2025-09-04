# Notes

When changing to a new version, do this:

* update submodules for ecere/dggal and ecere/eC
* copy these files
```
cp ecere/eC/bindings/rust/ecrt.rs src/bindings/ecrt.rs
cp ecere/eC/bindings/rust/ecrt_cffi.rs src/ffi/ecrt_cffi.rs
cp ecere/dggal/bindings/rust/dggal.rs src/bindings/dggal.rs 
cp ecere/dggal/bindings/rust/dggal_cffi.rs src/ffi/dggal_cffi.rs 
````
* add to `ecrt.rs`
````
unsafe impl Sync for Application {}
unsafe impl Send for Application {}
````
* make sure to adjust the `use` statements
* replace the README.md
