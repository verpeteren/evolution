pub mod gradient;
pub mod grayscale;
pub mod hsv;
pub mod mono;
pub mod rgb;

use rand::rngs::StdRng;
use std::collections::HashMap;
use std::sync::Arc;

use crate::pic::actual_picture::ActualPicture;
pub use crate::pic::pic::Pic;
pub use gradient::GradientData;
pub use grayscale::GrayscaleData;
pub use hsv::HSVData;
pub use mono::MonoData;
pub use rgb::RGBData;

use simdeez::prelude::*;

pub trait PicData {
    // `new` returns the `Pic` enum rather than `Self`, and `&Vec` keeps the
    // signature identical across all implementors and their caller (Pic::new).
    #[allow(clippy::new_ret_no_self, clippy::ptr_arg)]
    fn new(min: usize, max: usize, video: bool, rng: &mut StdRng, pic_names: &Vec<&String>) -> Pic;
    /// Render the picture into a caller-provided RGBA8 buffer, which must be
    /// exactly `w * h * 4` bytes. Lets callers (e.g. video frames, UI
    /// thumbnails) reuse one allocation across many renders.
    fn render_into<S: Simd>(
        &self,
        buf: &mut [u8],
        threaded: bool,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    );
    /// Convenience wrapper that allocates a fresh buffer and renders into it.
    fn get_rgba8<S: Simd>(
        &self,
        threaded: bool,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    ) -> Vec<u8> {
        let mut buf = vec![0u8; (w * h * 4) as usize];
        self.render_into::<S>(&mut buf, threaded, pics, w, h, t);
        buf
    }
    fn simplify<S: Simd>(
        &mut self,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    );
}
