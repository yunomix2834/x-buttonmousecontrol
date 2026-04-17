use crate::X11SyntheticFilter;
use std::collections::HashSet;
use std::sync::Arc;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ButtonIndex, ChangeWindowAttributesAux, ConnectionExt as XprotoExt, EventMask, GrabMode,
    ModMask, Window,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use xbuttonmousecontrol_core::{
    AppError, BindingMode, BindingProfile, InputEvent, InputInterceptor, InputPhase,
    InterceptDecision, KeySpec, MouseButton, Trigger,
};

pub struct X11GrabInputInterceptor {
    profile: BindingProfile,
    synthetic: X11SyntheticFilter,
}

impl X11GrabInputInterceptor {
    pub fn new(profile: BindingProfile, synthetic: X11SyntheticFilter) -> Result<Self, AppError> {
        if profile
            .bindings
            .iter()
            .any(|binding| binding.mode != BindingMode::Replace)
        {
            return Err(AppError::Unsupported(
                "X11 backend currently supports only mode = replace. Use Windows for additive mode."
                    .to_string(),
            ));
        }

        Ok(Self { profile, synthetic })
    }
}

impl InputInterceptor for X11GrabInputInterceptor {
    fn run<H>(&self, mut handler: H) -> Result<(), AppError>
    where
        H: FnMut(InputEvent) -> Result<InterceptDecision, AppError> + Send + 'static,
    {
        let (conn, screen_num) = x11rb::connect(None)
            .map_err(|e| AppError::Port(format!("x11 connect failed: {e}")))?;
        let conn = Arc::new(conn);
        let root = conn.setup().roots[screen_num].root;

        conn.change_window_attributes(
            root,
            &ChangeWindowAttributesAux::new().event_mask(
                EventMask::KEY_PRESS
                    | EventMask::KEY_RELEASE
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE,
            ),
        )
            .map_err(|e| AppError::Port(format!("change_window_attributes failed: {e}")))?;

        self.install_grabs(&conn, root)?;
        conn.flush().map_err(|e| AppError::Port(format!("flush failed: {e}")))?;

        loop {
            let event = conn
                .wait_for_event()
                .map_err(|e| AppError::Port(format!("wait_for_event failed: {e}")))?;

            if let Some(input) = map_event(&conn, event)? {
                if self.synthetic.consume(&input) {
                    continue;
                }

                let _ = handler(input)?;
            }
        }
    }
}

