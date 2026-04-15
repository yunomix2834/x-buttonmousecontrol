use xbuttonmousecontrol_core::{
    AppError, Binding, BindingAction, BindingProfile, BindingRepository, KeySpec, MouseButton,
};
use serde::Deserialize;
use std::{fs, path::PathBuf};

pub struct TomlBindingRepository {
    path: PathBuf,
}

impl TomlBindingRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[derive(Debug, Deserialize)]
struct FileProfile {
    bindings: Vec<FileBinding>,
}

#[derive(Debug, Deserialize)]
struct FileBinding {
    mouse_button: String,
    key: String,
    action: String,
}

impl BindingRepository for TomlBindingRepository {
    fn load(&self) -> Result<BindingProfile, AppError> {
        let text = fs::read_to_string(&self.path)?;
        let raw: FileProfile =
            toml::from_str(&text).map_err(|e| AppError::Parse(e.to_string()))?;

        let mut bindings = Vec::with_capacity(raw.bindings.len());

        for item in raw.bindings {
            bindings.push(Binding {
                mouse_button: parse_mouse_button(&item.mouse_button)?,
                key: KeySpec(item.key),
                action: parse_action(&item.action)?,
            });
        }

        Ok(BindingProfile { bindings })
    }
}

fn parse_mouse_button(value: &str) -> Result<MouseButton, AppError> {
    match value.trim().to_lowercase().as_str() {
        "left" => Ok(MouseButton::Left),
        "right" => Ok(MouseButton::Right),
        "middle" => Ok(MouseButton::Middle),
        "back" | "x1" | "mouse4" => Ok(MouseButton::Back),
        "forward" | "x2" | "mouse5" => Ok(MouseButton::Forward),
        other => Err(AppError::Parse(format!(
            "unknown mouse_button '{other}'"
        ))),
    }
}

fn parse_action(value: &str) -> Result<BindingAction, AppError> {
    match value.trim().to_lowercase().as_str() {
        "hold" => Ok(BindingAction::Hold),
        "tap" => Ok(BindingAction::Tap),
        other => Err(AppError::Parse(format!("unknown action '{other}'"))),
    }
}