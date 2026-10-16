use crate::Pic;

use image::math::Rect;

#[derive(Clone)]
pub struct Button {
    pub pic: Pic,
    pub rect: Rect,
}

impl Button {
    pub fn new(pic: Pic, rect: Rect) -> Self {
        Button { pic, rect }
    }
    pub fn hit(&self, x: u32, y: u32) -> bool {
        
        self.rect.x <= x
            && x < (self.rect.x + self.rect.width)
            && self.rect.y <= y
            && y < (self.rect.y + self.rect.height)
    }
}

/*
// fixme: get the imports sorted in combination with the --features="ui"
#[cfg(test)]
pub mod mock {
    use super::*;
    extern crate evolution;
    use evolution::{constants::DEFAULT_COORDINATE_SYSTEM, lisp_to_pic};

    pub fn mock_button() -> Button {
        let source = r#"( MONO POLAR ( MAX Y X ))"#;
        let pic = lisp_to_pic(source.to_string(), DEFAULT_COORDINATE_SYSTEM).unwrap();
        let rect = Rect {
            x: 10,
            y: 20,
            width: 30,
            height: 40,
        };
        let button = Button::new(pic.clone(), rect.clone());
        button
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use mock::mock_button;

    #[test]
    fn test_button_new() {
        let button = mock_button();
        //assert_eq!(&button.pic, &pic);
        assert_eq!(
            &button.rect,
            &Rect {
                x: 10,
                y: 20,
                width: 30,
                height: 40
            }
        );
    }
    #[test]
    fn test_button_hit() {
        let button = mock_button();
        assert_eq!(button.hit(09, 20), false);
        assert_eq!(button.hit(10, 20), true);
        assert_eq!(button.hit(39, 20), true);
        assert_eq!(button.hit(40, 20), false);
        assert_eq!(button.hit(10, 59), true);
        assert_eq!(button.hit(10, 60), false);
        assert_eq!(button.hit(39, 59), true);
        assert_eq!(button.hit(40, 60), false);
    }
}
*/
