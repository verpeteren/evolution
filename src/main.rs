// todo
// - fix up gradient to work properly when parsing
// - cross breeding of picture expressions
// - load up thumbnails in a background thread so ui isn't blocked

mod ui;
extern crate evolution;

extern crate image;
extern crate minifb;

use std::fs::{copy, create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "ui")]
use evolution::ui::{fsm::FSM, state::State};
use evolution::{
    filename_to_copy_to, get_picture_path, keep_aspect_ratio, lisp_to_pic, load_pictures,
    pic_get_rgba8, pic_get_video, pic_simplify,
    ActualPicture, Args, Pic, DEFAULT_FILE_OUT, DEFAULT_FPS, DEFAULT_VIDEO_DURATION, EXEC_NAME,
};
#[cfg(feature = "ui")]
use evolution::{
    EXEC_UI_THUMB_COLS, EXEC_UI_THUMB_HEIGHT, EXEC_UI_THUMB_ROWS, EXEC_UI_THUMB_WIDTH,
};

use clap::Parser;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{save_buffer_with_format, ColorType, Frame, ImageBuffer, ImageFormat};
use minifb::{Key, Scale, Window, WindowOptions};
use notify::{
    event::{AccessKind, AccessMode},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};

fn main_gui(args: &Args) -> Result<(), String> {
    match rayon::ThreadPoolBuilder::new()
        .num_threads(0)
        .build_global()
    {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    }

    let mut state = State::new(args)?;
    let options = WindowOptions {
        scale: Scale::X1,
        resize: false,
        ..WindowOptions::default()
    };
    let mut window = Window::new(
        EXEC_NAME,
        args.width as usize,
        args.height as usize,
        options,
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window.set_target_fps(DEFAULT_FPS as usize);
    window.topmost(true);

    let mut fsm = FSM::default();
    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }
        fsm = (fsm.cb)(&mut state, &window, fsm.pic);
        if fsm.stop {
            break;
        }
        let u32_buffer: Vec<u32> = state
            .image
            .as_raw()
            .chunks(4) // 3 -> 4
            .map(|v| ((v[0] as u32) << 16) | ((v[1] as u32) << 8) | v[2] as u32)
            .collect();
        window
            .update_with_buffer(&u32_buffer, args.width as usize, args.height as usize)
            .unwrap();
    }
    Ok(())
}

fn select_image_format(out_file: &Path) -> (ImageFormat, bool) {
    match out_file.extension() {
        Some(ext) => {
            match ext
                .to_str()
                .expect("Invalid file extension")
                .to_lowercase()
                .as_str()
            {
                // support these?
                "tga" => (ImageFormat::Tga, false),
                "dds" => (ImageFormat::Dds, false),
                "hdr" => (ImageFormat::Hdr, false),
                "farb" => (ImageFormat::Farbfeld, false),
                // these do imply video!
                "gif" => (ImageFormat::Gif, true),
                "avi" => (ImageFormat::Avif, false), // Todo: find out how to create avi writer
                // commodity
                "bmp" => (ImageFormat::Bmp, false),
                "ico" => (ImageFormat::Ico, false),
                "webp" => (ImageFormat::WebP, false),
                "pnm" => (ImageFormat::Pnm, false),
                "tif" | "tiff" => (ImageFormat::Tiff, false),
                "jpg" | "jpeg" => (ImageFormat::Jpeg, false),
                "png" => (ImageFormat::Png, false),
                _ => (ImageFormat::Png, false),
            }
        }
        None => (ImageFormat::Png, false),
    }
}

fn main_cli(args: &Args) -> Result<(PathBuf, PathBuf), String> {
    let out_filename = args.output.as_ref().expect("Invalid filename");
    let input_filename = args.input.as_ref().expect("Invalid filename");
    let (width, height, t) = (args.width, args.height, args.time);
    assert!(t >= 0.0);
    let pic_path = get_picture_path(args);
    let pictures = Arc::new(
        load_pictures(pic_path.as_path())
            .map_err(|e| format!("Cannot load picture folder. {:?}", e))?,
    );
    let mut contents = String::new();
    if input_filename == "-" {
        let _bytes = std::io::stdin()
            .read_to_string(&mut contents)
            .map_err(|e| format!("Cannot read from stdin. {}", e));
    } else {
        let mut file =
            File::open(input_filename).map_err(|e| format!("Cannot open input filename. {}", e))?;
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Cannot read input filename. {}", e))?;
    }
    let mut pic = lisp_to_pic(contents, args.coordinate_system.clone()).unwrap();
    pic_simplify(&mut pic, pictures.clone(), width, height, t);
    let out_file = Path::new(out_filename);
    let (format, mut is_video) = select_image_format(out_file);
    if is_video
        && !pic.can_animate() {
            println!("warning: the T Operator is needed to make an animation");
            is_video = false;
        }
    if is_video {
        assert_eq!(format, ImageFormat::Gif);
        let duration = if t == 0.0 { DEFAULT_VIDEO_DURATION } else { t };
        let raw_frames =
            pic_get_video(&pic, pictures, width, height, DEFAULT_FPS, duration);
        if raw_frames.is_empty() {
            println!("warning: not enough frames to make a usefull gif");
        } else {
            let file_out = File::create(out_file).unwrap();
            let mut encoder = GifEncoder::new(&file_out);
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for rgba8 in raw_frames {
                let gen_buf = ImageBuffer::from_raw(width, height, rgba8).unwrap();
                let rgba_img = gen_buf;
                let frame = Frame::new(rgba_img);
                encoder.encode_frame(frame).unwrap();
            }
        }
    } else {
        let rgba8 = pic_get_rgba8(&pic, false, pictures, width, height, t);
        save_buffer_with_format(
            out_file,
            &rgba8[0..],
            width,
            height,
            ColorType::Rgba8,
            format,
        )
        .map_err(|e| format!("Could not save {}", e))?;
    }
    Ok((
        Path::new(input_filename).to_path_buf(),
        out_file.to_path_buf(),
    ))
}

