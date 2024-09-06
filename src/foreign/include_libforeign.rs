#![expect(
    clippy::unreadable_literal,
    dead_code,
    non_camel_case_types,
    non_upper_case_globals,
    reason = "FFI"
)]

include!(concat!(env!("OUT_DIR"), "/libforeign.rs"));
