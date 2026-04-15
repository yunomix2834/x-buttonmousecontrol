use ashpd::desktop::{
    global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut},
    remote_desktop::{
        DeviceType, KeyState, NotifyKeyboardKeycodeOptions, NotifyPointerButtonOptions,
        RemoteDesktop, SelectDevicesOptions, StartOptions,
    },
};
use futures_util::StreamExt;
use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc::Sender, Arc, OnceLock},
    thread,
};
use tokio::{runtime::Runtime, sync::mpsc as tokio_mpsc};
use xbuttonmousecontrol_core::{
    AppError, BindingProfile, InputEvent, InputEventSource, InputPhase, KeySpec, MouseButton,
    OutputEmitter, Trigger,
};

#[derive(Clone)]
struct Shared {
    shortcuts: Vec<KeySpec>,
    cmd_tx: Arc<OnceLock<tokio_mpsc::UnboundedSender<EmitCmd>>>,
}

pub struct WaylandPortalSource {
    shared: Shared,
}

pub struct WaylandPortalEmitter {
    shared: Shared,
}

enum EmitCmd {
    KeyPress(KeySpec),
    KeyRelease(KeySpec),
    KeyTap(KeySpec),
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    MouseClick(MouseButton),
}

pub fn build_wayland_backend(profile: &BindingProfile) -> (WaylandPortalSource, WaylandPortalEmitter) {
    let shortcuts = collect_wayland_shortcuts(profile);

    let shared = Shared {
        shortcuts,
        cmd_tx: Arc::new(OnceLock::new()),
    };

    (
        WaylandPortalSource {
            shared: shared.clone(),
        },
        WaylandPortalEmitter { shared },
    )
}

impl InputEventSource for WaylandPortalSource {
    fn spawn(&self, tx: Sender<InputEvent>) -> Result<(), AppError> {
        let shared = self.shared.clone();

        thread::Builder::new()
            .name("wayland-portal-source".to_string())
            .spawn(move || {
                let rt = match Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        eprintln!("failed to create tokio runtime: {e}");
                        return;
                    }
                };

                if let Err(e) = rt.block_on(run_wayland_loop(shared, tx)) {
                    eprintln!("wayland portal loop error: {e}");
                }
            })
            .map_err(|e| AppError::Thread(e.to_string()))?;

        Ok(())
    }
}

impl OutputEmitter for WaylandPortalEmitter {
    fn key_press(&mut self, key: &KeySpec) -> Result<(), AppError> {
        self.send_cmd(EmitCmd::KeyPress(key.clone()))
    }

    fn key_release(&mut self, key: &KeySpec) -> Result<(), AppError> {
        self.send_cmd(EmitCmd::KeyRelease(key.clone()))
    }

    fn key_tap(&mut self, key: &KeySpec) -> Result<(), AppError> {
        self.send_cmd(EmitCmd::KeyTap(key.clone()))
    }

    fn mouse_press(&mut self, button: MouseButton) -> Result<(), AppError> {
        self.send_cmd(EmitCmd::MousePress(button))
    }

    fn mouse_release(&mut self, button: MouseButton) -> Result<(), AppError> {
        self.send_cmd(EmitCmd::MouseRelease(button))
    }

    fn mouse_click(&mut self, button: MouseButton) -> Result<(), AppError> {
        self.send_cmd(EmitCmd::MouseClick(button))
    }
}

impl WaylandPortalEmitter {
    fn send_cmd(&self, cmd: EmitCmd) -> Result<(), AppError> {
        let tx = self
            .shared
            .cmd_tx
            .get()
            .ok_or_else(|| AppError::Port("wayland portal backend is not ready yet".to_string()))?;

        tx.send(cmd)
            .map_err(|_| AppError::Port("failed to send command to wayland portal loop".to_string()))
    }
}

