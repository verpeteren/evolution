use crate::ui::state::State;
use crate::{
    keep_aspect_ratio, pic_get_rgba8, Pic, EXEC_NAME, EXEC_UI_THUMB_COLS,
    EXEC_UI_THUMB_HEIGHT, EXEC_UI_THUMB_ROWS, EXEC_UI_THUMB_WIDTH,
};

use image::{imageops::overlay, ImageBuffer};
use minifb::{Key, MouseButton, MouseMode, Window};

pub type FsmCbt = for<'a, 'b> fn(&'a mut State, &'b Window, Option<Pic>) -> FSM;

#[allow(clippy::upper_case_acronyms)]
pub struct FSM {
    pub cb: FsmCbt,
    pub stop: bool,
    pub pic: Option<Pic>,
}

impl Default for FSM {
    fn default() -> Self {
        Self {
            cb: _fsm_regenerate,
            stop: false,
            pic: None,
        }
    }
}

fn _fsm_regenerate(state: &mut State, _window: &Window, _pic: Option<Pic>) -> FSM {
    println!("repopulating, please be patient");
    state.generate_buttons();
    FSM {
        cb: _fsm_select_prep,
        ..FSM::default()
    }
}

fn _fsm_select_prep(state: &mut State, _window: &Window, pic: Option<Pic>) -> FSM {
    assert!(pic.is_none());
    assert_eq!(state.buttons.len(), EXEC_UI_THUMB_ROWS);
    assert_eq!(state.buttons.first().unwrap().len(), EXEC_UI_THUMB_COLS);
    let (twidth, theight) = keep_aspect_ratio(
        state.dimensions,
        (EXEC_UI_THUMB_WIDTH, EXEC_UI_THUMB_HEIGHT),
    );
    //todo: rayon par_iter
    for (r, row) in state.buttons.iter().enumerate() {
        for (c, button) in row.iter().enumerate() {
            let generated_buffer = pic_get_rgba8(
                &button.pic,
                false,
                state.pictures.clone(),
                twidth,
                theight,
                state.frame_elapsed(),
            );
            let img = ImageBuffer::from_raw(twidth, theight, &generated_buffer[0..]).unwrap();
            overlay(
                &mut state.image,
                &img,
                (c as u32 * twidth) as i64,
                (r as u32 * theight) as i64,
            );
        }
    }
    FSM {
        cb: _fsm_select_show,
        pic,
        ..FSM::default()
    }
}

fn _fsm_select_show(state: &mut State, window: &Window, pic: Option<Pic>) -> FSM {
    assert!(pic.is_none());
    if window.is_key_down(Key::Escape) {
        return FSM {
            cb: _fsm_exit,
            ..FSM::default()
        };
    }
    if window.is_key_down(Key::Space) {
        return FSM {
            cb: _fsm_regenerate,
            ..FSM::default()
        };
    }
    let right = window.get_mouse_down(MouseButton::Right);
    let left = window.get_mouse_down(MouseButton::Left);
    if right || left {
        if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
            //todo: rayon par_iter
            for row in &state.buttons {
                for button in row {
                    if button.hit(x as u32, y as u32) {
                        if right {
                            return FSM {
                                cb: _fsm_zoom_prep,
                                pic: Some(button.pic.clone()),
                                ..FSM::default()
                            };
                        }
                        if left {
                            state.save_to_files(&button.pic, EXEC_NAME);
                        }
                    }
                }
            }
        }
    }
    FSM {
        cb: _fsm_select_show,
        pic,
        ..FSM::default()
    }
}

fn _fsm_zoom_prep(state: &mut State, window: &Window, wpic: Option<Pic>) -> FSM {
    assert!(wpic.is_some());
    let pic = wpic.as_ref().unwrap();
    if window.is_key_down(Key::Escape) {
        return FSM {
            cb: _fsm_exit,
            ..FSM::default()
        };
    }
    let (width, height) = state.dimensions;
    let generated_buffer = pic_get_rgba8(
        pic,
        false,
        state.pictures.clone(),
        width,
        height,
        state.frame_elapsed(),
    );
    let img = ImageBuffer::from_raw(width, height, &generated_buffer[0..]).unwrap();
    overlay(&mut state.image, &img, 0, 0);
    FSM {
        cb: _fsm_zoom_show,
        pic: wpic,
        ..FSM::default()
    }
}

fn _fsm_zoom_show(state: &mut State, window: &Window, wpic: Option<Pic>) -> FSM {
    assert!(wpic.is_some());
    let pic = wpic.as_ref().unwrap();
    if window.is_key_down(Key::Escape) {
        return FSM {
            cb: _fsm_exit,
            ..FSM::default()
        };
    }

    if window.get_mouse_down(MouseButton::Right) {
        return FSM {
            cb: _fsm_select_prep,
            ..FSM::default()
        };
    }
    if window.get_mouse_down(MouseButton::Left) {
        state.save_to_files(pic, EXEC_NAME);
    }
    FSM {
        cb: _fsm_zoom_show,
        pic: wpic,
        ..FSM::default()
    }
}

fn _fsm_exit(_state: &mut State, _window: &Window, pic: Option<Pic>) -> FSM {
    assert!(pic.is_none());
    //todo: some cleanup here, before we set the stop flag
    FSM {
        cb: _fsm_exit,
        stop: true,
        ..FSM::default()
    }
}
