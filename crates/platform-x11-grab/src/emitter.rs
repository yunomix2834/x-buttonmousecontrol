use crate::X11SyntheticFilter;
use enigo::{
    Button as EnigoButton,
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Mouse, Settings,
};
use xbuttonmousecontrol_core::{
    AppError, InputEvent, InputPhase, KeySpec, MouseButton, OutputEmitter, Trigger,
};

pub struct X11GrabEmitter {
    inner: Enigo,
    synthetic: X11SyntheticFilter,
}

impl X11GrabEmitter {
    pub fn new(synthetic: X11SyntheticFilter) -> Result<Self, AppError> {
        let inner = Enigo::new(&Settings::default())
            .map_err(|e| AppError::Port(format!("cannot init enigo: {e}")))?;
        Ok(Self { inner, synthetic })
    }

    fn record_key(&self, key: &KeySpec, phase: InputPhase) {
        self.synthetic.record(InputEvent {
            trigger: Trigger::Key(key.clone()),
            phase,
        });
    }

    fn record_mouse(&self, button: MouseButton, phase: InputPhase) {
        self.synthetic.record(InputEvent {
            trigger: Trigger::Mouse(button),
            phase,
        });
    }
}

impl OutputEmitter for X11GrabEmitter {
    fn key_press(&mut self, key: &KeySpec) -> Result<(), AppError> {
        self.record_key(key, InputPhase::Press);
        self.inner
            .key(parse_key(&key.0)?, Press)
            .map_err(|e| AppError::Port(format!("key_press failed: {e}")))
    }

    fn key_release(&mut self, key: &KeySpec) -> Result<(), AppError> {
        self.record_key(key, InputPhase::Release);
        self.inner
            .key(parse_key(&key.0)?, Release)
            .map_err(|e| AppError::Port(format!("key_release failed: {e}")))
    }

    fn key_tap(&mut self, key: &KeySpec) -> Result<(), AppError> {
        self.record_key(key, InputPhase::Press);
        self.record_key(key, InputPhase::Release);
        self.inner
            .key(parse_key(&key.0)?, Click)
            .map_err(|e| AppError::Port(format!("key_tap failed: {e}")))
    }

    fn mouse_press(&mut self, button: MouseButton) -> Result<(), AppError> {
        self.record_mouse(button, InputPhase::Press);
        self.inner
            .button(parse_mouse(button)?, Press)
            .map_err(|e| AppError::Port(format!("mouse_press failed: {e}")))
    }

    fn mouse_release(&mut self, button: MouseButton) -> Result<(), AppError> {
        self.record_mouse(button, InputPhase::Release);
        self.inner
            .button(parse_mouse(button)?, Release)
            .map_err(|e| AppError::Port(format!("mouse_release failed: {e}")))
    }

    fn mouse_click(&mut self, button: MouseButton) -> Result<(), AppError> {
        self.record_mouse(button, InputPhase::Press);
        self.record_mouse(button, InputPhase::Release);
        self.inner
            .button(parse_mouse(button)?, Click)
            .map_err(|e| AppError::Port(format!("mouse_click failed: {e}")))
    }
}

fn parse_mouse(button: MouseButton) -> Result<EnigoButton, AppError> {
    Ok(match button {
        MouseButton::Left => EnigoButton::Left,
        MouseButton::Right => EnigoButton::Right,
        MouseButton::Middle => EnigoButton::Middle,
        MouseButton::Back => EnigoButton::Back,
        MouseButton::Forward => EnigoButton::Forward,
        MouseButton::Unknown(code) => {
            return Err(AppError::Unsupported(format!("unsupported x11 mouse button '{code}'")))
        }
    })
}

fn parse_key(raw: &str) -> Result<Key, AppError> {
    let s = raw.trim().to_lowercase();

    if s.chars().count() == 1 {
        return Ok(Key::Unicode(s.chars().next().unwrap()));
    }

    let key = match s.as_str() {
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
        other => return Err(AppError::Unsupported(format!("unsupported x11 key '{other}'"))),
    };

    Ok(key)
}
