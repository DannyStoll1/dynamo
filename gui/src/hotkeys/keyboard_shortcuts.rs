use egui::{Key, KeyboardShortcut, Modifiers};

macro_rules! key_press {
    ($key: expr) => {
        KeyboardShortcut::new(Modifiers::NONE, $key)
    };
}

macro_rules! ctrl {
    ($key: expr) => {
        KeyboardShortcut::new(Modifiers::CTRL, $key)
    };
}

macro_rules! shift {
    ($key: expr) => {
        KeyboardShortcut::new(Modifiers::SHIFT, $key)
    };
}

macro_rules! ctrl_shift {
    ($key: expr) => {
        KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), $key)
    };
}

macro_rules! shortcut_used {
    ($ctx: expr, $shortcut: expr) => {
        $ctx.input_mut(|i| i.consume_shortcut($shortcut))
    };
}

pub(crate) use shortcut_used;

pub const CTRL_A: KeyboardShortcut = ctrl!(Key::A);
pub const CTRL_B: KeyboardShortcut = ctrl!(Key::B);
pub const CTRL_C: KeyboardShortcut = ctrl!(Key::C);
pub const CTRL_D: KeyboardShortcut = ctrl!(Key::D);
pub const CTRL_E: KeyboardShortcut = ctrl!(Key::E);
pub const CTRL_F: KeyboardShortcut = ctrl!(Key::F);
pub const CTRL_G: KeyboardShortcut = ctrl!(Key::G);
pub const CTRL_H: KeyboardShortcut = ctrl!(Key::H);
pub const CTRL_I: KeyboardShortcut = ctrl!(Key::I);
pub const CTRL_J: KeyboardShortcut = ctrl!(Key::J);
pub const CTRL_K: KeyboardShortcut = ctrl!(Key::K);
pub const CTRL_L: KeyboardShortcut = ctrl!(Key::L);
pub const CTRL_M: KeyboardShortcut = ctrl!(Key::M);
pub const CTRL_N: KeyboardShortcut = ctrl!(Key::N);
pub const CTRL_O: KeyboardShortcut = ctrl!(Key::O);
pub const CTRL_P: KeyboardShortcut = ctrl!(Key::P);
pub const CTRL_Q: KeyboardShortcut = ctrl!(Key::Q);
pub const CTRL_R: KeyboardShortcut = ctrl!(Key::R);
pub const CTRL_S: KeyboardShortcut = ctrl!(Key::S);
pub const CTRL_T: KeyboardShortcut = ctrl!(Key::T);
pub const CTRL_U: KeyboardShortcut = ctrl!(Key::U);
pub const CTRL_V: KeyboardShortcut = ctrl!(Key::V);
pub const CTRL_W: KeyboardShortcut = ctrl!(Key::W);
pub const CTRL_X: KeyboardShortcut = ctrl!(Key::X);
pub const CTRL_Y: KeyboardShortcut = ctrl!(Key::Y);
pub const CTRL_Z: KeyboardShortcut = ctrl!(Key::Z);

pub const CTRL_0: KeyboardShortcut = ctrl!(Key::Num0);
pub const CTRL_1: KeyboardShortcut = ctrl!(Key::Num1);
pub const CTRL_2: KeyboardShortcut = ctrl!(Key::Num2);
pub const CTRL_3: KeyboardShortcut = ctrl!(Key::Num3);
pub const CTRL_4: KeyboardShortcut = ctrl!(Key::Num4);
pub const CTRL_5: KeyboardShortcut = ctrl!(Key::Num5);
pub const CTRL_6: KeyboardShortcut = ctrl!(Key::Num6);
pub const CTRL_7: KeyboardShortcut = ctrl!(Key::Num7);
pub const CTRL_8: KeyboardShortcut = ctrl!(Key::Num8);
pub const CTRL_9: KeyboardShortcut = ctrl!(Key::Num9);