impl X11GrabInputInterceptor {
    fn install_grabs(&self, conn: &RustConnection, root: Window) -> Result<(), AppError> {
        let mut keys = HashSet::new();
        let mut buttons = HashSet::new();

        for binding in &self.profile.bindings {
            match &binding.trigger {
                Trigger::Key(key) => {
                    if keys.insert(key.0.clone()) {
                        grab_key(conn, root, key)?;
                    }
                }
                Trigger::Mouse(button) => {
                    if buttons.insert(*button) {
                        grab_button(conn, root, *button)?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn grab_key(conn: &RustConnection, root: Window, key: &KeySpec) -> Result<(), AppError> {
    let keycode = key_to_keycode(conn, &key.0)?;
    for modifiers in modifier_variants() {
        conn.grab_key(false, root, modifiers, keycode, GrabMode::ASYNC, GrabMode::ASYNC)
            .map_err(|e| AppError::Port(format!("grab_key failed for {}: {e}", key.0)))?;
    }
    Ok(())
}

fn grab_button(conn: &RustConnection, root: Window, button: MouseButton) -> Result<(), AppError> {
    let btn = button_to_x11(button)?;
    for modifiers in modifier_variants() {
        conn.grab_button(
            false,
            root,
            EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
            x11rb::NONE,
            x11rb::NONE,
            ButtonIndex::from(btn),
            modifiers,
        )
            .map_err(|e| AppError::Port(format!("grab_button failed for {button:?}: {e}")))?;
    }
    Ok(())
}

fn map_event(conn: &RustConnection, event: Event) -> Result<Option<InputEvent>, AppError> {
    let mapped = match event {
        Event::KeyPress(ev) => Some(InputEvent {
            trigger: Trigger::Key(KeySpec(keycode_to_key(conn, ev.detail)?.to_string())),
            phase: InputPhase::Press,
        }),
        Event::KeyRelease(ev) => Some(InputEvent {
            trigger: Trigger::Key(KeySpec(keycode_to_key(conn, ev.detail)?.to_string())),
            phase: InputPhase::Release,
        }),
        Event::ButtonPress(ev) => Some(InputEvent {
            trigger: Trigger::Mouse(x11_to_button(ev.detail)?),
            phase: InputPhase::Press,
        }),
        Event::ButtonRelease(ev) => Some(InputEvent {
            trigger: Trigger::Mouse(x11_to_button(ev.detail)?),
            phase: InputPhase::Release,
        }),
        _ => None,
    };
    Ok(mapped)
}

fn modifier_variants() -> Vec<ModMask> {
    vec![
        ModMask::from(0u16),
        ModMask::LOCK,
        ModMask::M2,
        ModMask::LOCK | ModMask::M2,
    ]
}

fn button_to_x11(button: MouseButton) -> Result<u8, AppError> {
    Ok(match button {
        MouseButton::Left => 1,
        MouseButton::Middle => 2,
        MouseButton::Right => 3,
        MouseButton::Back => 8,
        MouseButton::Forward => 9,
        MouseButton::Unknown(code) => {
            return Err(AppError::Unsupported(format!("unsupported x11 mouse button '{code}'")))
        }
    })
}

fn x11_to_button(button: u8) -> Result<MouseButton, AppError> {
    Ok(match button {
        1 => MouseButton::Left,
        2 => MouseButton::Middle,
        3 => MouseButton::Right,
        8 => MouseButton::Back,
        9 => MouseButton::Forward,
        other => MouseButton::Unknown(other as u32),
    })
}

fn key_to_keycode(conn: &RustConnection, raw: &str) -> Result<u8, AppError> {
    let needle = raw.trim().to_lowercase();
    let setup = conn.setup();
    let min = setup.min_keycode;
    let max = setup.max_keycode;

    let reply = conn
        .get_keyboard_mapping(min, max - min + 1)
        .map_err(|e| AppError::Port(format!("get_keyboard_mapping failed: {e}")))?
        .reply()
        .map_err(|e| AppError::Port(format!("get_keyboard_mapping reply failed: {e}")))?;

    for (idx, syms) in reply.keysyms.chunks(reply.keysyms_per_keycode as usize).enumerate() {
        let keycode = min + idx as u8;
        let names = keysyms_to_names(syms);
        if names.iter().any(|n| n == &needle) {
            return Ok(keycode);
        }
    }

    Err(AppError::Unsupported(format!("unsupported x11 key '{needle}'")))
}

fn keycode_to_key(conn: &RustConnection, keycode: u8) -> Result<&'static str, AppError> {
    let reply = conn
        .get_keyboard_mapping(keycode, 1)
        .map_err(|e| AppError::Port(format!("get_keyboard_mapping failed: {e}")))?
        .reply()
        .map_err(|e| AppError::Port(format!("get_keyboard_mapping reply failed: {e}")))?;

    let names = keysyms_to_names(&reply.keysyms);
    names
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Unsupported(format!("unsupported x11 keycode '{keycode}'")))
}

fn keysyms_to_names(syms: &[u32]) -> Vec<&'static str> {
    let mut out = Vec::new();
    for sym in syms {
        match *sym {
            0x0020 => out.push("space"),
            0xff0d => out.push("enter"),
            0xff09 => out.push("tab"),
            0xff1b => out.push("esc"),
            0xffe3 | 0xffe4 => out.push("ctrl"),
            0xffe1 | 0xffe2 => out.push("shift"),
            0xffe9 | 0xffea => out.push("alt"),
            0xffbe => out.push("f1"),
            0xffbf => out.push("f2"),
            0xffc0 => out.push("f3"),
            0xffc1 => out.push("f4"),
            0xffc2 => out.push("f5"),
            0xffc3 => out.push("f6"),
            0xffc4 => out.push("f7"),
            0xffc5 => out.push("f8"),
            0xffc6 => out.push("f9"),
            0xffc7 => out.push("f10"),
            0xffc8 => out.push("f11"),
            0xffc9 => out.push("f12"),
            0x0030 => out.push("0"), 0x0031 => out.push("1"), 0x0032 => out.push("2"),
            0x0033 => out.push("3"), 0x0034 => out.push("4"), 0x0035 => out.push("5"),
            0x0036 => out.push("6"), 0x0037 => out.push("7"), 0x0038 => out.push("8"),
            0x0039 => out.push("9"),
            0x0041 | 0x0061 => out.push("a"), 0x0042 | 0x0062 => out.push("b"),
            0x0043 | 0x0063 => out.push("c"), 0x0044 | 0x0064 => out.push("d"),
            0x0045 | 0x0065 => out.push("e"), 0x0046 | 0x0066 => out.push("f"),
            0x0047 | 0x0067 => out.push("g"), 0x0048 | 0x0068 => out.push("h"),
            0x0049 | 0x0069 => out.push("i"), 0x004a | 0x006a => out.push("j"),
            0x004b | 0x006b => out.push("k"), 0x004c | 0x006c => out.push("l"),
            0x004d | 0x006d => out.push("m"), 0x004e | 0x006e => out.push("n"),
            0x004f | 0x006f => out.push("o"), 0x0050 | 0x0070 => out.push("p"),
            0x0051 | 0x0071 => out.push("q"), 0x0052 | 0x0072 => out.push("r"),
            0x0053 | 0x0073 => out.push("s"), 0x0054 | 0x0074 => out.push("t"),
            0x0055 | 0x0075 => out.push("u"), 0x0056 | 0x0076 => out.push("v"),
            0x0057 | 0x0077 => out.push("w"), 0x0058 | 0x0078 => out.push("x"),
            0x0059 | 0x0079 => out.push("y"), 0x005a | 0x007a => out.push("z"),
            _ => {}
        }
    }
    out
}
