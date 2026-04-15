use serde::Deserialize;
use std::{fs, path::PathBuf};
use xbuttonmousecontrol_core::{
    AppError, Binding, BindingAction, BindingProfile, BindingRepository, KeySpec, MouseButton,
    Target, Trigger,
};

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
    source_type: String,
    source: String,
    target_type: String,
    target: String,
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
                trigger: parse_trigger(&item.source_type, &item.source)?,
                target: parse_target(&item.target_type, &item.target)?,
                action: parse_action(&item.action)?,
            });
        }

        Ok(BindingProfile { bindings })
    }
}

fn parse_trigger(source_type: &str, source: &str) -> Result<Trigger, AppError> {
    match source_type.trim().to_lowercase().as_str() {
        "mouse" => Ok(Trigger::Mouse(parse_mouse_button(source)?)),
        "key" => Ok(Trigger::Key(KeySpec(source.trim().to_string()))),
        other => Err(AppError::Parse(format!("unknown source_type '{other}'"))),
    }
}

fn parse_target(target_type: &str, target: &str) -> Result<Target, AppError> {
    match target_type.trim().to_lowercase().as_str() {
        "mouse" => Ok(Target::Mouse(parse_mouse_button(target)?)),
        "key" => Ok(Target::Key(KeySpec(target.trim().to_string()))),
        other => Err(AppError::Parse(format!("unknown target_type '{other}'"))),
    }
}

fn parse_mouse_button(value: &str) -> Result<MouseButton, AppError> {
    match value.trim().to_lowercase().as_str() {
        "left" => Ok(MouseButton::Left),
        "right" => Ok(MouseButton::Right),
        "middle" => Ok(MouseButton::Middle),
        "back" | "x1" | "mouse4" => Ok(MouseButton::Back),
        "forward" | "x2" | "mouse5" => Ok(MouseButton::Forward),
        other => Err(AppError::Parse(format!("unknown mouse button '{other}'"))),
    }
}

fn parse_action(value: &str) -> Result<BindingAction, AppError> {
    match value.trim().to_lowercase().as_str() {
        "hold" => Ok(BindingAction::Hold),
        "tap" => Ok(BindingAction::Tap),
        other => Err(AppError::Parse(format!("unknown action '{other}'"))),
    }
}