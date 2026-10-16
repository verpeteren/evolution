use image::ImageReader;
use image::GenericImageView;

#[readonly::make]
pub struct ActualPicture {
    pub brightness: Vec<f32>,
    pub w: u16,
    pub h: u16,
    name: String,
}

impl ActualPicture {
    pub fn new_via_file(file_name: &str) -> Result<Self, String> {
        let img = ImageReader::open(file_name)
            .expect("Could not open file")
            .decode()
            .expect("Could not decode file");

        let (width, height) = img.dimensions();
        let raw_bytes = img.into_bytes();
        Self::new_from_bytes(&raw_bytes[0..], file_name, width as u16, height as u16)
    }

    pub fn new_from_bytes(raw_bytes: &[u8], name: &str, w: u16, h: u16) -> Result<Self, String> {
        let brightness: Vec<f32> = raw_bytes
            .chunks_exact(4)
            .map(|chunk| {
                let sum: u16 = chunk[0] as u16 + chunk[1] as u16 + chunk[2] as u16;
                (sum as f32 / (255.0 * 3.0)) * 2.0 - 1.0
            })
            .collect();
        Ok(Self {
            brightness,
            w,
            h,
            name: name.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_actualpicture_new_from_bytes() {
        let dummy = "This is not a file".to_string();
        let name = "fake";
        let ap = ActualPicture::new_from_bytes(dummy.as_bytes(), name, 800, 600).unwrap();
        assert_eq!(
            ap.brightness,
            vec![-0.23398691, -0.34117645, -0.11895424, -0.3960784]
        );
        assert_eq!(ap.name, "fake");
        assert_eq!(ap.w, 800);
        assert_eq!(ap.h, 600);
    }
}
