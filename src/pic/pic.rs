use std::collections::HashMap;
use std::sync::Arc;

use crate::constants::{PIC_RANDOM_TREE_MAX, PIC_RANDOM_TREE_MIN};
use crate::parser::aptnode::APTNode;
use crate::pic::actual_picture::ActualPicture;
use crate::pic::coordinatesystem::CoordinateSystem;
use crate::pic::data::gradient::GradientData;
use crate::pic::data::grayscale::GrayscaleData;
use crate::pic::data::hsv::HSVData;
use crate::pic::data::mono::MonoData;
use crate::pic::data::rgb::RGBData;
use crate::pic::data::PicData;

use rand::prelude::*;
use rand::rngs::StdRng;
use simdeez::prelude::*;

simd_runtime_generate!(
    pub fn pic_get_rgba8(
        pic: &Pic,
        threaded: bool,
        pictures: Arc<HashMap<String, ActualPicture>>,
        width: u32,
        height: u32,
        t: f32,
    ) -> Vec<u8> {
        pic.get_rgba8::<S>(threaded, pictures, width, height, t)
    }
);

simd_runtime_generate!(
    pub fn pic_get_video(
        pic: &Pic,
        pictures: Arc<HashMap<String, ActualPicture>>,
        width: u32,
        height: u32,
        fps: u16,
        duration_ms: f32,
    ) -> Vec<Vec<u8>> {
        pic.get_video::<S>(pictures, width, height, fps, duration_ms)
    }
);

simd_runtime_generate!(
    pub fn pic_simplify(
        pic: &mut Pic,
        pictures: Arc<HashMap<String, ActualPicture>>,
        width: u32,
        height: u32,
        t: f32,
    ) {
        pic.simplify::<S>(pictures, width, height, t)
    }
);

#[derive(Clone, Debug, PartialEq)]
pub enum Pic {
    Mono(MonoData),
    Grayscale(GrayscaleData),
    RGB(RGBData),
    HSV(HSVData),
    Gradient(GradientData),
}

impl Pic {
    pub fn new(rng: &mut StdRng, pic_names: &Vec<&String>) -> Self {
        let pic_type = rng.random_range(0..5);

        
        match pic_type {
            0 => MonoData::new(
                PIC_RANDOM_TREE_MIN,
                PIC_RANDOM_TREE_MAX,
                false,
                rng,
                pic_names,
            ),
            1 => GradientData::new(
                PIC_RANDOM_TREE_MIN,
                PIC_RANDOM_TREE_MAX,
                false,
                rng,
                pic_names,
            ),
            2 => RGBData::new(
                PIC_RANDOM_TREE_MIN,
                PIC_RANDOM_TREE_MAX,
                false,
                rng,
                pic_names,
            ),
            3 => HSVData::new(
                PIC_RANDOM_TREE_MIN,
                PIC_RANDOM_TREE_MAX,
                false,
                rng,
                pic_names,
            ),
            4 => GrayscaleData::new(
                PIC_RANDOM_TREE_MIN,
                PIC_RANDOM_TREE_MAX,
                false,
                rng,
                pic_names,
            ),
            _ => panic!("invalid"),
        }
    }

    pub fn simplify<S: Simd>(
        &mut self,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    ) {
        match self {
            Pic::Grayscale(data) => data.simplify::<S>(pics, w, h, t),
            Pic::Mono(data) => data.simplify::<S>(pics, w, h, t),
            Pic::Gradient(data) => data.simplify::<S>(pics, w, h, t),
            Pic::RGB(data) => data.simplify::<S>(pics, w, h, t),
            Pic::HSV(data) => data.simplify::<S>(pics, w, h, t),
        }
    }

    pub fn to_tree(&self) -> Vec<&APTNode> {
        match self {
            Pic::Grayscale(data) => vec![&data.c],
            Pic::Mono(data) => vec![&data.c],
            Pic::Gradient(data) => vec![&data.index],
            Pic::RGB(data) => vec![&data.r, &data.g, &data.b],
            Pic::HSV(data) => vec![&data.h, &data.s, &data.v],
        }
    }

