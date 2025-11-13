This is a supporting repository to build [DGGAL](https://github.com/ecere/dggal) for [GeoPlegma](https://github.com/GeoPlegma/GeoPlegma).

# Notes

When changing to a new version, do this:

* update submodules for ecere/dggal and ecere/eC using 
````
git submodule update --remote --merge
````

* `cd` into the submodule and checkout the commit or tag you want.

* copy these files
```
cp ecere/eC/bindings/rust/ecrt.rs src/bindings/ecrt.rs & \
cp ecere/eC/bindings/rust/ecrt_cffi.rs src/ffi/ecrt_cffi.rs &\
cp ecere/dggal/bindings/rust/dggal.rs src/bindings/dggal.rs & \
cp ecere/dggal/bindings/rust/dggal_cffi.rs src/ffi/dggal_cffi.rs 
````
* run `cargo fmt`

* Add `unsafe impl`, `Send` and `Sync` for `Application {}`, `DGGRS {}`, and `DGGAL {}`.
* make sure to adjust the `use` statements
in `src/bindings/ecrt.rs`
````
use crate::ffi::ecrt_cffi as ecrt_sys;
````
and 
````
unsafe impl Sync for Application {}
unsafe impl Send for Application {}
````

in `src/ffi/ecrt_cffi.rs`
````
unsafe impl Send for class_members_Instance {}
unsafe impl Sync for class_members_Instance {}
````

in `src/bindings/dggal.rs`
````
use crate::ffi::ecrt_cffi as ecrt_sys;

use crate::bindings::ecrt;

use crate::define_bitclass;
use crate::delegate_ttau64_and_default;
.
.
.
use crate::ffi::dggal_cffi as dggal_sys;

````
and
````
unsafe impl Send for DGGAL{};
unsafe impl Sync for DGGAL{};
````


in `src/ffi/dggal_cffi.rs`
````
use crate::ffi::ecrt_cffi as ecrt_sys;
````

* run `cargo fmt`

