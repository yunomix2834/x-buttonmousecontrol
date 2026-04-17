use crate::SYNTHETIC_TAG;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_CONTROL, VK_ESCAPE, VK_F1, VK_F10, VK_F11, VK_F12, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6,
    VK_F7, VK_F8, VK_F9, VK_MENU, VK_RETURN, VK_SHIFT, VK_SPACE, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, XBUTTON1, XBUTTON2, HC_ACTION, HHOOK, MSG,
    WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
};
use xbuttonmousecontrol_core::{
    AppError, InputEvent, InputInterceptor, InputPhase, InterceptDecision, KeySpec, MouseButton,
    Trigger,
};

static STATE: OnceCell<Arc<HookState>> = OnceCell::new();

pub struct WinHookInputInterceptor;

impl WinHookInputInterceptor {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self)
    }
}

impl InputInterceptor for WinHookInputInterceptor {
    fn run<H>(&self, handler: H) -> Result<(), AppError>
    where
        H: FnMut(InputEvent) -> Result<InterceptDecision, AppError> + Send + 'static,
    {
        let state = Arc::new(HookState::new(handler));
        let _ = STATE.set(state);

        let module: HINSTANCE = unsafe { GetModuleHandleW(None) }
            .map_err(|e| AppError::Port(format!("GetModuleHandleW failed: {e}")))?
            .into();

        let keyboard = unsafe {
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), Some(module), 0)
        }
            .map_err(|e| AppError::Port(format!("SetWindowsHookExW keyboard failed: {e}")))?;

        let mouse = unsafe {
            SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), Some(module), 0)
        }
            .map_err(|e| AppError::Port(format!("SetWindowsHookExW mouse failed: {e}")))?;

        let result = message_loop();

        unsafe {
            let _ = UnhookWindowsHookEx(keyboard);
            let _ = UnhookWindowsHookEx(mouse);
        }

        result
    }
}

struct HookState {
    handler: Mutex<Box<dyn FnMut(InputEvent) -> Result<InterceptDecision, AppError> + Send>>,
}

impl HookState {
    fn new<H>(handler: H) -> Self
    where
        H: FnMut(InputEvent) -> Result<InterceptDecision, AppError> + Send + 'static,
    {
        Self {
            handler: Mutex::new(Box::new(handler)),
        }
    }

    fn dispatch(&self, event: InputEvent) -> InterceptDecision {
        let mut handler = self.handler.lock().unwrap();
        match handler(event) {
            Ok(decision) => decision,
            Err(err) => {
                eprintln!("handler error: {err}");
                InterceptDecision::PassThrough
            }
        }
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < HC_ACTION as i32 {
        return CallNextHookEx(Some(HHOOK::default()), code, wparam, lparam);
    }

    let hook = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
    if hook.dwExtraInfo == SYNTHETIC_TAG {
        return CallNextHookEx(Some(HHOOK::default()), code, wparam, lparam);
    }

    let phase = match wparam.0 as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => Some(InputPhase::Press),
        WM_KEYUP | WM_SYSKEYUP => Some(InputPhase::Release),
        _ => None,
    };

    if let Some(phase) = phase {
        if let Some(key) = map_vk(hook.vkCode) {
            if let Some(state) = STATE.get() {
                let decision = state.dispatch(InputEvent {
                    trigger: Trigger::Key(KeySpec(key.to_string())),
                    phase,
                });
                if decision == InterceptDecision::Suppress {
                    return LRESULT(1);
                }
            }
        }
    }

    CallNextHookEx(Some(HHOOK::default()), code, wparam, lparam)
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < HC_ACTION as i32 {
        return CallNextHookEx(Some(HHOOK::default()), code, wparam, lparam);
    }

    let hook = &*(lparam.0 as *const MSLLHOOKSTRUCT);
    if hook.dwExtraInfo == SYNTHETIC_TAG {
        return CallNextHookEx(Some(HHOOK::default()), code, wparam, lparam);
    }

