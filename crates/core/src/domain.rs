use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Unknown(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputPhase {
    Press,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeySpec(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingAction {
    /// Nhấn chuột xuống => nhấn phím
    /// Thả chuột => thả phím
    Hold,

    /// Chỉ tap 1 lần khi nhấn chuột xuống
    Tap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub mouse_button: MouseButton,
    pub key: KeySpec,
    pub action: BindingAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingProfile {
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MouseInputEvent {
    pub button: MouseButton,
    pub phase: InputPhase,
}