    pub fn to_lisp(&self) -> String {
        match self {
            Pic::Mono(data) => format!(
                "( MONO {}\n\t( {} )\n)",
                data.coord.to_string().to_uppercase(),
                data.c.to_lisp()
            ),
            Pic::Grayscale(data) => {
                format!(
                    "( GRAYSCALE {}\n\t( {} )\n)",
                    data.coord.to_string().to_uppercase(),
                    data.c.to_lisp()
                )
            }
            Pic::Gradient(data) => {
                let mut colors = String::new();
                for (color, stop) in &data.colors {
                    if *stop {
                        colors +=
                            &format!("\n\t\t( STOPCOLOR {} {} {} )", color.r, color.g, color.b);
                    } else {
                        colors += &format!("\n\t\t( COLOR {} {} {} )", color.r, color.g, color.b);
                    }
                }
                format!(
                    "( GRADIENT {}\n\t( COLORS{}\n\t)\n\t{}\n)",
                    data.coord.to_string().to_uppercase(),
                    colors,
                    data.index.to_lisp()
                )
            }
            Pic::RGB(data) => format!(
                "( RGB {}\n\t( {} )\n\t( {} )\n\t( {} )\n)",
                data.coord.to_string().to_uppercase(),
                data.r.to_lisp(),
                data.g.to_lisp(),
                data.b.to_lisp()
            ),
            Pic::HSV(data) => format!(
                "( HSV {}\n\t( {} )\n\t( {} )\n\t( {} )\n)",
                data.coord.to_string().to_uppercase(),
                data.h.to_lisp(),
                data.s.to_lisp(),
                data.v.to_lisp()
            ),
        }
    }

    pub fn get_video<S: Simd>(
        &self,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        fps: u16,
        d_ms: f32,
    ) -> Vec<Vec<u8>> {
        // todo investigate if we can return an iterator instead of a vec
        let frames = (fps as f32 * (d_ms / 1000.0)) as i32;
        let frame_dt = 2.0 / frames as f32;
        let frame_len = (w * h * 4) as usize;
        let mut t = -1.0;
        let mut result = Vec::with_capacity(frames.max(0) as usize);
        for _i in 0..frames {
            let mut frame_buffer = vec![0u8; frame_len];
            self.render_into::<S>(&mut frame_buffer, true, pics.clone(), w, h, t);
            result.push(frame_buffer);
            t += frame_dt;
        }
        result
    }

    pub fn coord(&self) -> &CoordinateSystem {
        match self {
            Pic::Mono(data) => &data.coord,
            Pic::Grayscale(data) => &data.coord,
            Pic::Gradient(data) => &data.coord,
            Pic::RGB(data) => &data.coord,
            Pic::HSV(data) => &data.coord,
        }
    }

    /// Render into a caller-provided RGBA8 buffer (`w * h * 4` bytes). Lets a
    /// caller reuse one allocation across many renders (e.g. a video encoder).
    pub fn render_into<S: Simd>(
        &self,
        buf: &mut [u8],
        threaded: bool,
        pics: Arc<HashMap<String, ActualPicture>>,
        w: u32,
        h: u32,
        t: f32,
    ) {
        match self {
            Pic::Mono(data) => data.render_into::<S>(buf, threaded, pics, w, h, t),
            Pic::Grayscale(data) => data.render_into::<S>(buf, threaded, pics, w, h, t),
            Pic::Gradient(data) => data.render_into::<S>(buf, threaded, pics, w, h, t),
            Pic::RGB(data) => data.render_into::<S>(buf, threaded, pics, w, h, t),
            Pic::HSV(data) => data.render_into::<S>(buf, threaded, pics, w, h, t),
        }
    }

