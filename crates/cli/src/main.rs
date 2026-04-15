use std::{env, path::PathBuf};

use xbuttonmousecontrol_config_toml::TomlBindingRepository;
use xbuttonmousecontrol_core::BindingRuntime;
use xbuttonmousecontrol_platform_rdev::{EnigoOutputEmitter, RdevInputEventSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config/bindings.toml"));

    #[cfg(target_os = "linux")]
    warn_if_wayland();

    let repo = TomlBindingRepository::new(config_path);
    let source = RdevInputEventSource::default();
    let emitter = EnigoOutputEmitter::new()?;

    let mut runtime = BindingRuntime::new(repo, source, emitter);
    runtime.run()?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn warn_if_wayland() {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        eprintln!(
            "[warning] Wayland detected. This project currently targets Linux/X11. \
             Keyboard -> mouse click may not work correctly on Wayland."
        );
    }
}