async fn run_wayland_loop(shared: Shared, tx: Sender<InputEvent>) -> Result<(), AppError> {
    let global_shortcuts = GlobalShortcuts::new()
        .await
        .map_err(|e| AppError::Port(format!("GlobalShortcuts::new failed: {e}")))?;

    let remote_desktop = RemoteDesktop::new()
        .await
        .map_err(|e| AppError::Port(format!("RemoteDesktop::new failed: {e}")))?;

    // 1) Bind global shortcuts from config
    let gs_session = global_shortcuts
        .create_session(Default::default())
        .await
        .map_err(|e| AppError::Port(format!("GlobalShortcuts::create_session failed: {e}")))?;

    let shortcuts: Vec<NewShortcut> = shared
        .shortcuts
        .iter()
        .map(|key| {
            let preferred = portal_trigger_string(&key.0);

            NewShortcut::new(
                shortcut_id_for_key(key),
                format!("xbuttonmousecontrol trigger {}", key.0),
            )
                .preferred_trigger(Some(preferred.as_str()))
        })
        .collect();

    global_shortcuts
        .bind_shortcuts(&gs_session, &shortcuts, None, BindShortcutsOptions::default())
        .await
        .map_err(|e| AppError::Port(format!("bind_shortcuts request failed: {e}")))?
        .response()
        .map_err(|e| AppError::Port(format!("bind_shortcuts response failed: {e}")))?;

    // 2) Start RemoteDesktop session for input injection
    let rd_session = remote_desktop
        .create_session(Default::default())
        .await
        .map_err(|e| AppError::Port(format!("RemoteDesktop::create_session failed: {e}")))?;

    remote_desktop
        .select_devices(
            &rd_session,
            SelectDevicesOptions::default()
                .set_devices(DeviceType::Keyboard | DeviceType::Pointer),
        )
        .await
        .map_err(|e| AppError::Port(format!("select_devices request failed: {e}")))?
        .response()
        .map_err(|e| AppError::Port(format!("select_devices response failed: {e}")))?;

    remote_desktop
        .start(&rd_session, None, StartOptions::default())
        .await
        .map_err(|e| AppError::Port(format!("RemoteDesktop::start request failed: {e}")))?
        .response()
        .map_err(|e| AppError::Port(format!("RemoteDesktop::start response failed: {e}")))?;

    let (cmd_tx, mut cmd_rx) = tokio_mpsc::unbounded_channel::<EmitCmd>();
    let _ = shared.cmd_tx.set(cmd_tx);

    let id_to_key: HashMap<String, KeySpec> = shared
        .shortcuts
        .iter()
        .map(|k| (shortcut_id_for_key(k), k.clone()))
        .collect();

    let mut activated = global_shortcuts
        .receive_activated()
        .await
        .map_err(|e| AppError::Port(format!("receive_activated failed: {e}")))?;

    let mut deactivated = global_shortcuts
        .receive_deactivated()
        .await
        .map_err(|e| AppError::Port(format!("receive_deactivated failed: {e}")))?;

    loop {
        tokio::select! {
            Some(sig) = activated.next() => {
                if let Some(key) = id_to_key.get(sig.shortcut_id()) {
                    let _ = tx.send(InputEvent {
                        trigger: Trigger::Key(key.clone()),
                        phase: InputPhase::Press,
                    });
                }
            }

            Some(sig) = deactivated.next() => {
                if let Some(key) = id_to_key.get(sig.shortcut_id()) {
                    let _ = tx.send(InputEvent {
                        trigger: Trigger::Key(key.clone()),
                        phase: InputPhase::Release,
                    });
                }
            }

            Some(cmd) = cmd_rx.recv() => {
                handle_emit_cmd(&remote_desktop, &rd_session, cmd).await?;
            }
        }
    }
}

