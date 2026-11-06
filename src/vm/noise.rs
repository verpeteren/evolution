//! Generic per-sample noise kernels, bridging simdnoise 4.0's engine-specific
//! `intrinsics` API back to the generic-over-`S: Simd` signatures the VM relies
//! on.
//!
//! simdnoise 4.0 ([`MIGRATION.md`]) made the old generic `noise::*_32` kernels
//! (`fbm_2d`, `cellular_2d`, …) crate-internal. The supported low-level entry
//! point is now `simdnoise::intrinsics::{engine}`, but those functions take the
//! engine's concrete vector type (`__m128`/`__m256`/…) rather than `S::Vf32`,
//! and `get_*` was renamed to `generate_*`.
//!
//! The [`StackMachine`](super::stackmachine::StackMachine) is generic over the
//! SIMD engine `S` (it is monomorphized per engine by simdeez's
//! `simd_runtime_generate!`). To call the concrete intrinsics from generic code
//! we dispatch on the runtime engine via `TypeId` — the comparisons fold away
//! after monomorphization, so each engine variant keeps a single direct call —
//! and transmute `S::Vf32` to/from the matching concrete type (a zero-cost
//! reinterpret, since the engine always matches `S`).
//!
//! These per-sample intrinsics are x86/x86_64 only: on aarch64 the NEON engine
//! type is not nameable outside simdeez, so simdnoise exposes only its
//! buffer-level `generate_*` helpers there. That matches this crate's existing
//! `target-cpu=native` x86 build.

use std::any::TypeId;

use simdeez::prelude::*;
use simdeez::SimdTransmuteF32;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use simdeez::engines::{avx2::Avx2, avx512::Avx512, sse2::Sse2, sse41::Sse41};

use simdnoise::{CellDistanceFunction, CellReturnType};

/// Run `$kernel` against the concrete engine backing `$s`, transmuting lanes in
/// and out. The `TypeId` ladder is resolved to a single arm per monomorphization.
macro_rules! dispatch {
    ($s:ty, $kernel:ident ! ( $($args:tt)* )) => {{
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if TypeId::of::<$s>() == TypeId::of::<Avx512>() {
                return unsafe {
                    $kernel!(avx512, try_transmute_avx512, try_transmute_from_avx512, $s, $($args)*)
                };
            }
            if TypeId::of::<$s>() == TypeId::of::<Avx2>() {
                return unsafe {
                    $kernel!(avx2, try_transmute_avx2, try_transmute_from_avx2, $s, $($args)*)
                };
            }
            if TypeId::of::<$s>() == TypeId::of::<Sse41>() {
                return unsafe {
                    $kernel!(sse41, try_transmute_sse41, try_transmute_from_sse41, $s, $($args)*)
                };
            }
            if TypeId::of::<$s>() == TypeId::of::<Sse2>() {
                return unsafe {
                    $kernel!(sse2, try_transmute_sse2, try_transmute_from_sse2, $s, $($args)*)
                };
            }
        }
        unsafe {
            $kernel!(scalar, try_transmute_scalar, try_transmute_from_scalar, $s, $($args)*)
        }
    }};
}

/// Fractal (fbm/ridge/turbulence) kernels: `N` coordinate lanes, then
/// `lacunarity`, `gain`, `octaves: f32` and `seed: i32`.
macro_rules! fractal_2d {
    ($eng:ident, $to:ident, $from:ident, $s:ty, $func:ident, $x:expr, $y:expr, $lac:expr, $gain:expr, $oct:expr, $seed:expr) => {
        <<$s as Simd>::Vf32 as SimdTransmuteF32>::$from(simdnoise::intrinsics::$eng::$func::<$s>(
            SimdTransmuteF32::$to(&$x),
            SimdTransmuteF32::$to(&$y),
            SimdTransmuteF32::$to(&$lac),
            SimdTransmuteF32::$to(&$gain),
            $oct,
            $seed,
        ))
    };
}

