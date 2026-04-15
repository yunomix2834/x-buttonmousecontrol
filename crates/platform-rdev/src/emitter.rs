use enigo::{
    Button as EnigoButton,
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Mouse, Settings,
};
use xbuttonmousecontrol_core::{AppError, KeySpec, MouseButton, OutputEmitter};

pub struct EnigoOutputEmitter {
    inner: Enigo,
}

impl EnigoOutputEmitter {
    pub fn new() -> Result<Self, AppError> {
        let inner = Enigo::new(&Settings::default())
            .map_err(|e| AppError::Port(format!("cannot init enigo: {e}")))?;
        Ok(Self { inner })
    }
}

impl OutputEmitter for EnigoOutputEmitter {
    fn key_press(&mut self, key: &KeySpec) -> Result<(), AppError> {
        self.inner
            .key(parse_key(&key.0)?, Press)
            .map_err(|e| AppError::Port(format!("key_press failed: {e}")))
    }

    fn key_release(&mut self, key: &KeySpec) -> Result<(), AppError> {
        self.inner
            .key(parse_key(&key.0)?, Release)
            .map_err(|e| AppError::Port(format!("key_release failed: {e}")))
    }

    fn key_tap(&mut self, key: &KeySpec) -> Result<(), AppError> {
        self.inner
            .key(parse_key(&key.0)?, Click)
            .map_err(|e| AppError::Port(format!("key_tap failed: {e}")))
    }

    fn mouse_press(&mut self, button: MouseButton) -> Result<(), AppError> {
        self.inner
            .button(parse_mouse_button(button), Press)
            .map_err(|e| AppError::Port(format!("mouse_press failed: {e}")))
    }

    fn mouse_release(&mut self, button: MouseButton) -> Result<(), AppError> {
        self.inner
            .button(parse_mouse_button(button), Release)
            .map_err(|e| AppError::Port(format!("mouse_release failed: {e}")))
    }

    fn mouse_click(&mut self, button: MouseButton) -> Result<(), AppError> {
        self.inner
            .button(parse_mouse_button(button), Click)
            .map_err(|e| AppError::Port(format!("mouse_click failed: {e}")))
    }
}

fn parse_mouse_button(button: MouseButton) -> EnigoButton {
    match button {
        MouseButton::Left => EnigoButton::Left,
        MouseButton::Right => EnigoButton::Right,
        MouseButton::Middle => EnigoButton::Middle,
        MouseButton::Back => EnigoButton::Back,
        MouseButton::Forward => EnigoButton::Forward,
        MouseButton::Unknown(_) => EnigoButton::Left,
    }
}

fn parse_key(raw: &str) -> Result<Key, AppError> {
    let normalized = raw.trim().to_lowercase();

    if normalized.chars().count() == 1 {
        return Ok(Key::Unicode(normalized.chars().next().unwrap()));
    }

    let key = match normalized.as_str() {
        "space" => Key::Space,
        "enter" | "return" => Key::Return,
        "tab" => Key::Tab,
        "esc" | "escape" => Key::Escape,
        "ctrl" | "control" => Key::Control,
        "shift" => Key::Shift,
        "alt" => Key::Alt,
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
        other => {
            return Err(AppError::Parse(format!("unsupported key '{other}'")));
        }
    };

    Ok(key)
}