pub const KEY_A: KeyboardShortcut = key_press!(Key::A);
pub const KEY_B: KeyboardShortcut = key_press!(Key::B);
pub const KEY_C: KeyboardShortcut = key_press!(Key::C);
pub const KEY_D: KeyboardShortcut = key_press!(Key::D);
pub const KEY_E: KeyboardShortcut = key_press!(Key::E);
pub const KEY_F: KeyboardShortcut = key_press!(Key::F);
pub const KEY_G: KeyboardShortcut = key_press!(Key::G);
pub const KEY_H: KeyboardShortcut = key_press!(Key::H);
pub const KEY_I: KeyboardShortcut = key_press!(Key::I);
pub const KEY_J: KeyboardShortcut = key_press!(Key::J);
pub const KEY_K: KeyboardShortcut = key_press!(Key::K);
pub const KEY_L: KeyboardShortcut = key_press!(Key::L);
pub const KEY_M: KeyboardShortcut = key_press!(Key::M);
pub const KEY_N: KeyboardShortcut = key_press!(Key::N);
pub const KEY_O: KeyboardShortcut = key_press!(Key::O);
pub const KEY_P: KeyboardShortcut = key_press!(Key::P);
pub const KEY_Q: KeyboardShortcut = key_press!(Key::Q);
pub const KEY_R: KeyboardShortcut = key_press!(Key::R);
pub const KEY_S: KeyboardShortcut = key_press!(Key::S);
pub const KEY_T: KeyboardShortcut = key_press!(Key::T);
pub const KEY_U: KeyboardShortcut = key_press!(Key::U);
pub const KEY_V: KeyboardShortcut = key_press!(Key::V);
pub const KEY_W: KeyboardShortcut = key_press!(Key::W);
pub const KEY_X: KeyboardShortcut = key_press!(Key::X);
pub const KEY_Y: KeyboardShortcut = key_press!(Key::Y);
pub const KEY_Z: KeyboardShortcut = key_press!(Key::Z);

pub const KEY_0: KeyboardShortcut = key_press!(Key::Num0);
pub const KEY_1: KeyboardShortcut = key_press!(Key::Num1);
pub const KEY_2: KeyboardShortcut = key_press!(Key::Num2);
pub const KEY_3: KeyboardShortcut = key_press!(Key::Num3);
pub const KEY_4: KeyboardShortcut = key_press!(Key::Num4);
pub const KEY_5: KeyboardShortcut = key_press!(Key::Num5);
pub const KEY_6: KeyboardShortcut = key_press!(Key::Num6);
pub const KEY_7: KeyboardShortcut = key_press!(Key::Num7);
pub const KEY_8: KeyboardShortcut = key_press!(Key::Num8);
pub const KEY_9: KeyboardShortcut = key_press!(Key::Num9);

pub const KEY_UP: KeyboardShortcut = key_press!(Key::ArrowUp);
pub const KEY_DOWN: KeyboardShortcut = key_press!(Key::ArrowDown);
pub const KEY_LEFT: KeyboardShortcut = key_press!(Key::ArrowLeft);
pub const KEY_RIGHT: KeyboardShortcut = key_press!(Key::ArrowRight);
pub const KEY_SPACE: KeyboardShortcut = key_press!(Key::Space);
pub const KEY_MINUS: KeyboardShortcut = key_press!(Key::Minus);
pub const KEY_EQUALS: KeyboardShortcut = key_press!(Key::PlusEquals);
pub const KEY_INSERT: KeyboardShortcut = key_press!(Key::Insert);
pub const KEY_HOME: KeyboardShortcut = key_press!(Key::Home);

