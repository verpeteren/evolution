use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use crate::constants::MAX_PARSE_DEPTH;
use crate::parser::token::Token;
use crate::pic::actual_picture::ActualPicture;
use crate::pic::coordinatesystem::{cartesian_to_polar, CoordinateSystem};
use crate::vm::stackmachine::StackMachine;

use rand::prelude::*;
use simdeez::prelude::*;
use variant_count::VariantCount;

#[derive(VariantCount, Clone, Debug, PartialEq)]
pub enum APTNode {
    Add(Vec<APTNode>),
    Sub(Vec<APTNode>),
    Mul(Vec<APTNode>),
    Div(Vec<APTNode>),
    Mod(Vec<APTNode>),
    FBM(Vec<APTNode>),
    Ridge(Vec<APTNode>),
    Turbulence(Vec<APTNode>),
    Cell1(Vec<APTNode>),
    Cell2(Vec<APTNode>),
    Sqrt(Vec<APTNode>),
    Sin(Vec<APTNode>),
    Atan(Vec<APTNode>),
    Atan2(Vec<APTNode>),
    Tan(Vec<APTNode>),
    Log(Vec<APTNode>),
    Abs(Vec<APTNode>),
    Floor(Vec<APTNode>),
    Ceil(Vec<APTNode>),
    Clamp(Vec<APTNode>),
    Wrap(Vec<APTNode>),
    Square(Vec<APTNode>),
    Max(Vec<APTNode>),
    Min(Vec<APTNode>),
    Mandelbrot(Vec<APTNode>),
    Picture(String, Vec<APTNode>),
    Constant(f32),
    Width,
    Height,
    PI,
    E,
    X,
    Y,
    T,
    Empty,
}

impl APTNode {
    pub fn to_lisp(&self) -> String {
        match self {
            APTNode::Add(children) => {
                format!("( + {} {} )", children[0].to_lisp(), children[1].to_lisp())
            }
            APTNode::Sub(children) => {
                format!("( - {} {} )", children[0].to_lisp(), children[1].to_lisp())
            }
            APTNode::Mul(children) => {
                format!("( * {} {} )", children[0].to_lisp(), children[1].to_lisp())
            }
            APTNode::Div(children) => {
                format!("( / {} {} )", children[0].to_lisp(), children[1].to_lisp())
            }
            APTNode::Mod(children) => {
                format!("( % {} {} )", children[0].to_lisp(), children[1].to_lisp())
            }
            APTNode::FBM(children) => format!(
                "( FBM {} {} {} {} {} {} )",
                children[0].to_lisp(),
                children[1].to_lisp(),
                children[2].to_lisp(),
                children[3].to_lisp(),
                children[4].to_lisp(),
                children[5].to_lisp()
            ),
            APTNode::Ridge(children) => format!(
                "( RIDGE {} {} {} {} {} {} )",
                children[0].to_lisp(),
                children[1].to_lisp(),
                children[2].to_lisp(),
                children[3].to_lisp(),
                children[4].to_lisp(),
                children[5].to_lisp()
            ),
            APTNode::Cell1(children) => format!(
                "( CELL1 {} {} {} {} {} )",
                children[0].to_lisp(),
                children[1].to_lisp(),
                children[2].to_lisp(),
                children[3].to_lisp(),
                children[4].to_lisp()
            ),
            APTNode::Cell2(children) => format!(
                "( CELL2 {} {} {} {} {} )",
                children[0].to_lisp(),
                children[1].to_lisp(),
                children[2].to_lisp(),
                children[3].to_lisp(),
                children[4].to_lisp()
            ),
            APTNode::Turbulence(children) => format!(
                "( TURBULENCE {} {} {} {} {} {} )",
                children[0].to_lisp(),
                children[1].to_lisp(),
                children[2].to_lisp(),
                children[3].to_lisp(),
                children[4].to_lisp(),
                children[5].to_lisp()
            ),
            APTNode::Sqrt(children) => format!("( SQRT {} )", children[0].to_lisp()),
            APTNode::Sin(children) => format!("( SIN {} )", children[0].to_lisp()),
            APTNode::Atan(children) => format!("( ATAN {} )", children[0].to_lisp()),
            APTNode::Atan2(children) => format!(
                "( ATAN2 {} {} )",
                children[0].to_lisp(),
                children[1].to_lisp()
            ),
            APTNode::Tan(children) => format!("( TAN {} )", children[0].to_lisp()),
            APTNode::Log(children) => format!("( LOG {} )", children[0].to_lisp()),
            APTNode::Abs(children) => format!("( ABS {} )", children[0].to_lisp()),
            APTNode::Floor(children) => format!("( FLOOR {} )", children[0].to_lisp()),
            APTNode::Ceil(children) => format!("( CEIL {} )", children[0].to_lisp()),
            APTNode::Clamp(children) => format!("( CLAMP {} )", children[0].to_lisp()),
            APTNode::Wrap(children) => format!("( WRAP {} )", children[0].to_lisp()),
            APTNode::Square(children) => format!("( SQUARE {} )", children[0].to_lisp()),
            APTNode::Max(children) => format!(
                "( MAX {} {} )",
                children[0].to_lisp(),
                children[1].to_lisp()
            ),
            APTNode::Min(children) => format!(
                "( MIN {} {} )",
                children[0].to_lisp(),
                children[1].to_lisp()
            ),
            APTNode::Mandelbrot(children) => format!(
                "( MANDELBROT {} {} )",
                children[0].to_lisp(),
                children[1].to_lisp()
            ),
            APTNode::Picture(name, children) => format!(
                "( PIC-{} {} {} )",
                name,
                children[0].to_lisp(),
                children[1].to_lisp()
            ),
            APTNode::Constant(v) => {
                if v == &std::f32::consts::PI {
                    "PI".to_string()
                } else if v == &std::f32::consts::E {
                    "E".to_string()
                } else {
                    format!("{}", v)
                }
            }
            APTNode::Width => "WIDTH".to_string(),
            APTNode::Height => "HEIGHT".to_string(),
            APTNode::PI => "PI".to_string(),
            APTNode::E => "E".to_string(),
            APTNode::X => "X".to_string(),
            APTNode::Y => "Y".to_string(),
            APTNode::T => "T".to_string(),
            APTNode::Empty => "EMPTY".to_string(),
        }
    }