    let mapped = match wparam.0 as u32 {
        WM_LBUTTONDOWN => Some((MouseButton::Left, InputPhase::Press)),
        WM_LBUTTONUP => Some((MouseButton::Left, InputPhase::Release)),
        WM_RBUTTONDOWN => Some((MouseButton::Right, InputPhase::Press)),
        WM_RBUTTONUP => Some((MouseButton::Right, InputPhase::Release)),
        WM_MBUTTONDOWN => Some((MouseButton::Middle, InputPhase::Press)),
        WM_MBUTTONUP => Some((MouseButton::Middle, InputPhase::Release)),
        WM_XBUTTONDOWN => map_xbutton(hook.mouseData).map(|btn| (btn, InputPhase::Press)),
        WM_XBUTTONUP => map_xbutton(hook.mouseData).map(|btn| (btn, InputPhase::Release)),
        WM_MOUSEMOVE | WM_MOUSEWHEEL => None,
        _ => None,
    };

    if let Some((button, phase)) = mapped {
        if let Some(state) = STATE.get() {
            let decision = state.dispatch(InputEvent {
                trigger: Trigger::Mouse(button),
                phase,
            });
            if decision == InterceptDecision::Suppress {
                return LRESULT(1);
            }
        }
    }

    CallNextHookEx(Some(HHOOK::default()), code, wparam, lparam)
}

fn message_loop() -> Result<(), AppError> {
    let mut msg = MSG::default();
    loop {
        let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        let value = ret.0;
        if value == -1 {
            return Err(AppError::Port("GetMessageW failed".to_string()));
        }
        if value == 0 {
            break;
        }
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

fn map_xbutton(mouse_data: u32) -> Option<MouseButton> {
    let hi = ((mouse_data >> 16) & 0xffff) as u16;
    if hi == XBUTTON1 {
        MouseButton::Back
    } else if hi == XBUTTON2 {
        MouseButton::Forward
    } else {
        MouseButton::Unknown(hi as u32)
    }
}

fn map_vk(vk: u32) -> Option<&'static str> {
    match vk {
        0x41 => Some("a"), 0x42 => Some("b"), 0x43 => Some("c"), 0x44 => Some("d"),
        0x45 => Some("e"), 0x46 => Some("f"), 0x47 => Some("g"), 0x48 => Some("h"),
        0x49 => Some("i"), 0x4A => Some("j"), 0x4B => Some("k"), 0x4C => Some("l"),
        0x4D => Some("m"), 0x4E => Some("n"), 0x4F => Some("o"), 0x50 => Some("p"),
        0x51 => Some("q"), 0x52 => Some("r"), 0x53 => Some("s"), 0x54 => Some("t"),
        0x55 => Some("u"), 0x56 => Some("v"), 0x57 => Some("w"), 0x58 => Some("x"),
        0x59 => Some("y"), 0x5A => Some("z"),
        0x30 => Some("0"), 0x31 => Some("1"), 0x32 => Some("2"), 0x33 => Some("3"),
        0x34 => Some("4"), 0x35 => Some("5"), 0x36 => Some("6"), 0x37 => Some("7"),
        0x38 => Some("8"), 0x39 => Some("9"),
        x if x == VK_SPACE.0 as u32 => Some("space"),
        x if x == VK_RETURN.0 as u32 => Some("enter"),
        x if x == VK_TAB.0 as u32 => Some("tab"),
        x if x == VK_ESCAPE.0 as u32 => Some("esc"),
        x if x == VK_CONTROL.0 as u32 => Some("ctrl"),
        x if x == VK_SHIFT.0 as u32 => Some("shift"),
        x if x == VK_MENU.0 as u32 => Some("alt"),
        x if x == VK_F1.0 as u32 => Some("f1"),
        x if x == VK_F2.0 as u32 => Some("f2"),
        x if x == VK_F3.0 as u32 => Some("f3"),
        x if x == VK_F4.0 as u32 => Some("f4"),
        x if x == VK_F5.0 as u32 => Some("f5"),
        x if x == VK_F6.0 as u32 => Some("f6"),
        x if x == VK_F7.0 as u32 => Some("f7"),
        x if x == VK_F8.0 as u32 => Some("f8"),
        x if x == VK_F9.0 as u32 => Some("f9"),
        x if x == VK_F10.0 as u32 => Some("f10"),
        x if x == VK_F11.0 as u32 => Some("f11"),
        x if x == VK_F12.0 as u32 => Some("f12"),
        _ => None,
    }
}