macro_rules! fractal_3d {
    ($eng:ident, $to:ident, $from:ident, $s:ty, $func:ident, $x:expr, $y:expr, $z:expr, $lac:expr, $gain:expr, $oct:expr, $seed:expr) => {
        <<$s as Simd>::Vf32 as SimdTransmuteF32>::$from(simdnoise::intrinsics::$eng::$func::<$s>(
            SimdTransmuteF32::$to(&$x),
            SimdTransmuteF32::$to(&$y),
            SimdTransmuteF32::$to(&$z),
            SimdTransmuteF32::$to(&$lac),
            SimdTransmuteF32::$to(&$gain),
            $oct,
            $seed,
        ))
    };
}

macro_rules! fractal_4d {
    ($eng:ident, $to:ident, $from:ident, $s:ty, $func:ident, $x:expr, $y:expr, $z:expr, $w:expr, $lac:expr, $gain:expr, $oct:expr, $seed:expr) => {
        <<$s as Simd>::Vf32 as SimdTransmuteF32>::$from(simdnoise::intrinsics::$eng::$func::<$s>(
            SimdTransmuteF32::$to(&$x),
            SimdTransmuteF32::$to(&$y),
            SimdTransmuteF32::$to(&$z),
            SimdTransmuteF32::$to(&$w),
            SimdTransmuteF32::$to(&$lac),
            SimdTransmuteF32::$to(&$gain),
            $oct,
            $seed,
        ))
    };
}

/// Cellular/voronoi kernels: `N` coordinate lanes, a jitter lane, a seed, and a
/// [`CellReturnType`]. Always uses Euclidean distance, matching the VM's DSL.
macro_rules! cellular_2d {
    ($eng:ident, $to:ident, $from:ident, $s:ty, $x:expr, $y:expr, $jitter:expr, $seed:expr, $ret:expr) => {
        <<$s as Simd>::Vf32 as SimdTransmuteF32>::$from(simdnoise::intrinsics::$eng::cellular_2d::<
            $s,
        >(
            SimdTransmuteF32::$to(&$x),
            SimdTransmuteF32::$to(&$y),
            CellDistanceFunction::Euclidean,
            $ret,
            SimdTransmuteF32::$to(&$jitter),
            $seed,
        ))
    };
}

macro_rules! cellular_3d {
    ($eng:ident, $to:ident, $from:ident, $s:ty, $x:expr, $y:expr, $z:expr, $jitter:expr, $seed:expr, $ret:expr) => {
        <<$s as Simd>::Vf32 as SimdTransmuteF32>::$from(simdnoise::intrinsics::$eng::cellular_3d::<
            $s,
        >(
            SimdTransmuteF32::$to(&$x),
            SimdTransmuteF32::$to(&$y),
            SimdTransmuteF32::$to(&$z),
            CellDistanceFunction::Euclidean,
            $ret,
            SimdTransmuteF32::$to(&$jitter),
            $seed,
        ))
    };
}

macro_rules! cellular_4d {
    ($eng:ident, $to:ident, $from:ident, $s:ty, $x:expr, $y:expr, $z:expr, $w:expr, $jitter:expr, $seed:expr, $ret:expr) => {
        <<$s as Simd>::Vf32 as SimdTransmuteF32>::$from(simdnoise::intrinsics::$eng::cellular_4d::<
            $s,
        >(
            SimdTransmuteF32::$to(&$x),
            SimdTransmuteF32::$to(&$y),
            SimdTransmuteF32::$to(&$z),
            SimdTransmuteF32::$to(&$w),
            CellDistanceFunction::Euclidean,
            $ret,
            SimdTransmuteF32::$to(&$jitter),
            $seed,
        ))
    };
}

