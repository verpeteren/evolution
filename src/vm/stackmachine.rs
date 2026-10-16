use std::collections::HashMap;
use std::sync::Arc;

use crate::parser::aptnode::APTNode;
use crate::pic::actual_picture::ActualPicture;
use crate::vm::instruction::Instruction;

use simdeez::math::{SimdMathF32BinaryMisc, SimdMathF32Core, SimdMathF32InverseTrig};
use simdeez::prelude::*;
use simdnoise::noise::cell_32::cellular_2d;
use simdnoise::noise::fbm_32::fbm_2d;
use simdnoise::noise::ridge_32::ridge_2d;
use simdnoise::noise::turbulence_32::turbulence_2d;
use simdnoise::{CellDistanceFunction, CellReturnType};

pub struct StackMachine<S: Simd> {
    pub instructions: Vec<Instruction<S>>,
}

impl<S: Simd> StackMachine<S> {
    pub fn get_instruction(node: &APTNode) -> Instruction<S> {
        match node {
            APTNode::Add(_) => Instruction::Add,
            APTNode::Sub(_) => Instruction::Sub,
            APTNode::Mul(_) => Instruction::Mul,
            APTNode::Div(_) => Instruction::Div,
            APTNode::Mod(_) => Instruction::Mod,
            APTNode::FBM(_) => Instruction::FBM,
            APTNode::Ridge(_) => Instruction::Ridge,
            APTNode::Turbulence(_) => Instruction::Turbulence,
            APTNode::Cell1(_) => Instruction::Cell1,
            APTNode::Cell2(_) => Instruction::Cell2,
            APTNode::Sqrt(_) => Instruction::Sqrt,
            APTNode::Sin(_) => Instruction::Sin,
            APTNode::Atan(_) => Instruction::Atan,
            APTNode::Atan2(_) => Instruction::Atan2,
            APTNode::Tan(_) => Instruction::Tan,
            APTNode::Log(_) => Instruction::Log,
            APTNode::Abs(_) => Instruction::Abs,
            APTNode::Floor(_) => Instruction::Floor,
            APTNode::Ceil(_) => Instruction::Ceil,
            APTNode::Clamp(_) => Instruction::Clamp,
            APTNode::Wrap(_) => Instruction::Wrap,
            APTNode::Square(_) => Instruction::Square,
            APTNode::Max(_) => Instruction::Max,
            APTNode::Min(_) => Instruction::Min,
            APTNode::Mandelbrot(_) => Instruction::Mandelbrot,
            APTNode::Picture(name, _) => Instruction::Picture(name.to_string()),
            APTNode::Constant(v) => Instruction::Constant(unsafe { S::Vf32::set1(*v) }),
            APTNode::Width => Instruction::Width,
            APTNode::Height => Instruction::Height,
            APTNode::PI => Instruction::PI,
            APTNode::E => Instruction::E,
            APTNode::X => Instruction::X,
            APTNode::Y => Instruction::Y,
            APTNode::T => Instruction::T,
            APTNode::Empty => panic!("got empty building stack machine"),
        }
    }

    fn build_helper(&mut self, node: &APTNode) {
        match node.get_children() {
            Some(children) => {
                for child in children.iter().rev() {
                    self.build_helper(child);
                }
            }
            None => (),
        }
        let instruction = StackMachine::get_instruction(node);
        //println!("pushing {:?}", node);
        self.instructions.push(instruction)
    }

    pub fn build(node: &APTNode) -> StackMachine<S> {
        let mut sm = StackMachine {
            instructions: Vec::new(),
        };
        sm.build_helper(node);
        sm
    }

    #[inline(always)]
    pub fn deal_with_nan(mut a: S::Vf32) -> S::Vf32 {
        for i in 0..S::Vf32::WIDTH {
            if a[i] == std::f32::INFINITY {
                a[i] = 1.0;
            } else if a[i] == std::f32::NEG_INFINITY {
                a[i] = -1.0;
            } else if a[i].is_nan() {
                a[i] = 0.0;
            }
        }
        a
    }

