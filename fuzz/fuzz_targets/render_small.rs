#![no_main]

//! Fuzz the full parse → render pipeline.
//!
//! `parse_lisp` only exercises the lexer/parser. This target additionally
//! renders any successfully-parsed `Pic` into a tiny image, driving the VM's
//! `execute` arms — including the new 3D/4D noise nodes and their stack-index
//! arithmetic, which is where an arity bug would surface as an out-of-bounds
//! panic. It must never panic/crash for any input.

use libfuzzer_sys::fuzz_target;

use evolution::{lisp_to_pic, pic_get_rgba8, DEFAULT_COORDINATE_SYSTEM};
use std::collections::HashMap;
use std::sync::Arc;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data).into_owned();
    if let Ok(pic) = lisp_to_pic(source, DEFAULT_COORDINATE_SYSTEM) {
        // Small, single-threaded render keeps each case fast and deterministic.
        let _ = pic_get_rgba8(&pic, false, Arc::new(HashMap::new()), 8, 8, 0.0);
    }
});
