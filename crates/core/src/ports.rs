use crate::{AppError, BindingProfile, KeySpec, MouseInputEvent};
use std::sync::mpsc::Sender;

pub trait BindingRepository {
    fn load(&self) -> Result<BindingProfile, AppError>;
}

pub trait MouseEventSource {
    /// Adapter sẽ tự spawn thread riêng và bơm event vào channel.
    fn spawn(&self, tx: Sender<MouseInputEvent>) -> Result<(), AppError>;
}

pub trait KeyEmitter {
    fn press(&mut self, key: &KeySpec) -> Result<(), AppError>;
    fn release(&mut self, key: &KeySpec) -> Result<(), AppError>;
    fn tap(&mut self, key: &KeySpec) -> Result<(), AppError>;
}