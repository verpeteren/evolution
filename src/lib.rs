#[cfg(feature = "ui")]
pub mod args;

pub mod constants;
pub mod parser;
pub mod pic;
pub mod vm;

use std::collections::HashMap;

use std::fs::read_dir;
use std::path::{Path, PathBuf};

#[cfg(feature = "ui")]
use std::env::var;

#[cfg(feature = "ui")]
pub use args::Args;

pub use constants::{DEFAULT_COORDINATE_SYSTEM, DEFAULT_IMAGE_HEIGHT, DEFAULT_IMAGE_WIDTH};

#[cfg(feature = "ui")]
pub use constants::exec::{
    DEFAULT_FILE_OUT, DEFAULT_FPS, DEFAULT_PICTURES_PATH, DEFAULT_VIDEO_DURATION, EXEC_NAME,
    EXEC_UI_THUMB_COLS, EXEC_UI_THUMB_HEIGHT, EXEC_UI_THUMB_ROWS, EXEC_UI_THUMB_WIDTH,
};
#[cfg(feature = "ui")]
pub mod ui;

pub use parser::lexer::lisp_to_pic;
pub use pic::actual_picture::ActualPicture;
pub use pic::coordinatesystem::CoordinateSystem;
pub use pic::pic::{
    pic_get_rgba8, pic_get_video, pic_simplify, Pic,
};

#[cfg(feature = "ui")]
pub fn get_picture_path(args: &Args) -> PathBuf {
    let mut path_buf = if let Ok(manifest_dir) = var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        PathBuf::from("./")
    };
    path_buf.push(args.pictures_path.clone());
    path_buf
}

pub fn load_pictures(pic_path: &Path) -> Result<HashMap<String, ActualPicture>, String> {
    let mut pictures = HashMap::new();
    //todo rayon par_iter
    for file in read_dir(pic_path).expect(&format!("Cannot read path {:?}", pic_path)) {
        let short_file_name = file
            .as_ref()
            .unwrap()
            .file_name()
            .into_string()
            .expect("Cannot convert file's name ");
        let path = file.as_ref().unwrap().path();
        let full_file_name = path.to_string_lossy();
        if let Ok(pic) = ActualPicture::new_via_file(&full_file_name.to_owned()) {
            pictures.insert(short_file_name, pic);
        }
    }
    Ok(pictures)
}

pub fn keep_aspect_ratio(output: (u32, u32), thumb: (u32, u32)) -> (u32, u32) {
    // todo make this function signature type generic
    let (ow, oh) = output;
    let (tw, th) = thumb;
    assert!(ow > 0);
    assert!(oh > 0);
    assert!(tw > 0);
    assert!(th > 0);
    let ratio = ow as f32 / oh as f32;
    let nth = tw as f32 / ratio;
    (tw, nth.floor() as u32)
}

pub fn filename_to_copy_to(target_dir: &Path, now: u64, filename: &str) -> PathBuf {
    let new_filename = format!("{}_{}", now, filename);
    let mut dest = target_dir.to_path_buf();
    dest.push(Path::new(&new_filename));
    dest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_to_copy_to() {
        assert_eq!(
            filename_to_copy_to(&Path::new("./somedir"), 1100, "somefile.png"),
            Path::new("./somedir/1100_somefile.png").to_path_buf()
        );
    }

    #[test]
    fn test_main_aspect_ratio() {
        assert_eq!(keep_aspect_ratio((800, 600), (128, 128)), (128, 96));
        assert_eq!(keep_aspect_ratio((800, 600), (128, 128)), (128, 96));
        assert_eq!(keep_aspect_ratio((800, 600), (128, 100)), (128, 96));
        assert_eq!(keep_aspect_ratio((1000, 600), (128, 32)), (128, 76));
    }

    #[cfg(feature = "ui")]
    #[test]
    fn test_get_picture_path() {
        let args = Args {
            pictures_path: "pictures".to_string(),
            width: DEFAULT_IMAGE_WIDTH,
            height: DEFAULT_IMAGE_HEIGHT,
            time: 0.0,
            input: None,
            output: None,
            copy_path: None,
            coordinate_system: DEFAULT_COORDINATE_SYSTEM,
        };
        assert!(get_picture_path(&args)
            .to_string_lossy()
            .ends_with("/pictures"));
    }
}
