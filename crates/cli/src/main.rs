use std::{
    env,
    error::Error,
    path::{PathBuf},
};

use xbuttonmousecontrol_config_toml::TomlBindingRepository;
use xbuttonmousecontrol_core::{BindingRepository, BindingRuntime, Trigger};
use xbuttonmousecontrol_platform_rdev::{EnigoOutputEmitter, RdevInputEventSource};
use xbuttonmousecontrol_platform_wayland_portal::build_wayland_backend;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arg_config = env::args().nth(1);
    let config_path = resolve_config_path(arg_config.as_deref())?;

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

fn resolve_config_path(arg: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(arg) = arg {
        candidates.push(PathBuf::from(arg));
    }

    candidates.push(PathBuf::from("config").join("bindings.toml"));
    candidates.push(PathBuf::from("bindings.toml"));

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("config").join("bindings.toml"));
            candidates.push(exe_dir.join("bindings.toml"));
        }
    }

    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(
        "Cannot find bindings.toml. Tried: \
         <arg>, ./config/bindings.toml, ./bindings.toml, \
         <exe_dir>/config/bindings.toml, <exe_dir>/bindings.toml"
            .into(),
    )
}