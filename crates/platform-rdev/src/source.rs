use rdev::{listen, Button, Event, EventType, Key};
use std::{sync::mpsc::Sender, thread};
use xbuttonmousecontrol_core::{
    AppError, InputEvent, InputEventSource, InputPhase, KeySpec, MouseButton, Trigger,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct RdevInputEventSource;

impl InputEventSource for RdevInputEventSource {
    fn spawn(&self, tx: Sender<InputEvent>) -> Result<(), AppError> {
        thread::Builder::new()
            .name("global-input-source".to_string())
            .spawn(move || {
                let callback = move |event: Event| {
                    let mapped = match event.event_type {
                        EventType::ButtonPress(button) => Some(InputEvent {
                            trigger: Trigger::Mouse(map_button(button)),
                            phase: InputPhase::Press,
                        }),
                        EventType::ButtonRelease(button) => Some(InputEvent {
                            trigger: Trigger::Mouse(map_button(button)),
                            phase: InputPhase::Release,
                        }),
                        EventType::KeyPress(key) => map_key(key).map(|ks| InputEvent {
                            trigger: Trigger::Key(ks),
                            phase: InputPhase::Press,
                        }),
                        EventType::KeyRelease(key) => map_key(key).map(|ks| InputEvent {
                            trigger: Trigger::Key(ks),
                            phase: InputPhase::Release,
                        }),
                        _ => None,
                    };

                    if let Some(evt) = mapped {
                        let _ = tx.send(evt);
                    }
                };

                if let Err(err) = listen(callback) {
                    eprintln!("global input listener error: {err:?}");
                }
            })
            .map_err(|e| AppError::Thread(e.to_string()))?;

        Ok(())
    }
}

fn map_button(button: Button) -> MouseButton {
    match button {
        Button::Left => MouseButton::Left,
        Button::Right => MouseButton::Right,
        Button::Middle => MouseButton::Middle,
        Button::Unknown(1) => MouseButton::Back,
        Button::Unknown(2) => MouseButton::Forward,
        Button::Unknown(code) => MouseButton::Unknown(code as u32),
    }
}

fn map_key(key: Key) -> Option<KeySpec> {
    let s = match key {
        Key::F1 => "f1",
        Key::F2 => "f2",
        Key::F3 => "f3",
        Key::F4 => "f4",
        Key::F5 => "f5",
        Key::F6 => "f6",
        Key::F7 => "f7",
        Key::F8 => "f8",
        Key::F9 => "f9",
        Key::F10 => "f10",
        Key::F11 => "f11",
        Key::F12 => "f12",

        Key::KeyA => "a",
        Key::KeyB => "b",
        Key::KeyC => "c",
        Key::KeyD => "d",
        Key::KeyE => "e",
        Key::KeyF => "f",
        Key::KeyG => "g",
        Key::KeyH => "h",
        Key::KeyI => "i",
        Key::KeyJ => "j",
        Key::KeyK => "k",
        Key::KeyL => "l",
        Key::KeyM => "m",
        Key::KeyN => "n",
        Key::KeyO => "o",
        Key::KeyP => "p",
        Key::KeyQ => "q",
        Key::KeyR => "r",
        Key::KeyS => "s",
        Key::KeyT => "t",
        Key::KeyU => "u",
        Key::KeyV => "v",
        Key::KeyW => "w",
        Key::KeyX => "x",
        Key::KeyY => "y",
        Key::KeyZ => "z",

        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::Num0 => "0",

        Key::Space => "space",
        Key::Return => "enter",
        Key::Tab => "tab",
        Key::Escape => "esc",
        Key::ShiftLeft | Key::ShiftRight => "shift",
        Key::ControlLeft | Key::ControlRight => "ctrl",
        Key::Alt | Key::AltGr => "alt",

        _ => return None,
    };

    Some(KeySpec(s.to_string()))
}