#[inline(always)]
pub fn fbm_2d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    lacunarity: S::Vf32,
    gain: S::Vf32,
    octaves: f32,
    seed: i32,
) -> S::Vf32 {
    dispatch!(
        S,
        fractal_2d!(fbm_2d, x, y, lacunarity, gain, octaves, seed)
    )
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn fbm_3d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    z: S::Vf32,
    lacunarity: S::Vf32,
    gain: S::Vf32,
    octaves: f32,
    seed: i32,
) -> S::Vf32 {
    dispatch!(
        S,
        fractal_3d!(fbm_3d, x, y, z, lacunarity, gain, octaves, seed)
    )
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn fbm_4d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    z: S::Vf32,
    w: S::Vf32,
    lacunarity: S::Vf32,
    gain: S::Vf32,
    octaves: f32,
    seed: i32,
) -> S::Vf32 {
    dispatch!(
        S,
        fractal_4d!(fbm_4d, x, y, z, w, lacunarity, gain, octaves, seed)
    )
}

#[inline(always)]
pub fn ridge_2d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    lacunarity: S::Vf32,
    gain: S::Vf32,
    octaves: f32,
    seed: i32,
) -> S::Vf32 {
    dispatch!(
        S,
        fractal_2d!(ridge_2d, x, y, lacunarity, gain, octaves, seed)
    )
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn ridge_3d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    z: S::Vf32,
    lacunarity: S::Vf32,
    gain: S::Vf32,
    octaves: f32,
    seed: i32,
) -> S::Vf32 {
    dispatch!(
        S,
        fractal_3d!(ridge_3d, x, y, z, lacunarity, gain, octaves, seed)
    )
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn ridge_4d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    z: S::Vf32,
    w: S::Vf32,
    lacunarity: S::Vf32,
    gain: S::Vf32,
    octaves: f32,
    seed: i32,
) -> S::Vf32 {
    dispatch!(
        S,
        fractal_4d!(ridge_4d, x, y, z, w, lacunarity, gain, octaves, seed)
    )
}

#[inline(always)]
pub fn turbulence_2d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    lacunarity: S::Vf32,
    gain: S::Vf32,
    octaves: f32,
    seed: i32,
) -> S::Vf32 {
    dispatch!(
        S,
        fractal_2d!(turbulence_2d, x, y, lacunarity, gain, octaves, seed)
    )
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn turbulence_3d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    z: S::Vf32,
    lacunarity: S::Vf32,
    gain: S::Vf32,
    octaves: f32,
    seed: i32,
) -> S::Vf32 {
    dispatch!(
        S,
        fractal_3d!(turbulence_3d, x, y, z, lacunarity, gain, octaves, seed)
    )
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn turbulence_4d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    z: S::Vf32,
    w: S::Vf32,
    lacunarity: S::Vf32,
    gain: S::Vf32,
    octaves: f32,
    seed: i32,
) -> S::Vf32 {
    dispatch!(
        S,
        fractal_4d!(turbulence_4d, x, y, z, w, lacunarity, gain, octaves, seed)
    )
}

#[inline(always)]
pub fn cellular_2d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    jitter: S::Vf32,
    seed: i32,
    return_type: CellReturnType,
) -> S::Vf32 {
    dispatch!(S, cellular_2d!(x, y, jitter, seed, return_type))
}

#[inline(always)]
pub fn cellular_3d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    z: S::Vf32,
    jitter: S::Vf32,
    seed: i32,
    return_type: CellReturnType,
) -> S::Vf32 {
    dispatch!(S, cellular_3d!(x, y, z, jitter, seed, return_type))
}

#[inline(always)]
pub fn cellular_4d<S: Simd>(
    x: S::Vf32,
    y: S::Vf32,
    z: S::Vf32,
    w: S::Vf32,
    jitter: S::Vf32,
    seed: i32,
    return_type: CellReturnType,
) -> S::Vf32 {
    dispatch!(S, cellular_4d!(x, y, z, w, jitter, seed, return_type))
}
