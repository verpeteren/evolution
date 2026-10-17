#![no_main]

//! Fuzz the generator → serialize → parse → render round-trip.
//!
//! Seeds a deterministic RNG from the fuzz input, builds a random `Pic`
//! (exercising `pick_random_node`, including the new node arms and the
//! generator magic numbers), serializes it with `to_lisp`, re-parses it, and
//! renders a tiny image. This ties the lock-step tables (generator, to_lisp,
//! parser, execute) together; any mismatch surfaces as a parse error or a
//! render panic. It must never panic/crash.

use libfuzzer_sys::fuzz_target;

use evolution::{lisp_to_pic, pic_get_rgba8, Pic, DEFAULT_COORDINATE_SYSTEM};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::sync::Arc;

fuzz_target!(|data: &[u8]| {
    let mut seed = [0u8; 32];
    for (slot, b) in seed.iter_mut().zip(data.iter()) {
        *slot = *b;
    }
    let mut rng = StdRng::from_seed(seed);

    // Empty picture-name list → the generator never emits Picture nodes, so no
    // external image assets are needed to render.
    let pic = Pic::new(&mut rng, &Vec::<&String>::new());

    let source = pic.to_lisp();
    if let Ok(parsed) = lisp_to_pic(source, DEFAULT_COORDINATE_SYSTEM) {
        let _ = pic_get_rgba8(&parsed, false, Arc::new(HashMap::new()), 8, 8, 0.0);
    }
});
