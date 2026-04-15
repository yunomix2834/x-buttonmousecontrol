use xbuttonmousecontrol_config_toml::TomlBindingRepository;
use xbuttonmousecontrol_core::BindingRuntime;
use xbuttonmousecontrol_platform_rdev::{EnigoKeyEmitter, RdevMouseEventSource};
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config/bindings.toml"));

    #[cfg(target_os = "linux")]
    warn_if_wayland();

    let repo = TomlBindingRepository::new(config_path);
    let source = RdevMouseEventSource::default();
    let emitter = EnigoKeyEmitter::new()?;

    let mut runtime = BindingRuntime::new(repo, source, emitter);
    runtime.run()?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn warn_if_wayland() {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        eprintln!(
            "[warning] Wayland detected. This starter is designed for Windows + Linux/X11. \
             Wayland should use a dedicated adapter."
        );
    }
}