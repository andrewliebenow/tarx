#![expect(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    non_camel_case_types,
    non_upper_case_globals,
    unused,
    reason = "FFI"
)]

include!(concat!(env!("OUT_DIR"), "/libforeign.rs"));
