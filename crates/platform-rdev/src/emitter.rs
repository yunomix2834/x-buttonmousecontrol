use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use xbuttonmousecontrol_core::{AppError, KeyEmitter, KeySpec};

pub struct EnigoKeyEmitter {
    inner: Enigo,
}

impl EnigoKeyEmitter {
    pub fn new() -> Result<Self, AppError> {
        let inner = Enigo::new(&Settings::default())
            .map_err(|e| AppError::Port(format!("cannot init enigo: {e}")))?;
        Ok(Self { inner })
    }
}

impl KeyEmitter for EnigoKeyEmitter {
    fn press(&mut self, key: &KeySpec) -> Result<(), AppError> {
        let key = parse_key(&key.0)?;
        self.inner
            .key(key, Press)
            .map_err(|e| AppError::Port(format!("press failed: {e}")))
    }

    fn release(&mut self, key: &KeySpec) -> Result<(), AppError> {
        let key = parse_key(&key.0)?;
        self.inner
            .key(key, Release)
            .map_err(|e| AppError::Port(format!("release failed: {e}")))
    }

    fn tap(&mut self, key: &KeySpec) -> Result<(), AppError> {
        let key = parse_key(&key.0)?;
        self.inner
            .key(key, Click)
            .map_err(|e| AppError::Port(format!("tap failed: {e}")))
    }
}

fn parse_key(raw: &str) -> Result<Key, AppError> {
    let normalized = raw.trim().to_lowercase();

    if normalized.chars().count() == 1 {
        let ch = normalized.chars().next().unwrap();
        return Ok(Key::Unicode(ch));
    }

    let key = match normalized.as_str() {
        "space" => Key::Space,
        "enter" | "return" => Key::Return,
        "tab" => Key::Tab,
        "esc" | "escape" => Key::Escape,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "insert" => Key::Insert,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        "shift" => Key::Shift,
        "ctrl" | "control" => Key::Control,
        "alt" => Key::Alt,
        "meta" | "win" | "super" => Key::Meta,
        "capslock" => Key::CapsLock,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        "f13" => Key::F13,
        "f14" => Key::F14,
        "f15" => Key::F15,
        "f16" => Key::F16,
        "f17" => Key::F17,
        "f18" => Key::F18,
        "f19" => Key::F19,
        "f20" => Key::F20,
        "f21" => Key::F21,
        "f22" => Key::F22,
        "f23" => Key::F23,
        "f24" => Key::F24,
        other => {
            return Err(AppError::Parse(format!(
                "unsupported key '{other}'. Add it to parse_key()."
            )));
        }
    };

    Ok(key)
}