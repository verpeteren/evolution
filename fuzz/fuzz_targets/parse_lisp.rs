#![no_main]

//! Fuzz the APT "lisp" language front end.
//!
//! `lisp_to_pic` is the public entry point that lexes and parses an
//! arbitrary source string into a `Pic`. For any input it must return
//! `Ok`/`Err` rather than panic, overflow, or crash. The fuzzer feeds it
//! arbitrary (lossily-decoded) UTF-8 and we discard the result.

use libfuzzer_sys::fuzz_target;

use evolution::constants::DEFAULT_COORDINATE_SYSTEM;
use evolution::lisp_to_pic;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data).into_owned();
    let _ = lisp_to_pic(source, DEFAULT_COORDINATE_SYSTEM);
});