    pub fn execute(
        &self,
        stack: &mut Vec<S::Vf32>,
        pics: Arc<HashMap<String, ActualPicture>>,
        x: S::Vf32,
        y: S::Vf32,
        t: S::Vf32,
        w: S::Vf32,
        h: S::Vf32,
    ) -> S::Vf32 {
        unsafe {
            let mut sp = 0;
            for ins in &self.instructions {
                match ins {
                    Instruction::Add => {
                        sp -= 1;
                        stack[sp - 1] = stack[sp] + stack[sp - 1];
                    }
                    Instruction::Sub => {
                        sp -= 1;
                        stack[sp - 1] = stack[sp] - stack[sp - 1];
                    }
                    Instruction::Mul => {
                        sp -= 1;
                        stack[sp - 1] = stack[sp] * stack[sp - 1];
                    }
                    Instruction::Div => {
                        sp -= 1;
                        stack[sp - 1] = StackMachine::<S>::deal_with_nan(stack[sp] / stack[sp - 1]);
                    }
                    Instruction::Mod => {
                        sp -= 1;
                        let a = stack[sp - 1];
                        let b = stack[sp];
                        let mut r = S::Vf32::zeroes();
                        for i in 0..S::Vf32::WIDTH {
                            r[i] = a[i] % b[i];
                        }
                        stack[sp - 1] = StackMachine::<S>::deal_with_nan(r);
                    }
                    Instruction::FBM => {
                        sp -= 5;
                        let xfreq = stack[sp - 1] * S::Vf32::set1(15.0);
                        let yfreq = stack[sp + 4] * S::Vf32::set1(15.0);
                        let lacunarity = stack[sp + 2] * S::Vf32::set1(5.0);
                        let gain = stack[sp + 3] * S::Vf32::set1(0.5);
                        let octaves = 3;
                        stack[sp - 1] = fbm_2d::<S>(
                            stack[sp + 1] * xfreq,
                            stack[sp] * yfreq,
                            lacunarity,
                            gain,
                            octaves,
                            3,
                        );
                    }
                    Instruction::Ridge => {
                        sp -= 5;
                        let xfreq = stack[sp - 1] * S::Vf32::set1(15.0);
                        let yfreq = stack[sp + 4] * S::Vf32::set1(15.0);
                        let lacunarity = stack[sp + 2] * S::Vf32::set1(5.0);
                        let gain = stack[sp + 3] * S::Vf32::set1(0.5);
                        let octaves = 3;
                        stack[sp - 1] = ridge_2d::<S>(
                            stack[sp + 1] * xfreq,
                            stack[sp] * yfreq,
                            lacunarity,
                            gain,
                            octaves,
                            3,
                        );
                    }
                    Instruction::Turbulence => {
                        sp -= 5;
                        let xfreq = stack[sp - 1] * S::Vf32::set1(15.0);
                        let yfreq = stack[sp + 4] * S::Vf32::set1(15.0);
                        let lacunarity = stack[sp + 2] * S::Vf32::set1(5.0);
                        let gain = stack[sp + 3] * S::Vf32::set1(0.5);
                        let octaves = 3;
                        stack[sp - 1] = turbulence_2d::<S>(
                            stack[sp + 1] * xfreq,
                            stack[sp] * yfreq,
                            lacunarity,
                            gain,
                            octaves,
                            3,
                        );
                    }
                    Instruction::Cell1 => {
                        sp -= 4;
                        let xfreq = stack[sp - 1] * S::Vf32::set1(4.0);
                        let yfreq = stack[sp + 3] * S::Vf32::set1(4.0);
                        let jitter = stack[sp + 2] * S::Vf32::set1(0.5);
                        stack[sp - 1] = cellular_2d::<S>(
                            stack[sp + 1] * xfreq,
                            stack[sp] * yfreq,
                            CellDistanceFunction::Euclidean,
                            CellReturnType::Distance,
                            jitter,
                            1,
                        );
                    }
                    Instruction::Cell2 => {
                        sp -= 4;
                        let xfreq = stack[sp - 1] * S::Vf32::set1(4.0);
                        let yfreq = stack[sp + 3] * S::Vf32::set1(4.0);
                        let jitter = stack[sp + 2] * S::Vf32::set1(0.5);
                        stack[sp - 1] = cellular_2d::<S>(
                            stack[sp + 1] * xfreq,
                            stack[sp] * yfreq,
                            CellDistanceFunction::Euclidean,
                            CellReturnType::CellValue,
                            jitter,
                            1,
                        );
                    }
                    Instruction::Sqrt => {
                        let v = stack[sp - 1];
                        let positive = v.sqrt();
                        let negative = S::Vf32::set1(-1.0) * v.abs().sqrt();
                        let mask = v.cmp_gte(S::Vf32::zeroes());
                        stack[sp - 1] = mask.blendv(negative, positive);
                    }
                    Instruction::Sin => {
                        stack[sp - 1] =
                            (stack[sp - 1] * S::Vf32::set1(std::f32::consts::PI)).sin_u35();
                    }
                    Instruction::Atan => {
                        stack[sp - 1] = (stack[sp - 1] * S::Vf32::set1(4.0)).atan_u35()
                            * S::Vf32::set1(0.666666666);
                    }
                    Instruction::Atan2 => {
                        sp -= 1;
                        let x = stack[sp - 1];
                        let y = stack[sp] * S::Vf32::set1(4.0);
                        stack[sp - 1] =
                            y.atan2_u35(x) * S::Vf32::set1(std::f32::consts::FRAC_1_PI);
                    }
                    Instruction::Tan => {
                        stack[sp - 1] = (stack[sp - 1]
                            * S::Vf32::set1(std::f32::consts::FRAC_PI_2))
                        .tan_u35();
                    }
                    Instruction::Log => {
                        let v = stack[sp - 1] * S::Vf32::set1(4.0);
                        let positive = v.ln_u35();
                        let negative = S::Vf32::set1(-1.0) * v.abs().ln_u35();
                        let mask = v.cmp_gte(S::Vf32::zeroes());
                        stack[sp - 1] = mask.blendv(negative, positive)
                            * S::Vf32::set1(1.0 / std::f32::consts::E);
                    }
                    Instruction::Abs => {
                        stack[sp - 1] = stack[sp - 1].abs();
                    }
                    Instruction::Floor => {
                        stack[sp - 1] = stack[sp - 1].fast_floor();
                    }
                    Instruction::Ceil => {
                        stack[sp - 1] = stack[sp - 1].fast_ceil();
                    }
                    Instruction::Clamp => {
                        let mut v = stack[sp - 1];
                        for i in 0..S::Vf32::WIDTH {
                            if v[i] > 1.0 {
                                v[i] = 1.0
                            } else if v[i] < -1.0 {
                                v[i] = -1.0
                            }
                        }
                        stack[sp - 1] = v;
                    }
                    Instruction::Wrap => {
                        let mut v = stack[sp - 1];
                        for i in 0..S::Vf32::WIDTH {
                            if v[i] < -1.0 || v[i] > 1.0 {
                                let t = (v[i] + 1.0) / 2.0;
                                v[i] = -1.0 + 2.0 * (t - t.floor());
                            }
                        }
                        stack[sp - 1] = v;
                    }
                    Instruction::Square => {
                        let v = stack[sp - 1];
                        stack[sp - 1] = v * v;
                    }
                    Instruction::Max => {
                        sp -= 1;
                        stack[sp - 1] = stack[sp - 1].max(stack[sp]);
                    }
                    Instruction::Min => {
                        sp -= 1;
                        stack[sp - 1] = stack[sp - 1].min(stack[sp]);
                    }
                    Instruction::Mandelbrot => {
                        sp -= 1;
                        //todo do
                    }
                    Instruction::Picture(name) => {
                        sp -= 1;

                        let y = stack[sp - 1];
                        let x = stack[sp];

                        let picture = &pics[name];
                        let w = S::Vi32::set1(picture.w as i32);
                        let h = S::Vi32::set1(picture.h as i32);
                        let wf = w.cast_f32();
                        let hf = h.cast_f32();
                        let mut xpct = (x + S::Vf32::set1(1.0)) / S::Vf32::set1(2.0);
                        let mut ypct = (y + S::Vf32::set1(1.0)) / S::Vf32::set1(2.0);
                        for i in 0..S::Vf32::WIDTH {
                            xpct[i] = xpct[i] % 1.0;
                            ypct[i] = ypct[i] % 1.0;
                        }
                        let xi = (xpct * wf).cast_i32();
                        let yi = (ypct * hf).cast_i32();
                        let index = xi + w * yi;

                        // println!("w:{:?} h{:?} xpct:{:?} ypct:{:?} index:{},{}",w[0],h[0],xpct[0],ypct[0],index[0],index[1]);
                        let brightness_len = picture.brightness.len();
                        for i in 0..S::Vf32::WIDTH {
                            let slot: usize =
                                index[i] as usize % (picture.w as usize * picture.h as usize);
                            if slot >= brightness_len {
                                break;
                            }
                            stack[sp - 1][i] = picture.brightness[slot];
                        }
                    }
                    Instruction::Constant(v) => {
                        stack[sp] = *v;
                        sp += 1;
                    }
                    Instruction::Width => {
                        stack[sp] = w;
                        sp += 1;
                    }
                    Instruction::Height => {
                        stack[sp] = h;
                        sp += 1;
                    }
                    Instruction::PI => {
                        let v = S::Vf32::set1(std::f32::consts::PI);
                        stack[sp] = v;
                        sp += 1;
                    }
                    Instruction::E => {
                        let v = S::Vf32::set1(std::f32::consts::E);
                        stack[sp] = v;
                        sp += 1;
                    }
                    Instruction::X => {
                        stack[sp] = x;
                        sp += 1;
                    }
                    Instruction::Y => {
                        stack[sp] = y;
                        sp += 1;
                    }
                    Instruction::T => {
                        stack[sp] = t;
                        sp += 1;
                    }
                }
            }
            stack[sp - 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::aptnode::mock;

    use super::*;

    simd_runtime_generate!(
        fn impl_stackmachine_get_instruction() {
            match StackMachine::<S>::get_instruction(&APTNode::Add(mock::mock_params_add(true))) {
                Instruction::Add => {}
                _ => {
                    panic!("Unexpected result");
                }
            };
            match StackMachine::<S>::get_instruction(&APTNode::Sub(mock::mock_params_sub(true))) {
                Instruction::Sub => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Mul(mock::mock_params_mul(true))) {
                Instruction::Mul => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Div(mock::mock_params_div(true))) {
                Instruction::Div => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::FBM(mock::mock_params_fbm(true))) {
                Instruction::FBM => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Ridge(mock::mock_params_ridge(true)))
            {
                Instruction::Ridge => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Cell1(mock::mock_params_cell1(true)))
            {
                Instruction::Cell1 => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Cell2(mock::mock_params_cell2(true)))
            {
                Instruction::Cell2 => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Turbulence(
                mock::mock_params_turbulence(true),
            )) {
                Instruction::Turbulence => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Sqrt(mock::mock_params_sqrt(true))) {
                Instruction::Sqrt => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Sin(mock::mock_params_sin(true))) {
                Instruction::Sin => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Atan(mock::mock_params_atan(true))) {
                Instruction::Atan => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Atan2(mock::mock_params_atan2(true)))
            {
                Instruction::Atan2 => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Tan(mock::mock_params_tan(true))) {
                Instruction::Tan => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Log(mock::mock_params_log(true))) {
                Instruction::Log => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Abs(mock::mock_params_abs(true))) {
                Instruction::Abs => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Floor(mock::mock_params_floor(true)))
            {
                Instruction::Floor => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Ceil(mock::mock_params_ceil(true))) {
                Instruction::Ceil => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Clamp(mock::mock_params_clamp(true)))
            {
                Instruction::Clamp => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Wrap(mock::mock_params_wrap(true))) {
                Instruction::Wrap => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Square(mock::mock_params_square(
                true,
            ))) {
                Instruction::Square => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Max(mock::mock_params_max(true))) {
                Instruction::Max => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Min(mock::mock_params_min(true))) {
                Instruction::Min => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Mod(mock::mock_params_mod(true))) {
                Instruction::Mod => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Mandelbrot(
                mock::mock_params_mandelbrot(true),
            )) {
                Instruction::Mandelbrot => {}
                _ => {
                    panic!("Unexpected result");
                }
            }

            let name = "eye.jpg".to_string();
            match StackMachine::<S>::get_instruction(&APTNode::Picture(
                name.clone(),
                mock::mock_params_picture(true),
            )) {
                Instruction::Picture(got) => {
                    assert_eq!(got, name);
                }
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Constant(6.0)) {
                Instruction::Constant(_) => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Width) {
                Instruction::Width => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Height) {
                Instruction::Height => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::PI) {
                Instruction::PI => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::E) {
                Instruction::E => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::X) {
                Instruction::X => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::Y) {
                Instruction::Y => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
            match StackMachine::<S>::get_instruction(&APTNode::T) {
                Instruction::T => {}
                _ => {
                    panic!("Unexpected result");
                }
            }
        }
    );

    #[test]
    fn test_stackmachine_get_instruction() {
        impl_stackmachine_get_instruction();
    }

    simd_runtime_generate!(
        fn impl_stackmachine_build() {
            let sm = StackMachine::<S>::build(&APTNode::Add(vec![
                APTNode::Constant(2.0),
                APTNode::Cell1(vec![
                    APTNode::Constant(1.2),
                    APTNode::X,
                    APTNode::Y,
                    APTNode::T,
                ]),
            ]));
            assert_eq!(sm.instructions.len(), 7);
            //assert_eq!(sm.instructions.get(0).unwrap(), &Instruction::T);
        }
    );

    #[test]
    fn test_stackmachine_build() {
        impl_stackmachine_build();
    }

    simd_runtime_generate!(
        fn impl_stackmachine_deal_with_nan() {
            unsafe {
                let zeros = S::Vf32::set1(0.0);
                let ones = S::Vf32::set1(1.0);
                let neg_ones = S::Vf32::set1(-1.0);
                let neg_infs = S::Vf32::set1(std::f32::NEG_INFINITY);
                let infs = S::Vf32::set1(std::f32::INFINITY);
                let nans = S::Vf32::set1(std::f32::NAN);

                assert_eq!(StackMachine::<S>::deal_with_nan(ones)[0], ones[0]);
                assert_eq!(StackMachine::<S>::deal_with_nan(zeros)[0], zeros[0]);
                assert_eq!(StackMachine::<S>::deal_with_nan(neg_infs)[0], neg_ones[0]);
                assert_eq!(StackMachine::<S>::deal_with_nan(infs)[0], ones[0]);
                assert_eq!(StackMachine::<S>::deal_with_nan(nans)[0], zeros[0]);
            }
        }
    );

    #[test]
    fn test_stackmachine_deal_with_nan() {
        impl_stackmachine_deal_with_nan();
    }
}
