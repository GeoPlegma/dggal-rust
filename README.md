DGGAL-RUST
===========

Copyright
---------
Copyright (c) 2025 contributors to the GeoPlegmata project. All rights reserved. Any use of this software constitutes full acceptance of all terms of its licence.

Overview
--------
Just the facility to publish [DGGAL](https://dggal.org/) and its dependencies to crates.io.

Licence
-------

This project clones, compiles, and distributes code from [https://github.com/ecere/dggal](https://github.com/ecere/dggal) licensed under [BSD 3-Clause License](https://github.com/ecere/dggal/blob/eC-core/LICENSE), everythinh else in licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0) LICENSE-APACHE or the [MIT license](http://opensource.org/licenses/MIT), at your discretion. All contents herewith  may not be copied, modified, or distributed except according to the terms defined in the licence chosen. Refer to the files  [LICENCE](LICENCE), [LICENCE-APACHE.txt](LICENCE-APACHE.txt) and [LICENCE-MIT.txt](LICENCE-MIT.txt) for details.


Publishing
---------

To publish to crates.io set the `CARGO_REGISTRY_TOKEN` and then run `cargo publish`

Folder structure
----------------
```
dst/
├── dggal
│   ├── Cargo.toml
│   ├── libdggal.rlib
│   └── src
│       └── lib.rs
├── dggal_sys
│   ├── build.rs
│   ├── Cargo.toml
│   ├── libdggal_cStatic.a
│   ├── libdggalStatic.a
│   ├── libdggal_sys.rlib
│   └── src
│       └── lib.rs
└── ecrt_sys
    ├── build.rs
    ├── Cargo.toml
    ├── libecrt_cStatic.a
    ├── libecrtStatic.a
    ├── libecrt_sys.rlib
    └── src
        └── lib.rs
```