pub const SHIFT_A: KeyboardShortcut = shift!(Key::A);
pub const SHIFT_B: KeyboardShortcut = shift!(Key::B);
pub const SHIFT_C: KeyboardShortcut = shift!(Key::C);
pub const SHIFT_D: KeyboardShortcut = shift!(Key::D);
pub const SHIFT_E: KeyboardShortcut = shift!(Key::E);
pub const SHIFT_F: KeyboardShortcut = shift!(Key::F);
pub const SHIFT_G: KeyboardShortcut = shift!(Key::G);
pub const SHIFT_H: KeyboardShortcut = shift!(Key::H);
pub const SHIFT_I: KeyboardShortcut = shift!(Key::I);
pub const SHIFT_J: KeyboardShortcut = shift!(Key::J);
pub const SHIFT_K: KeyboardShortcut = shift!(Key::K);
pub const SHIFT_L: KeyboardShortcut = shift!(Key::L);
pub const SHIFT_M: KeyboardShortcut = shift!(Key::M);
pub const SHIFT_N: KeyboardShortcut = shift!(Key::N);
pub const SHIFT_O: KeyboardShortcut = shift!(Key::O);
pub const SHIFT_P: KeyboardShortcut = shift!(Key::P);
pub const SHIFT_Q: KeyboardShortcut = shift!(Key::Q);
pub const SHIFT_R: KeyboardShortcut = shift!(Key::R);
pub const SHIFT_S: KeyboardShortcut = shift!(Key::S);
pub const SHIFT_T: KeyboardShortcut = shift!(Key::T);
pub const SHIFT_U: KeyboardShortcut = shift!(Key::U);
pub const SHIFT_V: KeyboardShortcut = shift!(Key::V);
pub const SHIFT_W: KeyboardShortcut = shift!(Key::W);
pub const SHIFT_X: KeyboardShortcut = shift!(Key::X);
pub const SHIFT_Y: KeyboardShortcut = shift!(Key::Y);
pub const SHIFT_Z: KeyboardShortcut = shift!(Key::Z);

pub const SHIFT_0: KeyboardShortcut = shift!(Key::Num0);
pub const SHIFT_1: KeyboardShortcut = shift!(Key::Num1);
pub const SHIFT_2: KeyboardShortcut = shift!(Key::Num2);
pub const SHIFT_3: KeyboardShortcut = shift!(Key::Num3);
pub const SHIFT_4: KeyboardShortcut = shift!(Key::Num4);
pub const SHIFT_5: KeyboardShortcut = shift!(Key::Num5);
pub const SHIFT_6: KeyboardShortcut = shift!(Key::Num6);
pub const SHIFT_7: KeyboardShortcut = shift!(Key::Num7);
pub const SHIFT_8: KeyboardShortcut = shift!(Key::Num8);
pub const SHIFT_9: KeyboardShortcut = shift!(Key::Num9);

pub const SHIFT_UP: KeyboardShortcut = shift!(Key::ArrowUp);
pub const SHIFT_DOWN: KeyboardShortcut = shift!(Key::ArrowDown);
pub const SHIFT_LEFT: KeyboardShortcut = shift!(Key::ArrowLeft);
pub const SHIFT_RIGHT: KeyboardShortcut = shift!(Key::ArrowRight);
pub const SHIFT_SPACE: KeyboardShortcut = shift!(Key::Space);

pub const CTRL_SHIFT_0: KeyboardShortcut = ctrl_shift!(Key::Num0);
pub const CTRL_SHIFT_1: KeyboardShortcut = ctrl_shift!(Key::Num1);
pub const CTRL_SHIFT_2: KeyboardShortcut = ctrl_shift!(Key::Num2);
pub const CTRL_SHIFT_3: KeyboardShortcut = ctrl_shift!(Key::Num3);
pub const CTRL_SHIFT_4: KeyboardShortcut = ctrl_shift!(Key::Num4);
pub const CTRL_SHIFT_5: KeyboardShortcut = ctrl_shift!(Key::Num5);
pub const CTRL_SHIFT_6: KeyboardShortcut = ctrl_shift!(Key::Num6);
pub const CTRL_SHIFT_7: KeyboardShortcut = ctrl_shift!(Key::Num7);
pub const CTRL_SHIFT_8: KeyboardShortcut = ctrl_shift!(Key::Num8);
pub const CTRL_SHIFT_9: KeyboardShortcut = ctrl_shift!(Key::Num9);
