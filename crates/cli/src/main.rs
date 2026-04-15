use std::{env, path::PathBuf};

use xbuttonmousecontrol_config_toml::TomlBindingRepository;
use xbuttonmousecontrol_core::{BindingRepository, BindingRuntime, Trigger};
use xbuttonmousecontrol_platform_rdev::{EnigoOutputEmitter, RdevInputEventSource};
use xbuttonmousecontrol_platform_wayland_portal::build_wayland_backend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config/bindings.toml"));

    let repo = TomlBindingRepository::new(config_path.clone());

    #[cfg(target_os = "linux")]
    if std::env::var("XDG_SESSION_TYPE").ok().as_deref() == Some("wayland") {
        let profile = repo.load()?;

        let has_mouse_triggers = profile
            .bindings
            .iter()
            .any(|b| matches!(b.trigger, Trigger::Mouse(_)));

        if has_mouse_triggers {
            eprintln!(
                "[warning] Wayland adapter currently supports shortcut-driven bindings only \
                 (key -> mouse / key -> key). mouse -> key remains on X11 for now."
            );
        }

        let (source, emitter) = build_wayland_backend(&profile);
        let mut runtime = BindingRuntime::new(repo, source, emitter);
        runtime.run()?;
        return Ok(());
    }

    let repo = TomlBindingRepository::new(config_path);
    let source = RdevInputEventSource::default();
    let emitter = EnigoOutputEmitter::new()?;

    let mut runtime = BindingRuntime::new(repo, source, emitter);
    runtime.run()?;

    Ok(())
}