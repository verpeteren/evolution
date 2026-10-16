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
pub struct HSVData {
    pub h: APTNode,
    pub s: APTNode,
    pub v: APTNode,
    pub coord: CoordinateSystem,
}

impl PicData for HSVData {
    fn new(min: usize, max: usize, video: bool, rng: &mut StdRng, pic_names: &Vec<&String>) -> Pic {
        let (h, coord) =
            APTNode::create_random_tree(rng.gen_range(min..max), video, rng, pic_names);
        let (s, _coord) =
            APTNode::create_random_tree(rng.gen_range(min..max), video, rng, pic_names);
        let (v, _coord) =
            APTNode::create_random_tree(rng.gen_range(min..max), video, rng, pic_names);
        Pic::HSV(HSVData { h, s, v, coord })
    }
    fn get_rgba8<S: Simd>(
        &self,
        threaded: bool,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    ) -> Vec<u8> {
        unsafe {
            let ts = S::Vf32::set1(t);
            let wf = S::Vf32::set1(w as f32);
            let hf = S::Vf32::set1(h as f32);
            let vec_len = (w * h * 4) as usize;
            let mut result = Vec::<u8>::with_capacity(vec_len);
            result.set_len(vec_len);

            let h_sm = StackMachine::<S>::build(&self.h);
            let s_sm = StackMachine::<S>::build(&self.s);
            let v_sm = StackMachine::<S>::build(&self.v);
            let max_len = *[
                h_sm.instructions.len(),
                s_sm.instructions.len(),
                v_sm.instructions.len(),
            ]
            .iter()
            .max()
            .unwrap();

            let process = |(y_pixel, chunk): (usize, &mut [u8])| {
                let mut stack = Vec::with_capacity(max_len);
                stack.set_len(max_len);
                let y = S::Vf32::set1((y_pixel as f32 / h as f32) * 2.0 - 1.0);
                let x_step = 2.0 / (w - 1) as f32;
                let mut x = S::Vf32::zeroes();
                for i in (0..S::Vf32::WIDTH).rev() {
                    x[i] = -1.0 + (x_step * i as f32);
                }
                let x_step = S::Vf32::set1(x_step * S::Vf32::WIDTH as f32);
                let chunk_len = chunk.len();
                for i in (0..w * 4).step_by(S::Vf32::WIDTH * 4) {
                    let (hs, ss, vs) = if self.coord == CoordinateSystem::Cartesian {
                        let hs = (h_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(0.5);
                        let ss = (s_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(0.5);
                        let vs = (v_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(0.5);
                        (hs, ss, vs)
                    } else {
                        let (x, y) = cartesian_to_polar::<S>(x, y);
                        let hs = (h_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(0.5);
                        let ss = (s_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(0.5);
                        let vs = (v_sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                            + S::Vf32::set1(1.0))
                            * S::Vf32::set1(0.5);
                        (hs, ss, vs)
                    };

                    let (mut rs, mut gs, mut bs) =
                        hsv_to_rgb::<S>(wrap_0_1::<S>(hs), wrap_0_1::<S>(ss), wrap_0_1::<S>(vs));
                    rs = rs * S::Vf32::set1(255.0);
                    gs = gs * S::Vf32::set1(255.0);
                    bs = bs * S::Vf32::set1(255.0);
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
                        chunk[ij4 + 3] = 255 as u8;
                    }
                    x = x + x_step;
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

            //   println!("img elapsed:{}", now.elapsed().as_millis());
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
        self.h = self.h.constant_fold::<S>(
            &self.coord,
            pics.clone(),
            None,
            None,
            Some(w),
            Some(h),
            Some(t),
        );
        self.s = self.s.constant_fold::<S>(
            &self.coord,
            pics.clone(),
            None,
            None,
            Some(w),
            Some(h),
            Some(t),
        );
        self.v = self.v.constant_fold::<S>(
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

#[inline(always)]
fn wrap_0_1<S: Simd>(v: S::Vf32) -> S::Vf32 {
    unsafe {
        let mut r = S::Vf32::zeroes();
        for i in 0..S::Vf32::WIDTH {
            r[i] = v[i] % 1.0001;
        }
        r
    }
}

fn hsv_to_rgb<S: Simd>(h: S::Vf32, s: S::Vf32, v: S::Vf32) -> (S::Vf32, S::Vf32, S::Vf32) {
    unsafe {
        let six = S::Vf32::set1(6.0);
        let one = S::Vf32::set1(1.0);
        let hi = (h * six).fast_floor().cast_i32();
        let f = h * six - hi.cast_f32();
        let p = v * (one - s);
        let q = v * (one - f * s);
        let t = v * (one - (one - f) * s);

        let mut r = S::Vf32::zeroes();
        let mut g = S::Vf32::zeroes();
        let mut b = S::Vf32::zeroes();

        for i in 0..S::Vf32::WIDTH {
            match hi[i] % 6 {
                0 => {
                    r[i] = v[i];
                    g[i] = t[i];
                    b[i] = p[i];
                }
                1 => {
                    r[i] = q[i];
                    g[i] = v[i];
                    b[i] = p[i];
                }
                2 => {
                    r[i] = p[i];
                    g[i] = v[i];
                    b[i] = t[i];
                }
                3 => {
                    r[i] = p[i];
                    g[i] = q[i];
                    b[i] = v[i];
                }
                4 => {
                    r[i] = t[i];
                    g[i] = p[i];
                    b[i] = v[i];
                }
                _ => {
                    r[i] = v[i];
                    g[i] = p[i];
                    b[i] = q[i];
                }
            }
        }

        (r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pic_new_hsv() {
        let mut rng = StdRng::from_rng(rand::thread_rng()).unwrap();
        let pic = HSVData::new(0, 60, false, &mut rng, &vec![&"eye.jpg".to_string()]);
        match &pic {
            Pic::HSV(HSVData { h, s, v, coord: _ }) => {
                let len = h.get_children().unwrap().len();
                assert!(len > 0 && len < 60);

                let len = s.get_children().unwrap().len();
                assert!(len > 0 && len < 60);

                let len = v.get_children().unwrap().len();
                assert!(len > 0 && len < 60);
            }
            _ => {
                panic!("wrong type");
            }
        };
    }
}
