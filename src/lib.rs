//pub use ecrt::*;
pub mod ffi {
    pub mod dggal_cffi;
    pub mod ecrt_cffi;
}

pub mod bindings {
    pub mod dggal;
    pub mod ecrt;
}

// Optional, if you want public entry points
pub use bindings::*;
