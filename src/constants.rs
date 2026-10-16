use crate::pic::coordinatesystem::CoordinateSystem;

pub const DEFAULT_IMAGE_WIDTH: u32 = 1920;
pub const DEFAULT_IMAGE_HEIGHT: u32 = 1080;
pub const DEFAULT_COORDINATE_SYSTEM: CoordinateSystem = CoordinateSystem::Polar;

pub const PIC_RANDOM_TREE_MIN: usize = 1;
pub const PIC_RANDOM_TREE_MAX: usize = 40;

pub const PIC_GRADIENT_STOP_CHANCE: usize = 5; // 1 in 5
pub const PIC_GRADIENT_COUNT_MAX: usize = 10;
pub const PIC_GRADIENT_COUNT_MIN: usize = 2;
pub const PIC_GRADIENT_SIZE: usize = 512;

#[cfg(feature = "ui")]
pub mod exec {
    pub const EXEC_NAME: &str = "Evolution";
    pub const EXEC_UI_THUMB_ROWS: usize = 15;
    pub const EXEC_UI_THUMB_COLS: usize = 14;
    pub const EXEC_UI_THUMB_WIDTH: u32 = 128;
    pub const EXEC_UI_THUMB_HEIGHT: u32 = 72;
    pub const DEFAULT_PICTURES_PATH: &str = "pictures";
    pub const DEFAULT_FILE_OUT: &str = "out.png";
    pub const DEFAULT_FPS: u16 = 15;
    pub const DEFAULT_VIDEO_DURATION: f32 = 5000.0; //milliseconds
}
