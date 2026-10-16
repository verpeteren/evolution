use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::sync::Arc;

use crate::constants::{
    PIC_GRADIENT_COUNT_MAX, PIC_GRADIENT_COUNT_MIN, PIC_GRADIENT_SIZE, PIC_GRADIENT_STOP_CHANCE,
};
use crate::parser::aptnode::APTNode;
use crate::pic::actual_picture::ActualPicture;
use crate::pic::color::{get_random_color, lerp_color, Color};
use crate::pic::coordinatesystem::{cartesian_to_polar, CoordinateSystem};
use crate::pic::data::PicData;
use crate::pic::pic::Pic;
use crate::vm::stackmachine::StackMachine;

use rayon::prelude::*;
use simdeez::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct GradientData {
    pub colors: Vec<(Color, bool)>,
    pub index: APTNode,
    pub coord: CoordinateSystem,
}

impl PicData for GradientData {
    fn new(min: usize, max: usize, video: bool, rng: &mut StdRng, pic_names: &Vec<&String>) -> Pic {
        //todo cleanup
        //color theory?
        let num_colors = rng.random_range(PIC_GRADIENT_COUNT_MIN..PIC_GRADIENT_COUNT_MAX);
        let mut colors = Vec::with_capacity(num_colors);

        for _ in 0..num_colors {
            let stop = rng.random_range(0..PIC_GRADIENT_STOP_CHANCE);
            if stop == 0 {
                colors.push((get_random_color(rng), true));
            } else {
                colors.push((get_random_color(rng), false));
            }
        }

        let (tree, coord) =
            APTNode::create_random_tree(rng.random_range(min..max), video, rng, pic_names);
        Pic::Gradient(GradientData {
            colors,
            index: tree,
            coord,
        })
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
            let sm = StackMachine::<S>::build(&self.index);
            /*
            let mut min = 999999.0;
            let mut max = -99999.0;
            */

            let color_count = self.colors.iter().filter(|(_, stop)| !stop).count();
            let mut gradient = Vec::<Color>::new(); //todo actually compute this
            let step = (PIC_GRADIENT_SIZE as f32 / color_count as f32) / PIC_GRADIENT_SIZE as f32;
            let mut positions = Vec::<f32>::new();
            positions.push(0.0);
            let mut pos = step;
            for i in 1..self.colors.len() - 1 {
                let (_, stop) = self.colors[i];
                if stop {
                    positions.push(*positions.last().unwrap());
                } else {
                    positions.push(pos);
                    pos += step;
                }
            }
            positions.push(1.0);

            for i in 0..PIC_GRADIENT_SIZE {
                let pct = i as f32 / PIC_GRADIENT_SIZE as f32;
                let color2pos = positions.iter().position(|n| *n >= pct).unwrap();
                if color2pos == 0 {
                    gradient.push(self.colors[0].0);
                } else {
                    let color1 = self.colors[color2pos - 1].0;
                    let color2 = self.colors[color2pos].0;
                    let pct2 = positions[color2pos];
                    let pct1 = positions[color2pos - 1];
                    let range = pct2 - pct1;
                    let pct = (pct - pct1) / range;
                    gradient.push(lerp_color(color1, color2, pct));
                }
            }

            let process = |(y_pixel, chunk): (usize, &mut [u8])| {
                let mut stack = vec![S::Vf32::zeroes(); sm.instructions.len()];

                let y = S::Vf32::set1((y_pixel as f32 / h as f32) * 2.0 - 1.0);
                let x_step = 2.0 / (w - 1) as f32;
                let mut x = S::Vf32::zeroes();
                for i in (0..S::Vf32::WIDTH).rev() {
                    x[i] = -1.0 + (x_step * i as f32);
                }
                let x_step = S::Vf32::set1(x_step * S::Vf32::WIDTH as f32);
                let chunk_len = chunk.len();
                for i in (0..w * 4).step_by(S::Vf32::WIDTH * 4) {
                    let v = if self.coord == CoordinateSystem::Cartesian {
                        sm.execute(&mut stack, pics.clone(), x, y, ts, wf, hf)
                    } else {
                        let (r, theta) = cartesian_to_polar::<S>(x, y);
                        sm.execute(&mut stack, pics.clone(), r, theta, ts, wf, hf)
                    };
                    let scaled_v = (v + S::Vf32::set1(1.0)) * S::Vf32::set1(0.5);
                    let index = (scaled_v * S::Vf32::set1(PIC_GRADIENT_SIZE as f32)).cast_i32();

                    for j in 0..S::Vf32::WIDTH {
                        let j4: usize = j * 4;
                        let ij4 = i as usize + j4;
                        if ij4 >= chunk_len {
                            break;
                        }
                        let c = gradient[index[j] as usize % PIC_GRADIENT_SIZE];
                        chunk[ij4] = (c.r * 255.0) as u8;
                        chunk[ij4 + 1] = (c.g * 255.0) as u8;
                        chunk[ij4 + 2] = (c.b * 255.0) as u8;
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

            // println!("min:{} max:{} range:{}",min,max,max-min);
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
        self.index =
            self.index
                .constant_fold::<S>(&self.coord, pics, None, None, Some(w), Some(h), Some(t));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pic_new_gradient() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let pic = GradientData::new(0, 60, false, &mut rng, &vec![&"eye.jpg".to_string()]);
        match &pic {
            Pic::Gradient(GradientData {
                colors,
                index,
                coord: _coord,
            }) => {
                let len = colors.len();
                assert!(len > 1 && len < 10);
                let len = index.get_children().unwrap().len();
                assert!(len > 0 && len < 60);
            }
            _ => {
                panic!("wrong type");
            }
        };
    }
}