pub fn main() {
    let mut args = Args::parse();
    let run_gui = match &args.input {
        None => true,
        Some(_x) => {
            if args.output.is_none() {
                args.output = Some(DEFAULT_FILE_OUT.to_string());
            }
            false
        }
    };
    if run_gui {
        let min_width = EXEC_UI_THUMB_ROWS as u32 * EXEC_UI_THUMB_WIDTH;
        let min_height = EXEC_UI_THUMB_COLS as u32 * EXEC_UI_THUMB_HEIGHT;
        if min_width <= args.width {
            args.width = min_width;
        }
        if min_height <= args.height {
            args.height = min_height;
        }
        //todo keep also aspect ratio for thumbs and recalculate dimensions
        // calculate it once and set it to the the state to avoid usage of THUMBS constants
        main_gui(&args).unwrap();
    } else {
        let input_filename = args.input.as_ref().unwrap();
        let one_shot = input_filename == "-" || args.copy_path.is_none();
        if one_shot {
            let (_sexpr_filename, _img_filename) = main_cli(&args).unwrap();
        } else {
            let copy_path = args.copy_path.as_ref().unwrap();
            let target_dir = Path::new(&copy_path);
            if !target_dir.exists() {
                println!("Creating {} directory", copy_path);
                create_dir_all(target_dir).unwrap();
            }
            let input_file = Path::new(input_filename);
            println!("Watching changes to {}", input_filename);
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
            watcher
                .watch(input_file.as_ref(), RecursiveMode::NonRecursive)
                .unwrap();
            for res in rx {
                match res {
                    /*
                    If you came here to find out why this runs only during the first save, welcome!
                    Your editor is probably swapping files instead of actually writing them.
                    Try these workarounds:
                    - for vim users:
                      set backupcopy=yes
                      set nobackup
                      set nowritebackup
                    - use a real filesystem watcher like [entr](http://eradman.com/entrproject/)
                    - fix this, preferably by commiting something to [notify](https://crates.io/crates/notify)
                      watch the directory instead of a file, for every event, if the filename matches, then launch
                    */
                    Ok(event) => {
                        match event.kind {
                            EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                                println!("file {} changed, rerunning", input_filename);
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();
                                // todo better handle errors during run
                                if let Ok((sexpr_filename, img_filename)) = main_cli(&args) {
                                    let dest = filename_to_copy_to(
                                        target_dir,
                                        now,
                                        &sexpr_filename.file_name().unwrap().to_string_lossy(),
                                    );
                                    copy(&sexpr_filename, dest.as_path()).unwrap();

                                    let dest = filename_to_copy_to(
                                        target_dir,
                                        now,
                                        &img_filename.file_name().unwrap().to_string_lossy(),
                                    );
                                    copy(img_filename, dest.as_path()).unwrap();
                                    println!(
                                        ".. ran and copied as {} and {}",
                                        sexpr_filename.display(),
                                        dest.display()
                                    );
                                }
                            }
                            EventKind::Remove(_) => {
                                eprintln!("File was removed {:?}", input_filename);
                                exit(1);
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        eprintln!("watch error: {:?}", e);
                        exit(1);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_image_format() {
        assert_eq!(
            select_image_format(Path::new("somefile.tga")),
            (ImageFormat::Tga, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.dds")),
            (ImageFormat::Dds, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.hdr")),
            (ImageFormat::Hdr, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.farb")),
            (ImageFormat::Farbfeld, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.gif")),
            (ImageFormat::Gif, true)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.avi")),
            (ImageFormat::Avif, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.bmp")),
            (ImageFormat::Bmp, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.ico")),
            (ImageFormat::Ico, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.webp")),
            (ImageFormat::WebP, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.pnm")),
            (ImageFormat::Pnm, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.tiff")),
            (ImageFormat::Tiff, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.tif")),
            (ImageFormat::Tiff, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.jpeg")),
            (ImageFormat::Jpeg, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.jpg")),
            (ImageFormat::Jpeg, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.png")),
            (ImageFormat::Png, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.Png")),
            (ImageFormat::Png, false)
        );
        assert_eq!(
            select_image_format(Path::new("somefile.PNG")),
            (ImageFormat::Png, false)
        );
        assert_eq!(
            select_image_format(Path::new("./somedir")),
            (ImageFormat::Png, false)
        );
    }
}