    pub fn get_rgba8<S: Simd>(
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

    pub fn can_animate(&self) -> bool {
        let mut children = match self {
            Pic::Mono(data) => vec![&data.c],
            Pic::Grayscale(data) => vec![&data.c],
            Pic::Gradient(data) => vec![&data.index],
            Pic::RGB(data) => vec![&data.r, &data.g, &data.b],
            Pic::HSV(data) => vec![&data.h, &data.s, &data.v],
        };
        while !children.is_empty() {
            if let Some(child) = children.pop() {
                if child.is_leaf() {
                    if *child == APTNode::T {
                        return true;
                    }
                } else {
                    for kid in child.get_children().unwrap() {
                        children.push(kid);
                    }
                }
            };
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{DEFAULT_COORDINATE_SYSTEM, DEFAULT_IMAGE_HEIGHT, DEFAULT_IMAGE_WIDTH};
    use crate::parser::lexer::lisp_to_pic;
    use crate::pic::color::Color;
    use image::ImageReader;
    use image::{
        save_buffer_with_format, ColorType, DynamicImage, GenericImageView, ImageBuffer,
        ImageFormat,
    };

    #[test]
    fn test_pic_to_lisp_mono() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let pic = MonoData::new(0, 60, false, &mut rng, &vec![&"eye.jpg".to_string()]);
        let sexpr = pic.to_lisp();

        assert!(
            sexpr.starts_with("( MONO POLAR\n\t(") || sexpr.starts_with("( MONO CARTESIAN\n\t(")
        );
        assert!(sexpr.ends_with("\n)"));
        assert!(sexpr.lines().collect::<Vec<_>>().len() > 1);
    }

    #[test]
    fn test_pic_to_lisp_grayscale() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let pic = GrayscaleData::new(0, 60, false, &mut rng, &vec![&"eye.jpg".to_string()]);
        let sexpr = pic.to_lisp();
        assert!(
            sexpr.starts_with("( GRAYSCALE POLAR\n\t(")
                || sexpr.starts_with("( GRAYSCALE CARTESIAN\n\t(")
        );
        assert!(sexpr.ends_with("\n)"));
        assert!(sexpr.lines().collect::<Vec<_>>().len() > 1);
    }

    #[test]
    fn test_pic_to_lisp_gradient() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let pic = GradientData::new(0, 60, false, &mut rng, &vec![&"eye.jpg".to_string()]);
        let sexpr = pic.to_lisp();
        assert!(
            sexpr.starts_with("( GRADIENT POLAR\n\t(")
                || sexpr.starts_with("( GRADIENT CARTESIAN\n\t(")
        );
        assert!(sexpr.ends_with("\n)"));
        assert!(sexpr.contains("\n\t( COLORS\n\t"));
        assert!(sexpr.contains("\n\t)\n\t"));
        assert!(sexpr.contains("\n\t\t( COLOR ") || sexpr.contains("\n\t\t( STOPCOLOR "));
        assert!(!sexpr.lines().collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn test_pic_to_lisp_rgb() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let pic = RGBData::new(0, 60, false, &mut rng, &vec![&"eye.jpg".to_string()]);
        let sexpr = pic.to_lisp();
        assert!(sexpr.starts_with("( RGB POLAR\n\t(") || sexpr.starts_with("( RGB CARTESIAN\n\t("));
        assert!(sexpr.ends_with("\n)"));
        assert!(sexpr.lines().collect::<Vec<_>>().len() > 3);
    }

    #[test]
    fn test_pic_to_lisp_hsv() {
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let pic = HSVData::new(0, 60, false, &mut rng, &vec![&"eye.jpg".to_string()]);
        let sexpr = pic.to_lisp();
        assert!(sexpr.starts_with("( HSV POLAR\n\t(") || sexpr.starts_with("( HSV CARTESIAN\n\t("));
        assert!(sexpr.ends_with("\n)"));
        assert!(sexpr.lines().collect::<Vec<_>>().len() > 1);
    }

    #[test]
    fn test_pic_coord() {
        assert_eq!(
            lisp_to_pic("(Mono Polar (X) )".to_string(), CoordinateSystem::Polar)
                .unwrap()
                .coord(),
            &CoordinateSystem::Polar
        );
        assert_eq!(
            lisp_to_pic(
                "(Mono Cartesian (X) )".to_string(),
                CoordinateSystem::Cartesian
            )
            .unwrap()
            .coord(),
            &CoordinateSystem::Cartesian
        );
        assert_eq!(
            lisp_to_pic("(Mono (X) )".to_string(), CoordinateSystem::Polar)
                .unwrap()
                .coord(),
            &CoordinateSystem::Polar
        );
    }

    #[test]
    //todo Currently wrong CoordinateSystems are still accepted, but ignored
    fn test_pic_coord_fail() {
        lisp_to_pic("(Mono Lunar (X) )".to_string(), CoordinateSystem::Polar)
            .unwrap()
            .coord();
    }

    #[test]
    fn test_handle_width() {
        let sexpr = "(GrayScale ( / x Width ) )";
        match lisp_to_pic(sexpr.to_string(), CoordinateSystem::Polar) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::Grayscale(GrayscaleData {
                        c: APTNode::Div(vec![APTNode::X, APTNode::Width]),
                        coord: CoordinateSystem::Polar
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( GRAYSCALE POLAR\n\t( ( / X WIDTH ) )\n)");
            }
            Err(err) => {
                panic!("could not parse formula with width {:?}", err);
            }
        }
    }

    #[test]
    fn test_handle_height() {
        let sexpr = "(GrayScale ( / y Height ) )";
        match lisp_to_pic(sexpr.to_string(), CoordinateSystem::Polar) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::Grayscale(GrayscaleData {
                        c: APTNode::Div(vec![APTNode::Y, APTNode::Height]),
                        coord: CoordinateSystem::Polar
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( GRAYSCALE POLAR\n\t( ( / Y HEIGHT ) )\n)");
            }
            Err(err) => {
                panic!("could not parse formula with width {:?}", err);
            }
        }
    }

    #[test]
    fn test_handle_pi() {
        let sexpr = "(GrayScale( sin (/ x PI ) ) )";
        match lisp_to_pic(sexpr.to_string(), CoordinateSystem::Polar) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::Grayscale(GrayscaleData {
                        c: APTNode::Sin(vec![APTNode::Div(vec![APTNode::X, APTNode::PI,])]),
                        coord: CoordinateSystem::Polar
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( GRAYSCALE POLAR\n\t( ( SIN ( / X PI ) ) )\n)");
            }
            Err(err) => {
                panic!("could not parse formula with PI {:?}", err);
            }
        }
    }

    #[test]
    fn test_handle_e() {
        let sexpr = "(GrayScale( Log (/ x E ) ) )";
        match lisp_to_pic(sexpr.to_string(), CoordinateSystem::Polar) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::Grayscale(GrayscaleData {
                        c: APTNode::Log(vec![APTNode::Div(vec![APTNode::X, APTNode::E,])]),
                        coord: CoordinateSystem::Polar
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( GRAYSCALE POLAR\n\t( ( LOG ( / X E ) ) )\n)");
            }
            Err(err) => {
                panic!("could not parse formula with E {:?}", err);
            }
        }
    }

    #[test]
    fn test_handle_mono_coord_system_polar() {
        let sexpr = "(Mono POLAR ( X ))";
        match lisp_to_pic(sexpr.to_string(), DEFAULT_COORDINATE_SYSTEM) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::Mono(MonoData {
                        c: APTNode::X,
                        coord: CoordinateSystem::Polar
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( MONO POLAR\n\t( X )\n)");
            }
            Err(err) => {
                panic!("could not parse formula with E {:?}", err);
            }
        }
    }

    #[test]
    fn test_handle_mono_coord_system_cartesian() {
        let sexpr = "(Mono CARTESIAN ( X )";
        match lisp_to_pic(sexpr.to_string(), DEFAULT_COORDINATE_SYSTEM) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::Mono(MonoData {
                        c: APTNode::X,
                        coord: CoordinateSystem::Cartesian
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( MONO CARTESIAN\n\t( X )\n)"); //todo if coord != DEFAULT print
            }
            Err(err) => {
                panic!("could not parse formula with E {:?}", err);
            }
        }
    }

    #[test]
    fn test_handle_rgb_coord_system_cartesian() {
        let sexpr = "(RGB CARTESIAN ( X ) (Y) (T) )";
        match lisp_to_pic(sexpr.to_string(), DEFAULT_COORDINATE_SYSTEM) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::RGB(RGBData {
                        r: APTNode::X,
                        g: APTNode::Y,
                        b: APTNode::T,
                        coord: CoordinateSystem::Cartesian
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( RGB CARTESIAN\n\t( X )\n\t( Y )\n\t( T )\n)");
                //todo if coord != DEFAULT print
            }
            Err(err) => {
                panic!("could not parse formula with E {:?}", err);
            }
        }
    }

    #[test]
    fn test_handle_rgb_coord_system_polar() {
        let sexpr = "(RGB POLAR ( X ) ( Y ) (T) )";
        match lisp_to_pic(sexpr.to_string(), DEFAULT_COORDINATE_SYSTEM) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::RGB(RGBData {
                        r: APTNode::X,
                        g: APTNode::Y,
                        b: APTNode::T,
                        coord: CoordinateSystem::Polar
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( RGB POLAR\n\t( X )\n\t( Y )\n\t( T )\n)");
            }
            Err(err) => {
                panic!("could not parse formula with E {:?}", err);
            }
        }
    }

    #[test]
    fn test_handle_hsv_coord_system_cartesian() {
        let sexpr = "(HSV CARTESIAN ( X ) (Y) (T)";
        match lisp_to_pic(sexpr.to_string(), DEFAULT_COORDINATE_SYSTEM) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::HSV(HSVData {
                        h: APTNode::X,
                        s: APTNode::Y,
                        v: APTNode::T,
                        coord: CoordinateSystem::Cartesian
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( HSV CARTESIAN\n\t( X )\n\t( Y )\n\t( T )\n)");
                //todo if coord != DEFAULT print
            }
            Err(err) => {
                panic!("could not parse formula with E {:?}", err);
            }
        }
    }

    #[test]
    fn test_handle_hsv_coord_system_polar() {
        let sexpr = "(HSV POLAR ( X ) ( Y) (T) )";
        match lisp_to_pic(sexpr.to_string(), DEFAULT_COORDINATE_SYSTEM) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::HSV(HSVData {
                        h: APTNode::X,
                        s: APTNode::Y,
                        v: APTNode::T,
                        coord: CoordinateSystem::Polar
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( HSV POLAR\n\t( X )\n\t( Y )\n\t( T )\n)");
            }
            Err(err) => {
                panic!("could not parse formula with E {:?}", err);
            }
        }
    }
    #[test]
    fn test_handle_grayscale_coord_system_cartesian() {
        let sexpr = "(GrayScale CARTESIAN ( X )";
        match lisp_to_pic(sexpr.to_string(), DEFAULT_COORDINATE_SYSTEM) {
            Ok(pic) => {
                assert_eq!(
                    pic,
                    Pic::Grayscale(GrayscaleData {
                        c: APTNode::X,
                        coord: CoordinateSystem::Cartesian
                    })
                );
                let resexpr = pic.to_lisp();
                assert_eq!(resexpr, "( GRAYSCALE CARTESIAN\n\t( X )\n)"); //todo if coord != DEFAULT print
            }
            Err(err) => {
                panic!("could not parse formula with E {:?}", err);
            }
        }
    }

    #[test]
    fn test_crash_with_dims_mono() {
        let crashes_at_dim = (100, 100);
        let pictures = Arc::new(HashMap::new());
        let pic = Pic::Mono(MonoData {
            c: APTNode::X,
            coord: CoordinateSystem::Polar,
        });
        let _x = pic_get_rgba8(
            &pic,
            true,
            pictures,
            crashes_at_dim.0,
            crashes_at_dim.1,
            0.0,
        );
    }

    #[test]
    fn test_crash_with_dims_grayscale() {
        let crashes_at_dim = (100, 100);
        let pictures = Arc::new(HashMap::new());
        let pic = Pic::Grayscale(GrayscaleData {
            c: APTNode::X,
            coord: CoordinateSystem::Polar,
        });
        let _x = pic_get_rgba8(
            &pic,
            true,
            pictures,
            crashes_at_dim.0,
            crashes_at_dim.1,
            0.0,
        );
    }

    #[test]
    fn test_crash_with_dims_gradient() {
        let crashes_at_dim = (100, 100);
        let pictures = Arc::new(HashMap::new());
        let pic = Pic::Gradient(GradientData {
            colors: vec![
                (
                    Color {
                        r: 0.3690771,
                        g: 0.7165854,
                        b: 0.075644374,
                        a: 1.0,
                    },
                    false,
                ),
                (
                    Color {
                        r: 0.39675784,
                        g: 0.10509944,
                        b: 0.82246256,
                        a: 1.0,
                    },
                    false,
                ),
            ],
            index: APTNode::X,
            coord: CoordinateSystem::Polar,
        });
        let _x = pic_get_rgba8(
            &pic,
            true,
            pictures,
            crashes_at_dim.0,
            crashes_at_dim.1,
            0.0,
        );
    }

    #[test]
    fn test_crash_with_dims_hsv() {
        let crashes_at_dim = (100, 100);
        let pictures = Arc::new(HashMap::new());
        let pic = Pic::HSV(HSVData {
            h: APTNode::X,
            s: APTNode::Y,
            v: APTNode::T,
            coord: CoordinateSystem::Polar,
        });
        let _x = pic_get_rgba8(
            &pic,
            true,
            pictures,
            crashes_at_dim.0,
            crashes_at_dim.1,
            0.0,
        );
    }

    #[test]
    fn test_crash_with_dims_rgb() {
        let crashes_at_dim = (100, 100);
        let pictures = Arc::new(HashMap::new());
        let pic = Pic::RGB(RGBData {
            r: APTNode::X,
            g: APTNode::Y,
            b: APTNode::T,
            coord: CoordinateSystem::Polar,
        });
        let _x = pic_get_rgba8(
            &pic,
            true,
            pictures,
            crashes_at_dim.0,
            crashes_at_dim.1,
            0.0,
        );
    }

    fn render_source_and_read_sample_file(
        source: String,
        sample_file: &str,
        overwrite: bool,
    ) -> (DynamicImage, DynamicImage) {
        let pictures = Arc::new(HashMap::new());
        let pic = lisp_to_pic(source, DEFAULT_COORDINATE_SYSTEM).unwrap();
        let gen_rgba8 = pic_get_rgba8(
            &pic,
            true,
            pictures,
            DEFAULT_IMAGE_WIDTH,
            DEFAULT_IMAGE_HEIGHT,
            0.0,
        );
        if overwrite {
            save_buffer_with_format(
                sample_file,
                gen_rgba8.as_slice(),
                DEFAULT_IMAGE_WIDTH,
                DEFAULT_IMAGE_HEIGHT,
                ColorType::Rgba8,
                ImageFormat::Png,
            )
            .unwrap();
        }
        let gen_buf =
            ImageBuffer::from_raw(DEFAULT_IMAGE_WIDTH, DEFAULT_IMAGE_HEIGHT, gen_rgba8).unwrap();
        let generated = DynamicImage::ImageRgba8(gen_buf);

        let read_img = ImageReader::open(sample_file).unwrap().decode().unwrap();
        let read = DynamicImage::ImageRgba8(read_img.into_rgba8());

        (generated, read)
    }

    #[test]
    fn test_render_3d_4d_noise_nodes() {
        // Each new multi-dimensional noise node must parse and render without an
        // out-of-bounds panic in the execute stack arithmetic. Children are
        // coords..., freq, offset, then lac+gain (fbm/ridge/turbulence) or
        // jitter (cell).
        let pics: Arc<HashMap<String, ActualPicture>> = Arc::new(HashMap::new());
        let sources = [
            "( Mono ( FBM3 X Y T 0.5 0.0 2.0 0.5 ) )",
            "( Mono ( FBM4 X Y T X 0.5 0.0 2.0 0.5 ) )",
            "( Mono ( RIDGE3 X Y T 0.5 0.0 2.0 0.5 ) )",
            "( Mono ( RIDGE4 X Y T X 0.5 0.0 2.0 0.5 ) )",
            "( Mono ( TURBULENCE3 X Y T 0.5 0.0 2.0 0.5 ) )",
            "( Mono ( TURBULENCE4 X Y T X 0.5 0.0 2.0 0.5 ) )",
            "( Mono ( CELL1_3 X Y T 0.5 0.0 0.5 ) )",
            "( Mono ( CELL1_4 X Y T X 0.5 0.0 0.5 ) )",
            "( Mono ( CELL2_3 X Y T 0.5 0.0 0.5 ) )",
            "( Mono ( CELL2_4 X Y T X 0.5 0.0 0.5 ) )",
        ];
        for src in sources {
            let pic = lisp_to_pic(src.to_string(), CoordinateSystem::Cartesian)
                .unwrap_or_else(|e| panic!("parse failed for `{}`: {}", src, e));
            let out = pic_get_rgba8(&pic, false, pics.clone(), 8, 8, 0.0);
            assert_eq!(out.len(), 8 * 8 * 4, "wrong buffer size for `{}`", src);
        }
    }

    #[test]
    fn test_parse_and_render_mono() {
        let img_file = "./samples/mono.png";
        let source = r#"
( MONO CARTESIAN
	( ( ATAN ( + ( CELL1 Y Y Y X ( - Y 0.7253959 ) ) ( ATAN X ) ) ) )
)
        "#;
        let (generated, read) =
            render_source_and_read_sample_file(source.to_string(), img_file, false);
        assert_eq!(generated.dimensions(), read.dimensions());
        assert_eq!(generated.as_bytes(), read.as_bytes());
    }

    #[test]
    fn test_parse_and_render_grayscale() {
        let img_file = "./samples/grayscale.png";
        let source = r#"
( GRAYSCALE POLAR
	( ( LOG ( + ( CELL1 ( LOG ( RIDGE ( SQRT Y ) Y Y X X 0.5701809 ) ) ( ATAN Y ) ( % Y 0.12452102 ) ( FLOOR ( ATAN2 Y Y ) ) ( SIN Y ) ) ( * ( + X ( SIN ( - ( ATAN2 Y X ) X ) ) ) ( ATAN ( LOG ( FLOOR ( SIN ( TURBULENCE Y 0.91551733 ( SQRT ( SQRT X ) ) ( MIN X Y ) -0.83923936 ( MANDELBROT Y X ) ) ) ) ) ) ) ) ) )
)
        "#;
        let (generated, read) =
            render_source_and_read_sample_file(source.to_string(), img_file, false);
        assert_eq!(generated.dimensions(), read.dimensions());
        assert_eq!(generated.as_bytes(), read.as_bytes());
    }

    #[test]
    fn test_parse_and_render_hsv() {
        let img_file = "./samples/hsv.png";
        let source = r#"
( HSV CARTESIAN
	( ( SQUARE ( / ( MANDELBROT X Y ) 0.7601185 ) ) )
	( ( + ( TAN ( TAN ( RIDGE Y ( ATAN -0.74197626 ) ( + Y Y ) Y ( CLAMP Y ) ( + X Y ) ) ) ) ( ATAN2 X Y ) ) )
	( ( MAX -0.9284358 Y ) )
)
        "#;
        let (generated, read) =
            render_source_and_read_sample_file(source.to_string(), img_file, false);
        assert_eq!(generated.dimensions(), read.dimensions());
        assert_eq!(generated.as_bytes(), read.as_bytes());
    }

    #[test]
    fn test_parse_and_render_rgb() {
        let img_file = "./samples/rgb.png";
        let source = r#"
( RGB CARTESIAN
	( ( * ( TAN ( CLAMP ( ATAN ( SQRT ( MAX ( ABS ( FLOOR ( RIDGE 0.12349105 ( + X 0.500072 ) X X ( MAX Y Y ) 0.6249633 ) ) ) ( % ( CLAMP ( * Y ( SQUARE 0.39180493 ) ) ) ( WRAP ( CELL2 Y ( MIN -0.5756769 Y ) ( ABS 0.8329663 ) Y Y ) ) ) ) ) ) ) ) ( WRAP ( MANDELBROT ( SQRT ( TURBULENCE ( WRAP X ) 0.26766992 ( MANDELBROT -0.7147219 0.46446967 ) ( LOG 0.6340864 ) Y Y ) ) ( SQUARE ( * ( SIN ( / Y ( RIDGE X Y Y 0.49542284 X ( CEIL -0.7545812 ) ) ) ) ( CEIL ( TURBULENCE ( ATAN X ) X -0.52819157 -0.86907744 0.49089026 ( ATAN -0.5986686 ) ) ) ) ) ) ) ) )
	( ( / ( TURBULENCE ( FBM Y ( * ( RIDGE Y X X X X Y ) -0.98887086 ) 0.21490455 X X ( LOG X ) ) X ( % ( FLOOR X ) ( + X ( ATAN2 0.19268274 Y ) ) ) ( FBM Y -0.28251457 0.632663 X X X ) ( CEIL ( SQRT 0.8429725 ) ) ( WRAP ( MAX Y ( SQUARE ( TAN X ) ) ) ) ) ( FLOOR ( CELL1 ( + -0.5022187 ( LOG X ) ) ( RIDGE -0.8493159 Y ( TAN X ) Y Y Y ) ( ATAN ( SIN ( / ( ABS X ) ( CEIL 0.05049467 ) ) ) ) ( ATAN X ) ( TAN ( / ( FBM X X 0.802964 0.3002789 0.8905289 -0.06338668 ) ( SQUARE ( % X 0.48889422 ) ) ) ) ) ) ) )
	( ( ATAN ( SIN X ) ) )
)
        "#;
        let (generated, read) =
            render_source_and_read_sample_file(source.to_string(), img_file, false);
        assert_eq!(generated.dimensions(), read.dimensions());
        assert_eq!(generated.as_bytes(), read.as_bytes());
    }

    #[test]
    fn test_parse_and_render_gradient() {
        let img_file = "./samples/gradient.png";
        let source = r#"
( GRADIENT POLAR
	( COLORS
		( COLOR 0.38782334 0.18356442 0.5526812 )
		( COLOR 0.40132487 0.9418049 0.79687893 )
	)
	( FBM ( WRAP ( TAN ( - -0.90357685 ( ATAN Y ) ) ) ) ( ABS X ) ( ATAN2 Y X ) ( MAX Y ( MAX X X ) ) ( SQUARE ( CELL2 ( TAN Y ) Y Y X X ) ) ( * Y 0.009492159 ) )
)
        "#;
        let (generated, read) =
            render_source_and_read_sample_file(source.to_string(), img_file, false);
        assert_eq!(generated.dimensions(), read.dimensions());
        assert_eq!(generated.as_bytes(), read.as_bytes());
    }

    #[test]
    fn test_has_t_apt() {
        let source = r#"( MONO POLAR ( MAX X Y ) )"#;
        let pic = lisp_to_pic(source.to_string(), DEFAULT_COORDINATE_SYSTEM).unwrap();
        assert!(!pic.can_animate());

        let source = r#"( GRAYSCALE POLAR ( MAX T Y ) )"#;
        let pic = lisp_to_pic(source.to_string(), DEFAULT_COORDINATE_SYSTEM).unwrap();
        assert!(pic.can_animate());

        let source = r#"( RGB CARTESIAN ( ( x ) ( Y )  ( T ) ) )"#;
        let pic = lisp_to_pic(source.to_string(), DEFAULT_COORDINATE_SYSTEM).unwrap();
        assert!(pic.can_animate());
    }
}