    pub fn str_to_node(s: &str) -> Result<APTNode, String> {
        let lower = &s.to_lowercase()[..];
        match lower {
            "+" => Ok(APTNode::Add(vec![APTNode::Empty, APTNode::Empty])),
            "-" => Ok(APTNode::Sub(vec![APTNode::Empty, APTNode::Empty])),
            "*" => Ok(APTNode::Mul(vec![APTNode::Empty, APTNode::Empty])),
            "/" => Ok(APTNode::Div(vec![APTNode::Empty, APTNode::Empty])),
            "%" => Ok(APTNode::Mod(vec![APTNode::Empty, APTNode::Empty])),
            "fbm" => Ok(APTNode::FBM(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ])),
            "ridge" => Ok(APTNode::Ridge(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ])),
            "turbulence" => Ok(APTNode::Turbulence(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ])),
            "cell1" => Ok(APTNode::Cell1(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ])),
            "cell2" => Ok(APTNode::Cell2(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ])),
            "sqrt" => Ok(APTNode::Sqrt(vec![APTNode::Empty])),
            "sin" => Ok(APTNode::Sin(vec![APTNode::Empty])),
            "atan" => Ok(APTNode::Atan(vec![APTNode::Empty])),
            "atan2" => Ok(APTNode::Atan2(vec![APTNode::Empty, APTNode::Empty])),
            "tan" => Ok(APTNode::Tan(vec![APTNode::Empty])),
            "log" => Ok(APTNode::Log(vec![APTNode::Empty])),
            "abs" => Ok(APTNode::Abs(vec![APTNode::Empty])),
            "floor" => Ok(APTNode::Floor(vec![APTNode::Empty])),
            "ceil" => Ok(APTNode::Ceil(vec![APTNode::Empty])),
            "clamp" => Ok(APTNode::Clamp(vec![APTNode::Empty])),
            "wrap" => Ok(APTNode::Wrap(vec![APTNode::Empty])),
            "square" => Ok(APTNode::Square(vec![APTNode::Empty])),
            "max" => Ok(APTNode::Max(vec![APTNode::Empty, APTNode::Empty])),
            "min" => Ok(APTNode::Min(vec![APTNode::Empty, APTNode::Empty])),
            "mandelbrot" => Ok(APTNode::Mandelbrot(vec![APTNode::Empty, APTNode::Empty])),
            "width" => Ok(APTNode::Width),
            "height" => Ok(APTNode::Height),
            "pi" => Ok(APTNode::PI),
            "e" => Ok(APTNode::E),
            "x" => Ok(APTNode::X),
            "y" => Ok(APTNode::Y),
            "t" => Ok(APTNode::T),
            _ => {
                if let Some(name) = lower.strip_prefix("pic-") {
                    let name = name.to_owned();
                    Ok(APTNode::Picture(name, vec![APTNode::Empty, APTNode::Empty]))
                } else {
                    Err(format!("Unknown operation '{}' ", s))
                }
            }
        }
    }
    pub fn pick_random_coord(rng: &mut StdRng) -> CoordinateSystem {
        let r = rng.random_range(0..CoordinateSystem::VARIANT_COUNT);

        match r {
            0 => CoordinateSystem::Polar,
            1 => CoordinateSystem::Cartesian,
            _ => panic!("pick_random_coord generated unhandled r:{}", r),
        }
    }

    pub fn pick_random_node(rng: &mut StdRng, pic_names: &[&String]) -> APTNode {
        let ignore_variant_count = 9;
        let ignore_pictures = if pic_names.is_empty() { 1 } else { 0 };
        let r = rng.random_range(0..APTNode::VARIANT_COUNT - ignore_variant_count - ignore_pictures);

        match r {
            0 => APTNode::Add(vec![APTNode::Empty, APTNode::Empty]),
            1 => APTNode::Sub(vec![APTNode::Empty, APTNode::Empty]),
            2 => APTNode::Mul(vec![APTNode::Empty, APTNode::Empty]),
            3 => APTNode::Div(vec![APTNode::Empty, APTNode::Empty]),
            4 => APTNode::Mod(vec![APTNode::Empty, APTNode::Empty]),
            5 => APTNode::FBM(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ]),
            6 => APTNode::Ridge(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ]),
            7 => APTNode::Turbulence(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ]),
            8 => APTNode::Cell1(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ]),
            9 => APTNode::Cell2(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
            ]),
            10 => APTNode::Sqrt(vec![APTNode::Empty]),
            11 => APTNode::Sin(vec![APTNode::Empty]),
            12 => APTNode::Atan(vec![APTNode::Empty]),
            13 => APTNode::Atan2(vec![APTNode::Empty, APTNode::Empty]),
            14 => APTNode::Tan(vec![APTNode::Empty]),
            15 => APTNode::Log(vec![APTNode::Empty]),
            16 => APTNode::Abs(vec![APTNode::Empty]),
            17 => APTNode::Floor(vec![APTNode::Empty]),
            18 => APTNode::Ceil(vec![APTNode::Empty]),
            19 => APTNode::Clamp(vec![APTNode::Empty]),
            20 => APTNode::Wrap(vec![APTNode::Empty]),
            21 => APTNode::Square(vec![APTNode::Empty]),
            22 => APTNode::Max(vec![APTNode::Empty, APTNode::Empty]),
            23 => APTNode::Min(vec![APTNode::Empty, APTNode::Empty]),
            24 => APTNode::Mandelbrot(vec![APTNode::Empty, APTNode::Empty]),
            // Pictures should be the last one (see _ignore_pictures variable)
            25 => {
                let r = rng.random_range(0..pic_names.len());
                APTNode::Picture(
                    pic_names[r].to_string(),
                    vec![APTNode::Empty, APTNode::Empty],
                )
            }
            _ => panic!("pick_random_node generated unhandled r:{}", r),
        }
    }

    pub fn pick_random_leaf(rng: &mut StdRng) -> APTNode {
        let r = rng.random_range(0..3);
        match r {
            0 => APTNode::X,
            1 => APTNode::Y,
            2 => APTNode::Constant(rng.random_range(-1.0..1.0)),
            _ => panic!("pick_random_leaf generated unhandled r:{}", r),
        }
    }

    pub fn pick_random_leaf_video(rng: &mut StdRng) -> APTNode {
        let r = rng.random_range(0..4);
        match r {
            0 => APTNode::X,
            1 => APTNode::Y,
            2 => APTNode::T,
            3 => APTNode::Constant(rng.random_range(-1.0..1.0)),
            _ => panic!("pick_random_leaf generated unhandled r:{}", r),
        }
    }

    pub fn add_random(&mut self, node: APTNode, rng: &mut StdRng) {
        let children = match self.get_children_mut() {
            Some(children) => children,
            None => panic!("tried to add_random to a leaf"),
        };
        let add_index = rng.random_range(0..children.len());
        match children[add_index] {
            APTNode::Empty => children[add_index] = node,
            _ => children[add_index].add_random(node, rng),
        }
    }

    pub fn add_leaf(&mut self, leaf: &APTNode) -> bool {
        match self.get_children_mut() {
            None => false,
            Some(children) => {
                for child in children.iter_mut() {
                    match child {
                        APTNode::Empty => {
                            *child = leaf.clone();
                            return true;
                        }
                        _ => {
                            if !child.is_leaf() && child.add_leaf(leaf) {
                                return true;
                            }
                        }
                    }
                }
                false
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn constant_eval<S: Simd>(
        &self,
        coord: &CoordinateSystem,
        pics: Arc<HashMap<String, ActualPicture>>,
        x: Option<u32>,
        y: Option<u32>,
        w: Option<u32>,
        h: Option<u32>,
        t: Option<f32>,
    ) -> f32 {
        match self {
            APTNode::Width => match w {
                Some(value) => value as f32,
                None => {
                    panic!("invalid node passed to constant_esval")
                }
            },
            APTNode::Height => match h {
                Some(value) => value as f32,
                None => {
                    panic!("invalid node passed to constant_esval")
                }
            },
            APTNode::T => match t {
                Some(value) => value,
                None => {
                    panic!("invalid node passed to constant_esval")
                }
            },
            APTNode::X => match x {
                Some(value) => value as f32,
                None => {
                    panic!("invalid node passed to constant_esval")
                }
            },
            APTNode::Y => match y {
                Some(value) => value as f32,
                None => {
                    panic!("invalid node passed to constant_esval")
                }
            },
            APTNode::PI => std::f32::consts::PI,
            APTNode::E => std::f32::consts::E,
            APTNode::Constant(v) => *v,
            APTNode::Add(children)
            | APTNode::Sub(children)
            | APTNode::Mul(children)
            | APTNode::Div(children)
            | APTNode::Mod(children)
            | APTNode::FBM(children)
            | APTNode::Ridge(children)
            | APTNode::Turbulence(children)
            | APTNode::Cell1(children)
            | APTNode::Cell2(children)
            | APTNode::Sqrt(children)
            | APTNode::Sin(children)
            | APTNode::Atan(children)
            | APTNode::Atan2(children)
            | APTNode::Tan(children)
            | APTNode::Log(children)
            | APTNode::Abs(children)
            | APTNode::Floor(children)
            | APTNode::Ceil(children)
            | APTNode::Clamp(children)
            | APTNode::Wrap(children)
            | APTNode::Square(children)
            | APTNode::Max(children)
            | APTNode::Min(children)
            | APTNode::Mandelbrot(children)
            | APTNode::Picture(_, children) => {
                let mut sx = S::Vf32::set1(0.0);
                let mut sy = S::Vf32::set1(0.0);
                let mut st = S::Vf32::set1(0.0);
                let mut sw = S::Vf32::set1(0.0);
                let mut sh = S::Vf32::set1(0.0);
                children.iter().for_each(|a| match a {
                    APTNode::Width => match w {
                        None => panic!("invalid node passed to constant_esval"),
                        Some(value) => sw = S::Vf32::set1(value as f32),
                    },
                    APTNode::Height => match h {
                        None => panic!("invalid node passed to constant_esval"),
                        Some(value) => sh = S::Vf32::set1(value as f32),
                    },
                    APTNode::T => match t {
                        None => panic!("invalid node passed to constant_esval"),
                        Some(value) => st = S::Vf32::set1(value),
                    },
                    APTNode::X => match x {
                        None => panic!("invalid node passed to constant_esval"),
                        Some(value) => sx = S::Vf32::set1(value as f32),
                    },
                    APTNode::Y => match y {
                        None => panic!("invalid node passed to constant_esval"),
                        Some(value) => sy = S::Vf32::set1(value as f32),
                    },
                    _ => {}
                });
                let sm = StackMachine::<S>::build(self);
                let mut stack = vec![S::Vf32::zeroes(); sm.instructions.len()];

                let v = if coord == &CoordinateSystem::Cartesian {
                    sm.execute(&mut stack, pics, sx, sy, st, sw, sh)
                } else {
                    let (r, theta) = cartesian_to_polar::<S>(sx, sy);
                    sm.execute(&mut stack, pics, r, theta, st, sw, sh)
                };
                v[0] as f32
            },
            _ => panic!("invalid node passed to constant_esval"),
        }
    }

    fn set_children(&self, children: Vec<APTNode>) -> Self {
        match self {
            APTNode::Add(_) => APTNode::Add(children),
            APTNode::Sub(_) => APTNode::Sub(children),
            APTNode::Mul(_) => APTNode::Mul(children),
            APTNode::Div(_) => APTNode::Div(children),
            APTNode::Mod(_) => APTNode::Mod(children),
            APTNode::FBM(_) => APTNode::FBM(children),
            APTNode::Ridge(_) => APTNode::Ridge(children),
            APTNode::Turbulence(_) => APTNode::Turbulence(children),
            APTNode::Cell1(_) => APTNode::Cell1(children),
            APTNode::Cell2(_) => APTNode::Cell2(children),
            APTNode::Sqrt(_) => APTNode::Sqrt(children),
            APTNode::Sin(_) => APTNode::Sin(children),
            APTNode::Atan(_) => APTNode::Atan(children),
            APTNode::Atan2(_) => APTNode::Atan2(children),
            APTNode::Tan(_) => APTNode::Tan(children),
            APTNode::Log(_) => APTNode::Log(children),
            APTNode::Abs(_) => APTNode::Abs(children),
            APTNode::Floor(_) => APTNode::Floor(children),
            APTNode::Ceil(_) => APTNode::Ceil(children),
            APTNode::Clamp(_) => APTNode::Clamp(children),
            APTNode::Wrap(_) => APTNode::Wrap(children),
            APTNode::Square(_) => APTNode::Square(children),
            APTNode::Max(_) => APTNode::Max(children),
            APTNode::Min(_) => APTNode::Min(children),
            APTNode::Mandelbrot(_) => APTNode::Mandelbrot(children),
            APTNode::Picture(name, _) => APTNode::Picture(name.to_string(), children[1..].to_vec()),
            APTNode::Constant(v) => APTNode::Constant(*v),
            APTNode::Width => APTNode::Width,
            APTNode::Height => APTNode::Height,
            APTNode::PI => APTNode::PI,
            APTNode::E => APTNode::E,
            APTNode::X => APTNode::X,
            APTNode::Y => APTNode::Y,
            APTNode::T => APTNode::T,
            APTNode::Empty => panic!("tried to eval an empty node"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn constant_fold<S: Simd>(
        &self,
        coord: &CoordinateSystem,
        pics: Arc<HashMap<String, ActualPicture>>,
        x: Option<u32>,
        y: Option<u32>,
        w: Option<u32>,
        h: Option<u32>,
        t: Option<f32>,
    ) -> APTNode {
        match (self, x, y, w, h, t) {
            (APTNode::Constant(v), _, _, _, _, _) => APTNode::Constant(*v),
            (APTNode::E, _, _, _, _, _) => APTNode::Constant(std::f32::consts::E),
            (APTNode::PI, _, _, _, _, _) => APTNode::Constant(std::f32::consts::PI),
            (APTNode::X, None, _, _, _, _) => APTNode::X,
            (APTNode::Y, _, None, _, _, _) => APTNode::Y,
            (APTNode::Width, _, _, None, _, _) => APTNode::Width,
            (APTNode::Height, _, _, _, None, _) => APTNode::Height,
            (APTNode::T, _, _, _, _, None) => APTNode::T,
            (APTNode::X, Some(v), _, _, _, _) => APTNode::Constant(v as f32),
            (APTNode::Y, _, Some(v), _, _, _) => APTNode::Constant(v as f32),
            (APTNode::Width, _, _, Some(v), _, _) => APTNode::Constant(v as f32),
            (APTNode::Height, _, _, _, Some(v), _) => APTNode::Constant(v as f32),
            (APTNode::T, _, _, _, _, Some(v)) => APTNode::Constant(v),
            (APTNode::Picture(name, children), _, _, _, _, _) => {
                APTNode::Picture(name.to_string(), children.clone())
            }
            _ => {
                let children = self.get_children().unwrap();
                //foreach child -> constant_fold(child), if you get back all constants -> compute the new constant, and create it
                let folded_children: Vec<APTNode> = children
                    .iter()
                    .map(|child| child.constant_fold::<S>(coord, pics.clone(), x, y, w, h, t))
                    .collect();
                if folded_children
                    .iter()
                    .all(|child| matches!(child, APTNode::Constant(_)))
                {
                    let clone = self.set_children(folded_children);
                    APTNode::Constant(clone.constant_eval::<S>(coord, pics.clone(), x, y, w, h, t))
                } else {
                    
                    self.set_children(folded_children)
                }
            }
        }
    }

    pub fn create_random_tree(
        count: usize,
        video: bool,
        rng: &mut StdRng,
        pic_names: &Vec<&String>,
    ) -> (APTNode, CoordinateSystem) {
        let coord = APTNode::pick_random_coord(rng);
        let leaf_func = if video {
            APTNode::pick_random_leaf_video
        } else {
            APTNode::pick_random_leaf
        };
        let mut first = APTNode::pick_random_node(rng, pic_names);
        for _ in 1..count {
            first.add_random(APTNode::pick_random_node(rng, pic_names), rng);
        }
        while first.add_leaf(&leaf_func(rng)) {}
        (first, coord)
    }

    pub fn get_children_mut(&mut self) -> Option<&mut Vec<APTNode>> {
        match self {
            APTNode::Add(children)
            | APTNode::Sub(children)
            | APTNode::Mul(children)
            | APTNode::Div(children)
            | APTNode::Mod(children)
            | APTNode::FBM(children)
            | APTNode::Ridge(children)
            | APTNode::Turbulence(children)
            | APTNode::Cell1(children)
            | APTNode::Cell2(children)
            | APTNode::Sqrt(children)
            | APTNode::Sin(children)
            | APTNode::Atan(children)
            | APTNode::Atan2(children)
            | APTNode::Tan(children)
            | APTNode::Log(children)
            | APTNode::Abs(children)
            | APTNode::Floor(children)
            | APTNode::Ceil(children)
            | APTNode::Clamp(children)
            | APTNode::Wrap(children)
            | APTNode::Square(children)
            | APTNode::Max(children)
            | APTNode::Min(children)
            | APTNode::Mandelbrot(children) => Some(children),
            APTNode::Picture(_, children) => Some(children),
            _ => None,
        }
    }

    pub fn get_children(&self) -> Option<&Vec<APTNode>> {
        match self {
            APTNode::Add(children)
            | APTNode::Sub(children)
            | APTNode::Mul(children)
            | APTNode::Div(children)
            | APTNode::Mod(children)
            | APTNode::FBM(children)
            | APTNode::Ridge(children)
            | APTNode::Turbulence(children)
            | APTNode::Cell1(children)
            | APTNode::Cell2(children)
            | APTNode::Sqrt(children)
            | APTNode::Sin(children)
            | APTNode::Atan(children)
            | APTNode::Atan2(children)
            | APTNode::Tan(children)
            | APTNode::Log(children)
            | APTNode::Abs(children)
            | APTNode::Floor(children)
            | APTNode::Ceil(children)
            | APTNode::Clamp(children)
            | APTNode::Wrap(children)
            | APTNode::Square(children)
            | APTNode::Max(children)
            | APTNode::Min(children)
            | APTNode::Mandelbrot(children) => Some(children),
            APTNode::Picture(_, children) => Some(children),
            _ => None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(
            self,
            APTNode::Width
                | APTNode::Height
                | APTNode::PI
                | APTNode::E
                | APTNode::X
                | APTNode::Y
                | APTNode::T
                | APTNode::Constant(_)
                | APTNode::Empty
        )
    }

    pub fn parse_apt_node(receiver: &Receiver<Token>) -> Result<APTNode, String> {
        APTNode::parse_apt_node_at_depth(receiver, 0)
    }

    fn parse_apt_node_at_depth(
        receiver: &Receiver<Token>,
        depth: usize,
    ) -> Result<APTNode, String> {
        if depth > MAX_PARSE_DEPTH {
            return Err(format!(
                "expression nested too deeply (max depth {})",
                MAX_PARSE_DEPTH
            ));
        }
        loop {
            match receiver.recv() {
                Ok(token) => {
                    match token {
                        Token::Operation(s, line_num) => {
                            let mut node = APTNode::str_to_node(s)
                                .map_err(|msg| msg + &format!(" on line {}", line_num))?;
                            match node.get_children_mut() {
                                Some(children) => {
                                    for child in children {
                                        *child =
                                            APTNode::parse_apt_node_at_depth(receiver, depth + 1)?;
                                    }
                                    return Ok(node);
                                }
                                None => return Ok(node),
                            }
                        }
                        Token::Constant(vstr, line_num) => {
                            let v = vstr.parse::<f32>().map_err(|_| {
                                format!("Unable to parse number {} on line {}", vstr, line_num)
                            })?;
                            return Ok(APTNode::Constant(v));
                        }
                        _ => (), //parens don't matter
                    }
                }
                Err(_) => {
                    return Err("Unexpected end of file".to_string());
                }
            }
        }
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    pub fn mock_pics() -> Arc<HashMap<String, ActualPicture>> {
        Arc::new(HashMap::new())
    }

    fn mock_params(count: usize, filled: bool) -> Vec<APTNode> {
        let mut params = Vec::with_capacity(count);
        if filled {
            let base = 1.1;
            let mut val = 1.0;
            for _i in 0..count {
                params.push(APTNode::Constant(val));
                val += base;
            }
        } else {
            for _i in 0..count {
                params.push(APTNode::Empty);
            }
        }

        params
    }
    pub fn mock_params_add(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
    pub fn mock_params_sub(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
    pub fn mock_params_mul(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
    pub fn mock_params_div(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
    pub fn mock_params_fbm(filled: bool) -> Vec<APTNode> {
        mock_params(6, filled)
    }
    pub fn mock_params_ridge(filled: bool) -> Vec<APTNode> {
        mock_params(6, filled)
    }
    pub fn mock_params_turbulence(filled: bool) -> Vec<APTNode> {
        mock_params(6, filled)
    }
    pub fn mock_params_cell1(filled: bool) -> Vec<APTNode> {
        mock_params(5, filled)
    }
    pub fn mock_params_cell2(filled: bool) -> Vec<APTNode> {
        mock_params(5, filled)
    }
    pub fn mock_params_sqrt(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_sin(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_atan(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_atan2(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
    pub fn mock_params_tan(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_log(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_abs(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_floor(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_ceil(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_clamp(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_wrap(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_square(filled: bool) -> Vec<APTNode> {
        mock_params(1, filled)
    }
    pub fn mock_params_max(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
    pub fn mock_params_min(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
    pub fn mock_params_mod(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
    pub fn mock_params_mandelbrot(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
    pub fn mock_params_picture(filled: bool) -> Vec<APTNode> {
        mock_params(2, filled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use simdeez::scalar::Scalar;

    #[test]
    fn test_aptnode_to_lisp() {
        assert_eq!(
            APTNode::Add(mock::mock_params_add(true)).to_lisp(),
            "( + 1 2.1 )"
        );
        assert_eq!(
            APTNode::Sub(mock::mock_params_sub(true)).to_lisp(),
            "( - 1 2.1 )"
        );
        assert_eq!(
            APTNode::Mul(mock::mock_params_mul(true)).to_lisp(),
            "( * 1 2.1 )"
        );
        assert_eq!(
            APTNode::Div(mock::mock_params_div(true)).to_lisp(),
            "( / 1 2.1 )"
        );
        assert_eq!(
            APTNode::Mod(mock::mock_params_mod(true)).to_lisp(),
            "( % 1 2.1 )"
        );
        assert_eq!(
            APTNode::FBM(mock::mock_params_fbm(true)).to_lisp(),
            "( FBM 1 2.1 3.1999998 4.2999997 5.3999996 6.4999995 )"
        );
        assert_eq!(
            APTNode::Ridge(mock::mock_params_ridge(true)).to_lisp(),
            "( RIDGE 1 2.1 3.1999998 4.2999997 5.3999996 6.4999995 )"
        );
        assert_eq!(
            APTNode::Cell1(mock::mock_params_cell1(true)).to_lisp(),
            "( CELL1 1 2.1 3.1999998 4.2999997 5.3999996 )"
        );
        assert_eq!(
            APTNode::Cell2(mock::mock_params_cell2(true)).to_lisp(),
            "( CELL2 1 2.1 3.1999998 4.2999997 5.3999996 )"
        );
        assert_eq!(
            APTNode::Turbulence(mock::mock_params_turbulence(true)).to_lisp(),
            "( TURBULENCE 1 2.1 3.1999998 4.2999997 5.3999996 6.4999995 )"
        );
        assert_eq!(
            APTNode::Sqrt(mock::mock_params_sqrt(true)).to_lisp(),
            "( SQRT 1 )"
        );
        assert_eq!(
            APTNode::Sin(mock::mock_params_sin(true)).to_lisp(),
            "( SIN 1 )"
        );
        assert_eq!(
            APTNode::Atan(mock::mock_params_atan(true)).to_lisp(),
            "( ATAN 1 )"
        );
        assert_eq!(
            APTNode::Atan2(mock::mock_params_atan2(true)).to_lisp(),
            "( ATAN2 1 2.1 )"
        );
        assert_eq!(
            APTNode::Tan(mock::mock_params_tan(true)).to_lisp(),
            "( TAN 1 )"
        );
        assert_eq!(
            APTNode::Log(mock::mock_params_log(true)).to_lisp(),
            "( LOG 1 )"
        );
        assert_eq!(
            APTNode::Abs(mock::mock_params_abs(true)).to_lisp(),
            "( ABS 1 )"
        );
        assert_eq!(
            APTNode::Abs(vec![APTNode::Constant(-10000.5)]).to_lisp(),
            "( ABS -10000.5 )"
        );
        assert_eq!(
            APTNode::Floor(mock::mock_params_floor(true)).to_lisp(),
            "( FLOOR 1 )"
        );
        assert_eq!(
            APTNode::Floor(vec![APTNode::Constant(10000.5)]).to_lisp(),
            "( FLOOR 10000.5 )"
        );
        assert_eq!(
            APTNode::Ceil(mock::mock_params_ceil(true)).to_lisp(),
            "( CEIL 1 )"
        );
        assert_eq!(
            APTNode::Ceil(vec![APTNode::Constant(10000.5)]).to_lisp(),
            "( CEIL 10000.5 )"
        );
        assert_eq!(
            APTNode::Clamp(mock::mock_params_clamp(true)).to_lisp(),
            "( CLAMP 1 )"
        );
        assert_eq!(
            APTNode::Wrap(mock::mock_params_wrap(true)).to_lisp(),
            "( WRAP 1 )"
        );
        assert_eq!(
            APTNode::Square(mock::mock_params_square(true)).to_lisp(),
            "( SQUARE 1 )"
        );
        assert_eq!(
            APTNode::Max(mock::mock_params_max(true)).to_lisp(),
            "( MAX 1 2.1 )"
        );
        assert_eq!(
            APTNode::Min(mock::mock_params_min(true)).to_lisp(),
            "( MIN 1 2.1 )"
        );
        assert_eq!(
            APTNode::Mandelbrot(mock::mock_params_mandelbrot(true)).to_lisp(),
            "( MANDELBROT 1 2.1 )"
        );
        assert_eq!(
            APTNode::Picture(
                "eye.jpg".to_string(),
                vec![APTNode::Constant(800.0), APTNode::Constant(600.0)]
            )
            .to_lisp(),
            "( PIC-eye.jpg 800 600 )".to_string()
        );
        assert_eq!(
            APTNode::Picture("eye.jpg".to_string(), mock::mock_params_picture(true)).to_lisp(),
            "( PIC-eye.jpg 1 2.1 )".to_string()
        );
        assert_eq!(APTNode::Constant(123.456).to_lisp(), "123.456");
        assert_eq!(APTNode::Constant(0.0).to_lisp(), "0");
        assert_eq!(APTNode::Constant(1.0).to_lisp(), "1");
        assert_eq!(APTNode::Constant(std::f32::consts::PI).to_lisp(), "PI");
        assert_eq!(APTNode::PI.to_lisp(), "PI");
        assert_eq!(APTNode::Constant(std::f32::consts::E).to_lisp(), "E");
        assert_eq!(APTNode::E.to_lisp(), "E");
        assert_eq!(APTNode::Width.to_lisp(), "WIDTH");
        assert_eq!(APTNode::Height.to_lisp(), "HEIGHT");
        assert_eq!(APTNode::X.to_lisp(), "X");
        assert_eq!(APTNode::Y.to_lisp(), "Y");
        assert_eq!(APTNode::T.to_lisp(), "T");
        assert_eq!(APTNode::Empty.to_lisp(), "EMPTY");
    }

    #[test]
    fn test_aptnode_str_to_node() {
        assert_eq!(
            APTNode::str_to_node("+"),
            Ok(APTNode::Add(mock::mock_params_add(false)))
        );
        assert_eq!(
            APTNode::str_to_node("-"),
            Ok(APTNode::Sub(vec![APTNode::Empty, APTNode::Empty]))
        );
        assert_eq!(
            APTNode::str_to_node("*"),
            Ok(APTNode::Mul(vec![APTNode::Empty, APTNode::Empty]))
        );
        assert_eq!(
            APTNode::str_to_node("/"),
            Ok(APTNode::Div(vec![APTNode::Empty, APTNode::Empty]))
        );
        assert_eq!(
            APTNode::str_to_node("%"),
            Ok(APTNode::Mod(mock::mock_params_mod(false)))
        );
        assert_eq!(
            APTNode::str_to_node("fbm"),
            Ok(APTNode::FBM(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty
            ]))
        );
        assert_eq!(
            APTNode::str_to_node("ridge"),
            Ok(APTNode::Ridge(vec![
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty,
                APTNode::Empty
            ]))
        );
        assert_eq!(
            APTNode::str_to_node("turbulence"),
            Ok(APTNode::Turbulence(mock::mock_params_turbulence(false)))
        );
        assert_eq!(
            APTNode::str_to_node("cell1"),
            Ok(APTNode::Cell1(mock::mock_params_cell1(false)))
        );
        assert_eq!(
            APTNode::str_to_node("cell2"),
            Ok(APTNode::Cell2(mock::mock_params_cell2(false)))
        );
        assert_eq!(
            APTNode::str_to_node("sqrt"),
            Ok(APTNode::Sqrt(mock::mock_params_sqrt(false)))
        );
        assert_eq!(
            APTNode::str_to_node("sin"),
            Ok(APTNode::Sin(mock::mock_params_sin(false)))
        );
        assert_eq!(
            APTNode::str_to_node("atan"),
            Ok(APTNode::Atan(mock::mock_params_atan(false)))
        );
        assert_eq!(
            APTNode::str_to_node("atan2"),
            Ok(APTNode::Atan2(mock::mock_params_atan2(false)))
        );
        assert_eq!(
            APTNode::str_to_node("tan"),
            Ok(APTNode::Tan(mock::mock_params_tan(false)))
        );
        assert_eq!(
            APTNode::str_to_node("log"),
            Ok(APTNode::Log(mock::mock_params_log(false)))
        );
        assert_eq!(
            APTNode::str_to_node("abs"),
            Ok(APTNode::Abs(mock::mock_params_abs(false)))
        );
        assert_eq!(
            APTNode::str_to_node("floor"),
            Ok(APTNode::Floor(vec![APTNode::Empty]))
        );
        assert_eq!(
            APTNode::str_to_node("ceil"),
            Ok(APTNode::Ceil(mock::mock_params_ceil(false)))
        );
        assert_eq!(
            APTNode::str_to_node("clamp"),
            Ok(APTNode::Clamp(mock::mock_params_clamp(false)))
        );
        assert_eq!(
            APTNode::str_to_node("wrap"),
            Ok(APTNode::Wrap(mock::mock_params_wrap(false)))
        );
        assert_eq!(
            APTNode::str_to_node("square"),
            Ok(APTNode::Square(mock::mock_params_square(false)))
        );
        assert_eq!(
            APTNode::str_to_node("max"),
            Ok(APTNode::Max(mock::mock_params_max(false)))
        );
        assert_eq!(
            APTNode::str_to_node("min"),
            Ok(APTNode::Min(mock::mock_params_min(false)))
        );
        assert_eq!(
            APTNode::str_to_node("mandelbrot"),
            Ok(APTNode::Mandelbrot(mock::mock_params_mandelbrot(false)))
        );
        assert_eq!(
            APTNode::str_to_node("PIC-eye.jpg"),
            Ok(APTNode::Picture(
                "eye.jpg".to_string(),
                mock::mock_params_picture(false)
            ))
        );
        assert_eq!(APTNode::str_to_node("Width"), Ok(APTNode::Width));
        assert_eq!(APTNode::str_to_node("Height"), Ok(APTNode::Height));
        assert_eq!(APTNode::str_to_node("Pi"), Ok(APTNode::PI));
        assert_eq!(APTNode::str_to_node("e"), Ok(APTNode::E));
        assert_eq!(APTNode::str_to_node("x"), Ok(APTNode::X));
        assert_eq!(APTNode::str_to_node("y"), Ok(APTNode::Y));
        assert_eq!(APTNode::str_to_node("t"), Ok(APTNode::T));
        assert_eq!(
            APTNode::str_to_node("pizza 60.0 \""),
            Err("Unknown operation 'pizza 60.0 \"' ".to_string())
        );
    }

    #[test]
    fn test_aptnode_add_leaf() {
        let mut root = APTNode::Add(vec![APTNode::Empty, APTNode::Empty]);
        assert!(root.add_leaf(&APTNode::Constant(1.0)));
        assert!(root.add_leaf(&APTNode::Constant(2.1)));
        assert_eq!(
            root,
            APTNode::Add(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)])
        );
        assert!(!root.add_leaf(&APTNode::Constant(3.1)));
        assert_eq!(
            root,
            APTNode::Add(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)])
        );
    }

    #[test]
    fn test_aptnode_constant_eval() {
        let pics = mock::mock_pics();
        assert_eq!(
            APTNode::Add(vec![APTNode::Constant(10.0), APTNode::Constant(1.02)])
                .constant_eval::<Scalar>(
                    &CoordinateSystem::Polar,
                    pics.clone(),
                    None,
                    None,
                    None,
                    None,
                    None
                ),
            11.02
        );
        assert_eq!(
            APTNode::Sub(vec![APTNode::Constant(10.0), APTNode::Constant(1.02)])
                .constant_eval::<Scalar>(
                    &CoordinateSystem::Polar,
                    pics.clone(),
                    None,
                    None,
                    None,
                    None,
                    None
                ),
            8.98
        );
        assert_eq!(
            APTNode::Mul(vec![APTNode::Constant(10.0), APTNode::Constant(1.02)])
                .constant_eval::<Scalar>(
                    &CoordinateSystem::Polar,
                    pics.clone(),
                    None,
                    None,
                    None,
                    None,
                    None
                ),
            10.2
        );
        assert_eq!(
            APTNode::Div(vec![APTNode::Constant(10.0), APTNode::Constant(1.02)])
                .constant_eval::<Scalar>(
                    &CoordinateSystem::Polar,
                    pics.clone(),
                    None,
                    None,
                    None,
                    None,
                    None
                ),
            9.803922
        );
        assert_eq!(
            APTNode::FBM(vec![
                APTNode::Constant(0.0),
                APTNode::Constant(1.1),
                APTNode::Constant(2.2),
                APTNode::Constant(3.3),
                APTNode::Constant(4.4),
                APTNode::Constant(5.5)
            ])
            .constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.06039
        );
        assert_eq!(
            APTNode::Ridge(mock::mock_params_ridge(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.47665274
        );
        assert_eq!(
            APTNode::Cell1(mock::mock_params_cell1(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.8156951
        );
        assert_eq!(
            APTNode::Cell2(mock::mock_params_cell2(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -0.25962472
        );
        assert_eq!(
            APTNode::Turbulence(mock::mock_params_turbulence(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            2.5233474
        );
        assert_eq!(
            APTNode::Sqrt(vec![APTNode::Constant(16.0)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            4.0
        );
        assert_eq!(
            APTNode::Sin(mock::mock_params_sin(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -8.742279e-8
        );
        assert_eq!(
            APTNode::Atan(mock::mock_params_atan(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.88387847
        );
        assert_eq!(
            APTNode::Atan2(mock::mock_params_atan2(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.34611407
        );
        assert_eq!(
            APTNode::Tan(mock::mock_params_tan(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -22877332.0
        );
        assert_eq!(
            APTNode::Log(mock::mock_params_log(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.5099892
        );
        assert_eq!(
            APTNode::Log(vec![APTNode::Constant(10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            3.8983026
        );
        assert_eq!(
            APTNode::Abs(mock::mock_params_abs(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Abs(vec![APTNode::Constant(10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            10000.5
        );
        assert_eq!(
            APTNode::Floor(mock::mock_params_floor(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Floor(vec![APTNode::Constant(-10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -10001.0
        );
        assert_eq!(
            APTNode::Floor(vec![APTNode::Constant(10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            10000.0
        );
        assert_eq!(
            APTNode::Ceil(mock::mock_params_ceil(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Ceil(vec![APTNode::Constant(10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            10001.0
        );
        assert_eq!(
            APTNode::Ceil(vec![APTNode::Constant(-10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -10000.0
        );
        assert_eq!(
            APTNode::Clamp(mock::mock_params_clamp(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Clamp(vec![APTNode::Constant(10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Clamp(vec![APTNode::Constant(1.0)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Clamp(vec![APTNode::Constant(0.8)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.8
        );
        assert_eq!(
            APTNode::Clamp(vec![APTNode::Constant(-0.8)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -0.8
        );
        assert_eq!(
            APTNode::Clamp(vec![APTNode::Constant(-1.0)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -1.0
        );
        assert_eq!(
            APTNode::Clamp(vec![APTNode::Constant(-10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -1.0
        );
        assert_eq!(
            APTNode::Wrap(mock::mock_params_wrap(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Wrap(vec![APTNode::Constant(10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.5
        );
        assert_eq!(
            APTNode::Wrap(vec![APTNode::Constant(1.0)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Wrap(vec![APTNode::Constant(0.8)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.8
        );
        assert_eq!(
            APTNode::Wrap(vec![APTNode::Constant(-0.8)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -0.8
        );
        assert_eq!(
            APTNode::Wrap(vec![APTNode::Constant(-1.0)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -1.0
        );
        assert_eq!(
            APTNode::Wrap(vec![APTNode::Constant(-10000.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            -0.5
        );
        assert_eq!(
            APTNode::Square(mock::mock_params_square(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Square(vec![APTNode::Constant(4.5)]).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            20.25
        );
        assert_eq!(
            APTNode::Max(mock::mock_params_max(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            2.1
        );
        assert_eq!(
            APTNode::Max(vec![APTNode::Constant(1.0), APTNode::Constant(-2.1)])
                .constant_eval::<Scalar>(
                    &CoordinateSystem::Polar,
                    pics.clone(),
                    None,
                    None,
                    None,
                    None,
                    None
                ),
            1.0
        );
        assert_eq!(
            APTNode::Min(mock::mock_params_min(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
        assert_eq!(
            APTNode::Min(vec![APTNode::Constant(1.0), APTNode::Constant(-2.1)])
                .constant_eval::<Scalar>(
                    &CoordinateSystem::Polar,
                    pics.clone(),
                    None,
                    None,
                    None,
                    None,
                    None
                ),
            -2.1
        );
        assert_eq!(
            APTNode::Mod(mock::mock_params_mod(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.099999905
        );
        assert_eq!(
            APTNode::Mod(vec![APTNode::Constant(2.1), APTNode::Constant(1.0)])
                .constant_eval::<Scalar>(
                    &CoordinateSystem::Polar,
                    pics.clone(),
                    None,
                    None,
                    None,
                    None,
                    None
                ),
            1.0
        );
        assert_eq!(
            APTNode::Mandelbrot(mock::mock_params_mandelbrot(true)).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            2.1
        );
        /*
        @todo
        assert_eq!(
            APTNode::Picture("eye.jpg".to_string(), mock::mock_params_picture(true))
                .constant_eval::<Scalar>(CoordinateSystem::Polar, pics.clone(), None, None, None, None, None),
            0.0
        );
        */
        assert_eq!(
            APTNode::PI.constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            std::f32::consts::PI
        );
        assert_eq!(
            APTNode::E.constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            std::f32::consts::E
        );
        assert_eq!(
            APTNode::Constant(123.456).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            123.456
        );
        assert_eq!(
            APTNode::Constant(0.0).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            0.0
        );
        assert_eq!(
            APTNode::Constant(1.0).constant_eval::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            1.0
        );
    }

    #[should_panic(expected = "invalid node passed to constant_esval")]
    #[test]
    fn test_aptnode_constant_eval_width() {
        let pics = mock::mock_pics();
        APTNode::Width.constant_eval::<Scalar>(
            &CoordinateSystem::Polar,
            pics,
            None,
            None,
            None,
            None,
            None,
        );
    }

    #[should_panic(expected = "invalid node passed to constant_esval")]
    #[test]
    fn test_aptnode_constant_eval_height() {
        let pics = mock::mock_pics();
        APTNode::Height.constant_eval::<Scalar>(
            &CoordinateSystem::Polar,
            pics,
            None,
            None,
            None,
            None,
            None,
        );
    }

    #[should_panic(expected = "invalid node passed to constant_esval")]
    #[test]
    fn test_aptnode_constant_eval_x() {
        let pics = mock::mock_pics();
        APTNode::X.constant_eval::<Scalar>(
            &CoordinateSystem::Polar,
            pics,
            None,
            None,
            None,
            None,
            None,
        );
    }

    #[should_panic(expected = "invalid node passed to constant_esval")]
    #[test]
    fn test_aptnode_constant_eval_y() {
        let pics = mock::mock_pics();
        APTNode::Y.constant_eval::<Scalar>(
            &CoordinateSystem::Polar,
            pics,
            None,
            None,
            None,
            None,
            None,
        );
    }

    #[should_panic(expected = "invalid node passed to constant_esval")]
    #[test]
    fn test_aptnode_constant_eval_t() {
        let pics = mock::mock_pics();
        APTNode::T.constant_eval::<Scalar>(
            &CoordinateSystem::Polar,
            pics,
            None,
            None,
            None,
            None,
            None,
        );
    }

    #[should_panic(expected = "invalid node passed to constant_esval")]
    #[test]
    fn test_aptnode_constant_eval_eval() {
        let pics = mock::mock_pics();
        APTNode::Empty.constant_eval::<Scalar>(
            &CoordinateSystem::Polar,
            pics,
            None,
            None,
            None,
            None,
            None,
        );
    }

    #[test]
    fn test_aptnode_constant_set_children() {
        //@todo: check that the vector lengths are correct for each of the enums
        assert_eq!(
            APTNode::str_to_node("abs")
                .unwrap()
                .set_children(mock::mock_params_abs(true)),
            APTNode::Abs(mock::mock_params_abs(true))
        );

        assert_eq!(
            APTNode::str_to_node("+")
                .unwrap()
                .set_children(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)]),
            APTNode::Add(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)])
        );

        assert_eq!(
            APTNode::str_to_node("atan")
                .unwrap()
                .set_children(mock::mock_params_atan(true)),
            APTNode::Atan(mock::mock_params_atan(true))
        );

        if false {
            assert_eq!(
                APTNode::str_to_node("atan2")
                    .unwrap()
                    .set_children(mock::mock_params_atan2(true)),
                APTNode::Atan2(mock::mock_params_atan2(true))
            );
        }

        assert_eq!(
            APTNode::str_to_node("ceil")
                .unwrap()
                .set_children(mock::mock_params_ceil(true)),
            APTNode::Ceil(mock::mock_params_ceil(true))
        );

        assert_eq!(
            APTNode::str_to_node("cell1")
                .unwrap()
                .set_children(mock::mock_params_cell1(true)),
            APTNode::Cell1(mock::mock_params_cell1(true))
        );

        assert_eq!(
            APTNode::str_to_node("cell2")
                .unwrap()
                .set_children(mock::mock_params_cell2(true)),
            APTNode::Cell2(mock::mock_params_cell2(true))
        );

        assert_eq!(
            APTNode::str_to_node("clamp")
                .unwrap()
                .set_children(mock::mock_params_clamp(true)),
            APTNode::Clamp(mock::mock_params_clamp(true))
        );

        if false {
            assert_eq!(
                APTNode::Constant(0.0).set_children(vec![APTNode::Constant(123.456)]),
                APTNode::Constant(123.456)
            );
        }

        assert_eq!(
            APTNode::str_to_node("/")
                .unwrap()
                .set_children(vec![APTNode::Constant(0.0), APTNode::Constant(9_992.111)]),
            APTNode::Div(vec![APTNode::Constant(0.0), APTNode::Constant(9_992.111)])
        );

        assert_eq!(
            APTNode::str_to_node("fbm").unwrap().set_children(vec![
                APTNode::Constant(0.0),
                APTNode::Constant(1.1),
                APTNode::Constant(2.2),
                APTNode::Constant(3.3),
                APTNode::Constant(4.4),
                APTNode::Constant(5.5)
            ]),
            APTNode::FBM(vec![
                APTNode::Constant(0.0),
                APTNode::Constant(1.1),
                APTNode::Constant(2.2),
                APTNode::Constant(3.3),
                APTNode::Constant(4.4),
                APTNode::Constant(5.5)
            ])
        );

        assert_eq!(
            APTNode::str_to_node("floor")
                .unwrap()
                .set_children(mock::mock_params_floor(true)),
            APTNode::Floor(mock::mock_params_floor(true))
        );

        assert_eq!(
            APTNode::str_to_node("log")
                .unwrap()
                .set_children(mock::mock_params_log(true)),
            APTNode::Log(mock::mock_params_log(true))
        );

        assert_eq!(
            APTNode::Mandelbrot(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)])
                .set_children(mock::mock_params_mandelbrot(true)),
            APTNode::Mandelbrot(mock::mock_params_mandelbrot(true))
        );

        assert_eq!(
            APTNode::str_to_node("max")
                .unwrap()
                .set_children(mock::mock_params_max(true)),
            APTNode::Max(mock::mock_params_max(true))
        );

        assert_eq!(
            APTNode::str_to_node("min")
                .unwrap()
                .set_children(mock::mock_params_min(true)),
            APTNode::Min(mock::mock_params_min(true))
        );

        assert_eq!(
            APTNode::str_to_node("*")
                .unwrap()
                .set_children(vec![APTNode::Constant(1.0), APTNode::Constant(2.0)]),
            APTNode::Mul(vec![APTNode::Constant(1.0), APTNode::Constant(2.0)])
        );

        assert_eq!(
            APTNode::str_to_node("%")
                .unwrap()
                .set_children(mock::mock_params_mod(true)),
            APTNode::Mod(mock::mock_params_mod(true))
        );

        assert_eq!(
            APTNode::str_to_node("pic-eye.jpg")
                .unwrap()
                .set_children(vec![
                    APTNode::Picture("cow.jpg".to_string(), mock::mock_params_picture(true)),
                    APTNode::Constant(333.0),
                    APTNode::Constant(444.0)
                ]),
            APTNode::Picture(
                "eye.jpg".to_string(),
                vec![APTNode::Constant(333.0), APTNode::Constant(444.0)]
            )
        );

        assert_eq!(
            APTNode::str_to_node("ridge")
                .unwrap()
                .set_children(mock::mock_params_ridge(true)),
            APTNode::Ridge(mock::mock_params_ridge(true))
        );

        assert_eq!(
            APTNode::str_to_node("sin")
                .unwrap()
                .set_children(mock::mock_params_sin(true)),
            APTNode::Sin(mock::mock_params_sin(true))
        );

        assert_eq!(
            APTNode::str_to_node("sqrt")
                .unwrap()
                .set_children(mock::mock_params_sqrt(true)),
            APTNode::Sqrt(mock::mock_params_sqrt(true)),
        );

        assert_eq!(
            APTNode::str_to_node("square")
                .unwrap()
                .set_children(mock::mock_params_square(true)),
            APTNode::Square(mock::mock_params_square(true))
        );

        assert_eq!(
            APTNode::str_to_node("-")
                .unwrap()
                .set_children(vec![APTNode::Constant(1.1), APTNode::Constant(2.0)]),
            APTNode::Sub(vec![APTNode::Constant(1.1), APTNode::Constant(2.0)])
        );

        assert_eq!(
            APTNode::str_to_node("tan")
                .unwrap()
                .set_children(mock::mock_params_tan(true)),
            APTNode::Tan(mock::mock_params_tan(true))
        );

        assert_eq!(
            APTNode::str_to_node("turbulence")
                .unwrap()
                .set_children(mock::mock_params_turbulence(true)),
            APTNode::Turbulence(mock::mock_params_turbulence(true))
        );

        assert_eq!(
            APTNode::str_to_node("wrap")
                .unwrap()
                .set_children(mock::mock_params_wrap(true)),
            APTNode::Wrap(mock::mock_params_wrap(true))
        );

        assert_eq!(
            APTNode::Width.set_children(vec![APTNode::Empty]),
            APTNode::Width
        );
        assert_eq!(
            APTNode::Height.set_children(vec![APTNode::Empty]),
            APTNode::Height
        );
        assert_eq!(APTNode::PI.set_children(vec![APTNode::Empty]), APTNode::PI);
        assert_eq!(APTNode::E.set_children(vec![APTNode::Empty]), APTNode::E);

        assert_eq!(APTNode::X.set_children(vec![APTNode::Empty]), APTNode::X);

        assert_eq!(APTNode::Y.set_children(vec![APTNode::Empty]), APTNode::Y);

        assert_eq!(APTNode::T.set_children(vec![APTNode::Empty]), APTNode::T);
    }

    #[should_panic(expected = "tried to eval an empty node")]
    #[test]
    fn test_aptnode_constant_set_children_empty() {
        APTNode::Empty.set_children(vec![APTNode::Constant(10.0)]);
    }

    #[test]
    fn test_aptnode_constant_fold() {
        let pics = mock::mock_pics();
        assert_eq!(
            APTNode::Width.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Width
        );
        assert_eq!(
            APTNode::Height.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Height
        );
        assert_eq!(
            APTNode::PI.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Constant(std::f32::consts::PI)
        );
        assert_eq!(
            APTNode::E.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Constant(std::f32::consts::E)
        );
        assert_eq!(
            APTNode::X.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::X
        );
        assert_eq!(
            APTNode::Y.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Y
        );
        assert_eq!(
            APTNode::T.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::T
        );
        assert_eq!(
            APTNode::Add(vec![APTNode::Constant(1.0), APTNode::Constant(2.0)])
                .constant_fold::<Scalar>(
                    &CoordinateSystem::Polar,
                    pics.clone(),
                    None,
                    None,
                    None,
                    None,
                    None
                ),
            APTNode::Constant(3.0)
        );
        assert_eq!(
            APTNode::Add(vec![
                APTNode::Constant(1.0),
                APTNode::Mul(vec![APTNode::Constant(6.0), APTNode::Constant(0.5)])
            ])
            .constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Constant(4.0)
        );
        assert_eq!(
            APTNode::Add(vec![APTNode::Width, APTNode::Height]).constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                Some(800),
                Some(600),
                None,
            ),
            APTNode::Constant(1400.0)
        );
        assert_eq!(
            APTNode::Min(vec![
                APTNode::Mul(vec![APTNode::X, APTNode::Y]),
                APTNode::Add(vec![APTNode::Width, APTNode::Height])
            ])
            .constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                Some(12),
                Some(20),
                Some(800),
                Some(600),
                None,
            ),
            APTNode::Constant(240.0)
        );
    }

    #[test]
    fn test_aptnode_get_children_mut() {
        assert_eq!(
            APTNode::Add(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)])
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Sub(vec![APTNode::Constant(1.1), APTNode::Constant(2.0)])
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Mul(vec![APTNode::Constant(1.0), APTNode::Constant(2.0)])
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Div(vec![APTNode::Constant(0.0), APTNode::Constant(9_992.111)])
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::FBM(vec![
                APTNode::Constant(0.0),
                APTNode::Constant(1.1),
                APTNode::Constant(2.2)
            ])
            .get_children_mut()
            .unwrap()
            .len(),
            3
        );
        assert_eq!(
            APTNode::Ridge(mock::mock_params_ridge(true))
                .get_children_mut()
                .unwrap()
                .len(),
            6
        );
        assert_eq!(
            APTNode::Cell1(mock::mock_params_cell1(true))
                .get_children_mut()
                .unwrap()
                .len(),
            5
        );
        assert_eq!(
            APTNode::Cell2(mock::mock_params_cell2(true))
                .get_children_mut()
                .unwrap()
                .len(),
            5
        );
        assert_eq!(
            APTNode::Turbulence(mock::mock_params_turbulence(true))
                .get_children_mut()
                .unwrap()
                .len(),
            6
        );
        assert_eq!(
            APTNode::Sqrt(mock::mock_params_sqrt(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Sin(mock::mock_params_sin(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Atan(mock::mock_params_atan(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Atan2(mock::mock_params_atan2(true))
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Tan(mock::mock_params_tan(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Log(mock::mock_params_log(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Abs(mock::mock_params_abs(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Floor(mock::mock_params_floor(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Ceil(mock::mock_params_ceil(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Clamp(mock::mock_params_clamp(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Wrap(mock::mock_params_wrap(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Square(mock::mock_params_square(true))
                .get_children_mut()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Max(mock::mock_params_max(true))
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Min(mock::mock_params_min(true))
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Mod(mock::mock_params_mod(true))
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Mandelbrot(mock::mock_params_mandelbrot(true))
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Picture("eye.jpg".to_string(), mock::mock_params_picture(true))
                .get_children_mut()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(APTNode::Constant(1.2).get_children_mut(), None);
        assert_eq!(APTNode::Width.get_children_mut(), None);
        assert_eq!(APTNode::Height.get_children_mut(), None);
        assert_eq!(APTNode::PI.get_children_mut(), None);
        assert_eq!(APTNode::E.get_children_mut(), None);
        assert_eq!(APTNode::X.get_children_mut(), None);
        assert_eq!(APTNode::Y.get_children_mut(), None);
        assert_eq!(APTNode::T.get_children_mut(), None);
        assert_eq!(APTNode::Empty.get_children(), None);
    }

    #[test]
    fn test_aptnode_get_children() {
        assert_eq!(
            APTNode::Add(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)])
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Sub(vec![APTNode::Constant(1.1), APTNode::Constant(2.0)])
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Mul(vec![APTNode::Constant(1.0), APTNode::Constant(2.0)])
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Div(vec![APTNode::Constant(0.0), APTNode::Constant(9_992.111)])
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::FBM(vec![
                APTNode::Constant(0.0),
                APTNode::Constant(1.1),
                APTNode::Constant(2.2)
            ])
            .get_children()
            .unwrap()
            .len(),
            3
        );
        assert_eq!(
            APTNode::Ridge(mock::mock_params_ridge(true))
                .get_children()
                .unwrap()
                .len(),
            6
        );
        assert_eq!(
            APTNode::Cell1(mock::mock_params_cell1(true))
                .get_children()
                .unwrap()
                .len(),
            5
        );
        assert_eq!(
            APTNode::Cell2(mock::mock_params_cell2(true))
                .get_children()
                .unwrap()
                .len(),
            5
        );
        assert_eq!(
            APTNode::Turbulence(mock::mock_params_turbulence(true))
                .get_children()
                .unwrap()
                .len(),
            6
        );
        assert_eq!(
            APTNode::Sqrt(mock::mock_params_sqrt(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Sin(mock::mock_params_sin(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Atan(mock::mock_params_atan(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Atan2(mock::mock_params_atan2(true))
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Tan(mock::mock_params_tan(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Log(mock::mock_params_log(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Abs(mock::mock_params_abs(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Floor(mock::mock_params_floor(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Ceil(mock::mock_params_ceil(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Clamp(mock::mock_params_clamp(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Wrap(mock::mock_params_wrap(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Square(mock::mock_params_square(true))
                .get_children()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            APTNode::Max(mock::mock_params_max(true))
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Min(mock::mock_params_min(true))
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Mod(mock::mock_params_mod(true))
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Mandelbrot(mock::mock_params_mandelbrot(true))
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            APTNode::Picture("eye.jpg".to_string(), mock::mock_params_picture(true))
                .get_children()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(APTNode::Constant(1.2).get_children(), None);
        assert_eq!(APTNode::Width.get_children(), None);
        assert_eq!(APTNode::Height.get_children(), None);
        assert_eq!(APTNode::PI.get_children(), None);
        assert_eq!(APTNode::E.get_children(), None);
        assert_eq!(APTNode::X.get_children(), None);
        assert_eq!(APTNode::Y.get_children(), None);
        assert_eq!(APTNode::T.get_children(), None);
        assert_eq!(APTNode::Empty.get_children(), None);
    }

    #[test]
    fn test_aptnode_aptnode_is_leaf() {
        assert!(
            !APTNode::Add(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)]).is_leaf()
        );
        assert!(
            !APTNode::Sub(vec![APTNode::Constant(1.1), APTNode::Constant(2.0)]).is_leaf()
        );
        assert!(
            !APTNode::Mul(vec![APTNode::Constant(1.0), APTNode::Constant(2.0)]).is_leaf()
        );
        assert!(
            !APTNode::Div(vec![APTNode::Constant(0.0), APTNode::Constant(9_992.111)]).is_leaf()
        );
        assert!(
            !APTNode::FBM(vec![
                APTNode::Constant(0.0),
                APTNode::Constant(1.1),
                APTNode::Constant(2.2)
            ])
            .is_leaf()
        );
        assert!(
            !APTNode::Ridge(mock::mock_params_ridge(true)).is_leaf()
        );
        assert!(
            !APTNode::Cell1(mock::mock_params_cell1(true)).is_leaf()
        );
        assert!(
            !APTNode::Cell2(mock::mock_params_cell2(true)).is_leaf()
        );
        assert!(
            !APTNode::Turbulence(mock::mock_params_turbulence(true)).is_leaf()
        );
        assert!(!APTNode::Sqrt(mock::mock_params_sqrt(true)).is_leaf());
        assert!(!APTNode::Sin(mock::mock_params_sin(true)).is_leaf());
        assert!(!APTNode::Atan(mock::mock_params_atan(true)).is_leaf());
        assert!(
            !APTNode::Atan2(mock::mock_params_atan2(true)).is_leaf()
        );
        assert!(!APTNode::Tan(mock::mock_params_tan(true)).is_leaf());
        assert!(!APTNode::Log(mock::mock_params_log(true)).is_leaf());
        assert!(!APTNode::Abs(mock::mock_params_abs(true)).is_leaf());
        assert!(
            !APTNode::Floor(mock::mock_params_floor(true)).is_leaf()
        );
        assert!(!APTNode::Ceil(mock::mock_params_ceil(true)).is_leaf());
        assert!(
            !APTNode::Clamp(mock::mock_params_clamp(true)).is_leaf()
        );
        assert!(!APTNode::Wrap(mock::mock_params_wrap(true)).is_leaf());
        assert!(
            !APTNode::Square(mock::mock_params_square(true)).is_leaf()
        );
        assert!(!APTNode::Max(mock::mock_params_max(true)).is_leaf());
        assert!(!APTNode::Min(mock::mock_params_min(true)).is_leaf());
        assert!(!APTNode::Mod(mock::mock_params_mod(true)).is_leaf());
        assert!(
            !APTNode::Mandelbrot(mock::mock_params_mandelbrot(true)).is_leaf()
        );
        assert!(
            !APTNode::Picture("eye.jpg".to_string(), mock::mock_params_picture(true)).is_leaf()
        );
        assert!(APTNode::Constant(1.2).is_leaf());
        assert!(APTNode::X.is_leaf());
        assert!(APTNode::Width.is_leaf());
        assert!(APTNode::Height.is_leaf());
        assert!(APTNode::PI.is_leaf());
        assert!(APTNode::E.is_leaf());
        assert!(APTNode::Y.is_leaf());
        assert!(APTNode::T.is_leaf());
        assert!(APTNode::Empty.is_leaf());
    }

    #[test]
    fn test_aptnode_pick_random_node() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let name = "eye.jpg".to_string();

        let pic_names = vec![&name];
        for _i in 0..100 {
            match APTNode::pick_random_node(&mut rng, &pic_names) {
                APTNode::Constant(_) | APTNode::X | APTNode::Y | APTNode::T | APTNode::Empty => {
                    panic!("This APTNode was not expected");
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_aptnode_pick_random_leaf() {
        let mut rng = StdRng::from_rng(&mut rand::rng());

        for _i in 0..100 {
            match APTNode::pick_random_leaf(&mut rng) {
                APTNode::Constant(value) => {
                    assert!((-1.0..=1.0).contains(&value));
                }
                APTNode::X | APTNode::Y | APTNode::Empty => {}
                _ => {
                    panic!("This APTNode was not expected");
                }
            }
        }
    }

    #[test]
    fn test_aptnode_pick_random_leaf_video() {
        let mut rng = StdRng::from_rng(&mut rand::rng());

        for _i in 0..100 {
            match APTNode::pick_random_leaf_video(&mut rng) {
                APTNode::Constant(value) => {
                    assert!((-1.0..=1.0).contains(&value));
                }
                APTNode::X | APTNode::Y | APTNode::T | APTNode::Empty => {}
                _ => {
                    panic!("This APTNode was not expected");
                }
            }
        }
    }

    #[test]
    #[should_panic(expected = "tried to add_random to a leaf")]
    fn test_aptnode_add_random_fail() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let mut this_node = APTNode::X;
        let that_node = APTNode::Y;
        this_node.add_random(that_node, &mut rng);
    }

    #[test]
    #[ignore] // findout what is wrong here
    fn test_aptnode_add_random() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let mut this_node = APTNode::Add(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)]);
        let that_node = APTNode::Add(vec![APTNode::Constant(1.0), APTNode::Constant(2.1)]);
        this_node.add_random(that_node.clone(), &mut rng);
        let kids = this_node.get_children().unwrap();
        assert!(kids.first().unwrap() == &that_node || kids.get(1).unwrap() == &that_node);
    }

    #[test]
    fn test_apt_node_simplify_vars() {
        let pics = mock::mock_pics();
        let apt = APTNode::X;
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::X
        );
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                Some(80),
                None,
                None,
                None,
                None
            ),
            APTNode::Constant(80.0)
        );

        let apt = APTNode::Y;
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Y
        );
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                Some(60),
                None,
                None,
                None
            ),
            APTNode::Constant(60.0)
        );

        let apt = APTNode::T;
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::T
        );
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                Some(2.0)
            ),
            APTNode::Constant(2.0)
        );

        let apt = APTNode::Width;
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Width
        );
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                Some(800),
                None,
                None
            ),
            APTNode::Constant(800.0)
        );

        let apt = APTNode::Height;
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Height
        );
        assert_eq!(
            apt.constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                Some(600),
                None
            ),
            APTNode::Constant(600.0)
        );
    }

    #[test]
    fn test_apt_node_simplify_noise() {
        let pics = mock::mock_pics();
        let apt = APTNode::Turbulence(vec![
            APTNode::X,
            APTNode::Y,
            APTNode::Constant(2.2),
            APTNode::T,
            APTNode::E,
            APTNode::Height,
        ]);

        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                Some(12),
                Some(55),
                Some(800),
                Some(600),
                Some(1.2)
            ),
            APTNode::Constant(415.83722)
        );

        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                Some(12),
                Some(55),
                None,
                Some(600),
                Some(1.2)
            ),
            APTNode::Constant(415.83722)
        );
        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                Some(55),
                None,
                None,
                Some(1.2)
            ),
            APTNode::Turbulence(vec![
                APTNode::X,
                APTNode::Constant(55.0),
                APTNode::Constant(2.2),
                APTNode::Constant(1.2),
                APTNode::Constant(std::f32::consts::E),
                APTNode::Height,
            ])
        );
        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                Some(55),
                None,
                None,
                None
            ),
            APTNode::Turbulence(vec![
                APTNode::X,
                APTNode::Constant(55.0),
                APTNode::Constant(2.2),
                APTNode::T,
                APTNode::Constant(std::f32::consts::E),
                APTNode::Height,
            ])
        );
        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Turbulence(vec![
                APTNode::X,
                APTNode::Y,
                APTNode::Constant(2.2),
                APTNode::T,
                APTNode::Constant(std::f32::consts::E),
                APTNode::Height,
            ])
        );

        let apt = APTNode::Mul(vec![APTNode::Constant(2.2), APTNode::T]);
        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                Some(12),
                Some(55),
                Some(800),
                Some(600),
                Some(1.2)
            ),
            APTNode::Constant(2.2 * 1. * 1.2)
        );

        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                Some(1.2)
            ),
            APTNode::Constant(2.2 * 1. * 1.2)
        );

        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Mul(vec![APTNode::Constant(2.2), APTNode::T])
        );

        let apt = APTNode::Min(vec![APTNode::E, APTNode::Height]);
        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                Some(12),
                Some(55),
                Some(800),
                Some(600),
                Some(1.2)
            ),
            APTNode::Constant(std::f32::consts::E)
        );

        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                Some(600),
                None
            ),
            APTNode::Constant(std::f32::consts::E)
        );

        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Min(vec![
                APTNode::Constant(std::f32::consts::E),
                APTNode::Height
            ])
        );

        let apt = APTNode::Add(vec![APTNode::PI, APTNode::Width]);
        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                Some(12),
                Some(55),
                Some(800),
                Some(600),
                Some(1.2)
            ),
            APTNode::Constant(std::f32::consts::PI + 800.0)
        );
        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                Some(800),
                None,
                None
            ),
            APTNode::Constant(std::f32::consts::PI + 800.0)
        );
        assert_eq!(
            apt.clone().constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                None,
                None,
                None,
                None,
                None
            ),
            APTNode::Add(vec![
                APTNode::Constant(std::f32::consts::PI),
                APTNode::Width
            ])
        );
    }

    #[test]
    fn test_aptnode_constant_fold_coordinate_system() {
        let pics = mock::mock_pics();
        assert_eq!(
            APTNode::Mul(vec![APTNode::X, APTNode::Y]).constant_fold::<Scalar>(
                &CoordinateSystem::Polar,
                pics.clone(),
                Some(200),
                Some(150),
                None,
                None,
                None
            ),
            APTNode::Constant(200.0 * 150.)
        );
        assert_eq!(
            APTNode::Mul(vec![APTNode::X, APTNode::Y]).constant_fold::<Scalar>(
                &CoordinateSystem::Cartesian,
                pics.clone(),
                Some(200),
                Some(150),
                None,
                None,
                None
            ),
            APTNode::Constant(200.0 * 150.)
        );
    }
}