async fn handle_emit_cmd(
    remote_desktop: &RemoteDesktop,
    session: &ashpd::desktop::Session<RemoteDesktop>,
    cmd: EmitCmd,
) -> Result<(), AppError> {
    match cmd {
        EmitCmd::MouseClick(btn) => {
            let code = mouse_button_to_evdev(btn)?;
            remote_desktop
                .notify_pointer_button(session, code, KeyState::Pressed, NotifyPointerButtonOptions::default())
                .await
                .map_err(|e| AppError::Port(format!("mouse press failed: {e}")))?;
            remote_desktop
                .notify_pointer_button(session, code, KeyState::Released, NotifyPointerButtonOptions::default())
                .await
                .map_err(|e| AppError::Port(format!("mouse release failed: {e}")))?;
        }

        EmitCmd::MousePress(btn) => {
            let code = mouse_button_to_evdev(btn)?;
            remote_desktop
                .notify_pointer_button(session, code, KeyState::Pressed, NotifyPointerButtonOptions::default())
                .await
                .map_err(|e| AppError::Port(format!("mouse press failed: {e}")))?;
        }

        EmitCmd::MouseRelease(btn) => {
            let code = mouse_button_to_evdev(btn)?;
            remote_desktop
                .notify_pointer_button(session, code, KeyState::Released, NotifyPointerButtonOptions::default())
                .await
                .map_err(|e| AppError::Port(format!("mouse release failed: {e}")))?;
        }

        EmitCmd::KeyTap(key) => {
            let code = key_to_evdev(&key.0)?;
            remote_desktop
                .notify_keyboard_keycode(session, code, KeyState::Pressed, NotifyKeyboardKeycodeOptions::default())
                .await
                .map_err(|e| AppError::Port(format!("key press failed: {e}")))?;
            remote_desktop
                .notify_keyboard_keycode(session, code, KeyState::Released, NotifyKeyboardKeycodeOptions::default())
                .await
                .map_err(|e| AppError::Port(format!("key release failed: {e}")))?;
        }

        EmitCmd::KeyPress(key) => {
            let code = key_to_evdev(&key.0)?;
            remote_desktop
                .notify_keyboard_keycode(session, code, KeyState::Pressed, NotifyKeyboardKeycodeOptions::default())
                .await
                .map_err(|e| AppError::Port(format!("key press failed: {e}")))?;
        }

        EmitCmd::KeyRelease(key) => {
            let code = key_to_evdev(&key.0)?;
            remote_desktop
                .notify_keyboard_keycode(session, code, KeyState::Released, NotifyKeyboardKeycodeOptions::default())
                .await
                .map_err(|e| AppError::Port(format!("key release failed: {e}")))?;
        }
    }

    Ok(())
}

fn collect_wayland_shortcuts(profile: &BindingProfile) -> Vec<KeySpec> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for binding in &profile.bindings {
        if let Trigger::Key(key) = &binding.trigger {
            if seen.insert(key.0.clone()) {
                out.push(key.clone());
            }
        }
    }

    out
}

fn shortcut_id_for_key(key: &KeySpec) -> String {
    format!("trigger-{}", key.0.to_lowercase())
}

fn portal_trigger_string(key: &str) -> String {
    let k = key.trim().to_lowercase();
    if let Some(rest) = k.strip_prefix('f') {
        if rest.parse::<u8>().is_ok() {
            return format!("F{}", rest);
        }
    }
    if k.len() == 1 {
        return k.to_uppercase();
    }
    match k.as_str() {
        "ctrl" => "Ctrl".to_string(),
        "alt" => "Alt".to_string(),
        "shift" => "Shift".to_string(),
        "space" => "space".to_string(),
        "enter" => "Return".to_string(),
        _ => k,
    }
}

fn mouse_button_to_evdev(button: MouseButton) -> Result<i32, AppError> {
    match button {
        MouseButton::Left => Ok(272),    // BTN_LEFT
        MouseButton::Right => Ok(273),   // BTN_RIGHT
        MouseButton::Middle => Ok(274),  // BTN_MIDDLE
        MouseButton::Back => Ok(275),    // BTN_SIDE
        MouseButton::Forward => Ok(276), // BTN_EXTRA
        MouseButton::Unknown(code) => Err(AppError::Unsupported(format!(
            "unsupported mouse button for wayland portal: {code}"
        ))),
    }
}

fn key_to_evdev(key: &str) -> Result<i32, AppError> {
    let k = key.trim().to_lowercase();
    let code = match k.as_str() {
        "a" => 30, "b" => 48, "c" => 46, "d" => 32, "e" => 18, "f" => 33,
        "g" => 34, "h" => 35, "i" => 23, "j" => 36, "k" => 37, "l" => 38,
        "m" => 50, "n" => 49, "o" => 24, "p" => 25, "q" => 16, "r" => 19,
        "s" => 31, "t" => 20, "u" => 22, "v" => 47, "w" => 17, "x" => 45,
        "y" => 21, "z" => 44,

        "1" => 2, "2" => 3, "3" => 4, "4" => 5, "5" => 6,
        "6" => 7, "7" => 8, "8" => 9, "9" => 10, "0" => 11,

        "enter" => 28,
        "esc" => 1,
        "tab" => 15,
        "space" => 57,
        "ctrl" => 29,
        "shift" => 42,
        "alt" => 56,

        "f1" => 59, "f2" => 60, "f3" => 61, "f4" => 62,
        "f5" => 63, "f6" => 64, "f7" => 65, "f8" => 66,
        "f9" => 67, "f10" => 68, "f11" => 87, "f12" => 88,

        _ => {
            return Err(AppError::Unsupported(format!(
                "unsupported key for wayland portal emitter: {k}"
            )));
        }
    };

    Ok(code)
}