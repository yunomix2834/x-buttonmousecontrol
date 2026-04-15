use crate::{AppError, BindingProfile, InputEvent, KeySpec, MouseButton};
use std::sync::mpsc::Sender;

pub trait BindingRepository {
    fn load(&self) -> Result<BindingProfile, AppError>;
}

pub trait InputEventSource {
    fn spawn(&self, tx: Sender<InputEvent>) -> Result<(), AppError>;
}

pub trait OutputEmitter {
    fn key_press(&mut self, key: &KeySpec) -> Result<(), AppError>;
    fn key_release(&mut self, key: &KeySpec) -> Result<(), AppError>;
    fn key_tap(&mut self, key: &KeySpec) -> Result<(), AppError>;

    fn mouse_press(&mut self, button: MouseButton) -> Result<(), AppError>;
    fn mouse_release(&mut self, button: MouseButton) -> Result<(), AppError>;
    fn mouse_click(&mut self, button: MouseButton) -> Result<(), AppError>;
}