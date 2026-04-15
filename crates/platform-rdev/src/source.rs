use xbuttonmousecontrol_core::{AppError, InputPhase, MouseButton, MouseEventSource, MouseInputEvent};
use rdev::{listen, Button, Event, EventType};
use std::{sync::mpsc::Sender, thread};

#[derive(Debug, Default, Clone, Copy)]
pub struct RdevMouseEventSource;

impl MouseEventSource for RdevMouseEventSource {
    fn spawn(&self, tx: Sender<MouseInputEvent>) -> Result<(), AppError> {
        thread::Builder::new()
            .name("mouse-event-source".to_string())
            .spawn(move || {
                let callback = move |event: Event| {
                    let mapped = match event.event_type {
                        EventType::ButtonPress(button) => Some(MouseInputEvent {
                            button: map_button(button),
                            phase: InputPhase::Press,
                        }),
                        EventType::ButtonRelease(button) => Some(MouseInputEvent {
                            button: map_button(button),
                            phase: InputPhase::Release,
                        }),
                        _ => None,
                    };

                    if let Some(evt) = mapped {
                        let _ = tx.send(evt);
                    }
                };

                if let Err(err) = listen(callback) {
                    eprintln!("mouse listener error: {err:?}");
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