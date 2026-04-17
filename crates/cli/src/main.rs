use std::{env, error::Error, path::PathBuf};

use xbuttonmousecontrol_config_toml::TomlBindingRepository;
use xbuttonmousecontrol_core::{BindingRepository, BindingRuntime};

#[cfg(target_os = "windows")]
use xbuttonmousecontrol_platform_win_hook::{WinHookEmitter, WinHookInputInterceptor};

#[cfg(target_os = "linux")]
use xbuttonmousecontrol_platform_x11_grab::{
    X11GrabEmitter, X11GrabInputInterceptor, X11SyntheticFilter,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let config_path = resolve_config_path(env::args().nth(1).as_deref())?;
    let repo = TomlBindingRepository::new(config_path);
    let profile = repo.load()?;

    #[cfg(target_os = "windows")]
    {
        let source = WinHookInputInterceptor::new()?;
        let emitter = WinHookEmitter::new()?;
        let runtime = BindingRuntime::new(repo, source, emitter);
        runtime.run()?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let synthetic = X11SyntheticFilter::default();
        let source = X11GrabInputInterceptor::new(profile.clone(), synthetic.clone())?;
        let emitter = X11GrabEmitter::new(synthetic)?;
        let runtime = BindingRuntime::new(repo, source, emitter);
        runtime.run()?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("unsupported platform".into())
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
        "Cannot find bindings.toml. Tried: <arg>, ./config/bindings.toml, ./bindings.toml, <exe_dir>/config/bindings.toml, <exe_dir>/bindings.toml"
            .into(),
    )
}
