use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rand::rngs::StdRng;
use rand::SeedableRng;

use image::math::Rect;
use image::{save_buffer_with_format, ColorType, ImageFormat, RgbaImage};

use crate::filename_to_copy_to;
use crate::ui::button::Button;
use crate::{
    get_picture_path, keep_aspect_ratio, load_pictures, pic_get_rgba8,
    pic_simplify, ActualPicture, Args, Pic, EXEC_UI_THUMB_COLS,
    EXEC_UI_THUMB_HEIGHT, EXEC_UI_THUMB_ROWS, EXEC_UI_THUMB_WIDTH,
};

pub struct State {
    pub buttons: Vec<Vec<Button>>,
    pub pictures: Arc<HashMap<String, ActualPicture>>,
    pub dimensions: (u32, u32),
    rng: StdRng,
    offset: f32,
    start_time: Duration,
    pub image: RgbaImage,
}

impl State {
    pub fn new(args: &Args) -> Result<State, String> {
        let dimensions = (args.width, args.height);
        let pic_path = get_picture_path(&args);
        let pictures = Arc::new(
            load_pictures(pic_path.as_path())
                .map_err(|e| format!("Cannot load picture folder. {:?}", e))?,
        );

        let state = State {
            buttons: Vec::new(), //this will be overridden by generate_buttons() during _fsm_regenerate_
            pictures,
            dimensions,
            rng: StdRng::from_rng(&mut rand::rng()),
            offset: args.time,
            start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            image: RgbaImage::new(args.width, args.height),
        };
        Ok(state)
    }

    pub fn generate_buttons(&mut self) {
        let pic_names: Vec<&String> = self.pictures.keys().collect();
        let mut rows = Vec::with_capacity(EXEC_UI_THUMB_ROWS);
        let (twidth, theight) =
            keep_aspect_ratio(self.dimensions, (EXEC_UI_THUMB_WIDTH, EXEC_UI_THUMB_HEIGHT));
        //todo: rayon par_iter
        for r in 0..EXEC_UI_THUMB_ROWS {
            let mut cols = Vec::with_capacity(EXEC_UI_THUMB_COLS);
            for c in 0..EXEC_UI_THUMB_COLS {
                let rect = Rect {
                    x: twidth * c as u32,
                    y: theight * r as u32,
                    width: twidth,
                    height: theight,
                };
                let mut pic = Pic::new(&mut self.rng, &pic_names);
                pic_simplify(
                    &mut pic,
                    self.pictures.clone(),
                    twidth,
                    theight,
                    self.frame_elapsed(),
                );
                let button = Button::new(pic, rect);
                cols.push(button);
            }
            rows.push(cols);
        }
        self.buttons = rows;
        self.start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    }

    pub fn frame_elapsed(&self) -> f32 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let diff = now - self.start_time;
        let offset_from_start = diff.as_millis() as f32 + self.offset;
        offset_from_start //% VIDEO_DURATION
    }

    pub fn save_to_files(&self, pic: &Pic, exec_name: &str) {
        let target_dir = Path::new(".");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let sexpr = pic.to_lisp();
        //let's save this to a sexpr_file
        let ts = self.frame_elapsed();
        let tfn = format!("{}_{}.sexpr", exec_name, ts);
        let sexpr_filename = Path::new(&tfn);
        let dest = filename_to_copy_to(
            &target_dir,
            now,
            &sexpr_filename.file_name().unwrap().to_string_lossy(),
        );
        println!("writing to {:?}", dest);
        File::create(dest)
            .unwrap()
            .write_all(sexpr.as_bytes())
            .unwrap();
        //let's save this to a png file
        let tfn = format!("{}_{}.png", exec_name, ts);
        let png_filename = Path::new(&tfn);
        let dest = filename_to_copy_to(
            &target_dir,
            now,
            &png_filename.file_name().unwrap().to_string_lossy(),
        );
        let (width, height) = self.dimensions;
        let rgba8 =
            pic_get_rgba8(&pic, false, self.pictures.clone(), width, height, ts);
        save_buffer_with_format(
            dest,
            &rgba8[..],
            width,
            height,
            ColorType::Rgba8,
            ImageFormat::Png,
        )
        .unwrap();
    }
}
