use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::sync::Arc;

use crate::parser::aptnode::APTNode;
use crate::pic::actual_picture::ActualPicture;
use crate::pic::coordinatesystem::{cartesian_to_polar, CoordinateSystem};
use crate::pic::data::PicData;
use crate::pic::pic::Pic;
use crate::vm::stackmachine::StackMachine;

use rayon::prelude::*;
use simdeez::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RGBData {
    pub r: APTNode,
    pub g: APTNode,
    pub b: APTNode,
    pub coord: CoordinateSystem,
}

impl PicData for RGBData {
    fn new(min: usize, max: usize, video: bool, rng: &mut StdRng, pic_names: &Vec<&String>) -> Pic {
        let (r, coord) =
            APTNode::create_random_tree(rng.random_range(min..max), video, rng, pic_names);
        let (g, _coord) =
            APTNode::create_random_tree(rng.random_range(min..max), video, rng, pic_names);
        let (b, _coord) =
            APTNode::create_random_tree(rng.random_range(min..max), video, rng, pic_names);
        Pic::RGB(RGBData { r, g, b, coord })
    }
    fn get_rgba8<S: Simd>(
        &self,
        threaded: bool,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    ) -> Vec<u8> {
        {
            let ts = S::Vf32::set1(t);
            let wf = S::Vf32::set1(w as f32);
            let hf = S::Vf32::set1(h as f32);

            let vec_len = (w * h * 4) as usize;
            let mut result = vec![0u8; vec_len];

            let r_sm = StackMachine::<S>::build(&self.r);
            let g_sm = StackMachine::<S>::build(&self.g);
            let b_sm = StackMachine::<S>::build(&self.b);
            let max_len = *[
                r_sm.instructions.len(),
                g_sm.instructions.len(),
                b_sm.instructions.len(),
            ]
            .iter()
            .max()
            .unwrap();

            let process = |(y_pixel, chunk): (usize, &mut [u8])| {
                let mut stack = vec![S::Vf32::zeroes(); max_len];
                let y = S::Vf32::set1((y_pixel as f32 / h as f32) * 2.0 - 1.0);
                let x_step = 2.0 / (w - 1) as f32;
                let mut x = S::Vf32::zeroes();
                for i in (0..S::Vf32::WIDTH).rev() {
                    x[i] = -1.0 + (x_step * i as f32);
                }
                let x_step = S::Vf32::set1(x_step * S::Vf32::WIDTH as f32);
                let chunk_len = chunk.len();
                for i in (0..w * 4).step_by(S::Vf32::WIDTH * 4) {
                    let (rs, gs, bs) = if self.coord == CoordinateSystem::Cartesian {
                        let rs = (r_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(128.0);
                        let gs = (g_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(128.0);
                        let bs = (b_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(128.0);
                        (rs, gs, bs)
                    } else {
                        let (x, y) = cartesian_to_polar::<S>(x, y);
                        let rs = (r_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(128.0);
                        let gs = (g_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(128.0);
                        let bs = (b_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(128.0);
                        (rs, gs, bs)
                    };

                    for j in 0..S::Vf32::WIDTH {
                        let j4: usize = j * 4;
                        let ij4 = i as usize + j4;
                        if ij4 >= chunk_len {
                            break;
                        }
                        let r = (rs[j] as i32 % 255) as u8;
                        let g = (gs[j] as i32 % 255) as u8;
                        let b = (bs[j] as i32 % 255) as u8;
                        chunk[ij4] = r;
                        chunk[ij4 + 1] = g;
                        chunk[ij4 + 2] = b;
                        chunk[ij4 + 3] = 255_u8;
                    }
                    x += x_step;
                }
            };
            if threaded {
                result
                    .par_chunks_mut(4 * w as usize)
                    .enumerate()
                    .for_each(process);
            } else {
                result
                    .chunks_exact_mut(4 * w as usize)
                    .enumerate()
                    .for_each(process);
            }

            result
        }
    }
    fn simplify<S: Simd>(
        &mut self,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    ) {
        self.r = self.r.constant_fold::<S>(
            &self.coord,
            pics.clone(),
            None,
            None,
            Some(w),
            Some(h),
            Some(t),
        );
        self.g = self.g.constant_fold::<S>(
            &self.coord,
            pics.clone(),
            None,
            None,
            Some(w),
            Some(h),
            Some(t),
        );
        self.b = self.b.constant_fold::<S>(
            &self.coord,
            pics.clone(),
            None,
            None,
            Some(w),
            Some(h),
            Some(t),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pic_new_rgb() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let pic = RGBData::new(0, 60, false, &mut rng, &vec![&"eye.jpg".to_string()]);
        match &pic {
            Pic::RGB(RGBData { r, g, b, coord: _ }) => {
                let len = r.get_children().unwrap().len();
                assert!(len > 0 && len < 60);

                let len = g.get_children().unwrap().len();
                assert!(len > 0 && len < 60);

                let len = b.get_children().unwrap().len();
                assert!(len > 0 && len < 60);
            }
            _ => {
                panic!("wrong type");
            }
        };
    }
}
