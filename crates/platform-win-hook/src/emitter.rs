use crate::SYNTHETIC_TAG;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    MAPVK_VK_TO_VSC, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_XDOWN,
    MOUSEEVENTF_XUP, MOUSEINPUT, MapVirtualKeyW, VIRTUAL_KEY, VK_CONTROL, VK_ESCAPE, VK_F1,
    VK_F10, VK_F11, VK_F12, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_MENU,
    VK_RETURN, VK_SHIFT, VK_SPACE, VK_TAB, XBUTTON1, XBUTTON2,
};
use xbuttonmousecontrol_core::{AppError, KeySpec, MouseButton, OutputEmitter};

#[derive(Default)]
pub struct WinHookEmitter;

impl WinHookEmitter {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self)
    }
}

impl OutputEmitter for WinHookEmitter {
    fn key_press(&mut self, key: &KeySpec) -> Result<(), AppError> {
        send_key(parse_vk(&key.0)?, false)
    }

    fn key_release(&mut self, key: &KeySpec) -> Result<(), AppError> {
        send_key(parse_vk(&key.0)?, true)
    }

    fn key_tap(&mut self, key: &KeySpec) -> Result<(), AppError> {
        let vk = parse_vk(&key.0)?;
        send_key(vk, false)?;
        send_key(vk, true)
    }

    fn mouse_press(&mut self, button: MouseButton) -> Result<(), AppError> {
        send_mouse(button, false)
    }

    fn mouse_release(&mut self, button: MouseButton) -> Result<(), AppError> {
        send_mouse(button, true)
    }

    fn mouse_click(&mut self, button: MouseButton) -> Result<(), AppError> {
        send_mouse(button, false)?;
        send_mouse(button, true)
    }
}

fn parse_vk(raw: &str) -> Result<VIRTUAL_KEY, AppError> {
    let s = raw.trim().to_lowercase();
    let vk = match s.as_str() {
        "a" => VIRTUAL_KEY(0x41), "b" => VIRTUAL_KEY(0x42), "c" => VIRTUAL_KEY(0x43),
        "d" => VIRTUAL_KEY(0x44), "e" => VIRTUAL_KEY(0x45), "f" => VIRTUAL_KEY(0x46),
        "g" => VIRTUAL_KEY(0x47), "h" => VIRTUAL_KEY(0x48), "i" => VIRTUAL_KEY(0x49),
        "j" => VIRTUAL_KEY(0x4A), "k" => VIRTUAL_KEY(0x4B), "l" => VIRTUAL_KEY(0x4C),
        "m" => VIRTUAL_KEY(0x4D), "n" => VIRTUAL_KEY(0x4E), "o" => VIRTUAL_KEY(0x4F),
        "p" => VIRTUAL_KEY(0x50), "q" => VIRTUAL_KEY(0x51), "r" => VIRTUAL_KEY(0x52),
        "s" => VIRTUAL_KEY(0x53), "t" => VIRTUAL_KEY(0x54), "u" => VIRTUAL_KEY(0x55),
        "v" => VIRTUAL_KEY(0x56), "w" => VIRTUAL_KEY(0x57), "x" => VIRTUAL_KEY(0x58),
        "y" => VIRTUAL_KEY(0x59), "z" => VIRTUAL_KEY(0x5A),

        "0" => VIRTUAL_KEY(0x30), "1" => VIRTUAL_KEY(0x31), "2" => VIRTUAL_KEY(0x32),
        "3" => VIRTUAL_KEY(0x33), "4" => VIRTUAL_KEY(0x34), "5" => VIRTUAL_KEY(0x35),
        "6" => VIRTUAL_KEY(0x36), "7" => VIRTUAL_KEY(0x37), "8" => VIRTUAL_KEY(0x38),
        "9" => VIRTUAL_KEY(0x39),

        "space" => VK_SPACE,
        "enter" | "return" => VK_RETURN,
        "tab" => VK_TAB,
        "esc" | "escape" => VK_ESCAPE,
        "ctrl" | "control" => VK_CONTROL,
        "shift" => VK_SHIFT,
        "alt" => VK_MENU,
        "f1" => VK_F1,
        "f2" => VK_F2,
        "f3" => VK_F3,
        "f4" => VK_F4,
        "f5" => VK_F5,
        "f6" => VK_F6,
        "f7" => VK_F7,
        "f8" => VK_F8,
        "f9" => VK_F9,
        "f10" => VK_F10,
        "f11" => VK_F11,
        "f12" => VK_F12,
        other => return Err(AppError::Unsupported(format!("unsupported windows key '{other}'"))),
    };

    Ok(vk)
}

fn send_key(vk: VIRTUAL_KEY, keyup: bool) -> Result<(), AppError> {
    let scan = unsafe { MapVirtualKeyW(vk.0 as u32, MAPVK_VK_TO_VSC) } as u16;
    let flags = if keyup { KEYEVENTF_KEYUP } else { Default::default() };

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: SYNTHETIC_TAG,
            },
        },
    };

    let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    if sent == 0 {
        return Err(AppError::Port("SendInput keyboard failed".to_string()));
    }
    Ok(())
}

fn send_mouse(button: MouseButton, release: bool) -> Result<(), AppError> {
    let (flags, data) = match (button, release) {
        (MouseButton::Left, false) => (MOUSEEVENTF_LEFTDOWN, 0),
        (MouseButton::Left, true) => (MOUSEEVENTF_LEFTUP, 0),
        (MouseButton::Right, false) => (MOUSEEVENTF_RIGHTDOWN, 0),
        (MouseButton::Right, true) => (MOUSEEVENTF_RIGHTUP, 0),
        (MouseButton::Middle, false) => (MOUSEEVENTF_MIDDLEDOWN, 0),
        (MouseButton::Middle, true) => (MOUSEEVENTF_MIDDLEUP, 0),
        (MouseButton::Back, false) => (MOUSEEVENTF_XDOWN, XBUTTON1 as u32),
        (MouseButton::Back, true) => (MOUSEEVENTF_XUP, XBUTTON1 as u32),
        (MouseButton::Forward, false) => (MOUSEEVENTF_XDOWN, XBUTTON2 as u32),
        (MouseButton::Forward, true) => (MOUSEEVENTF_XUP, XBUTTON2 as u32),
        (MouseButton::Unknown(code), _) => {
            return Err(AppError::Unsupported(format!("unsupported windows mouse button '{code}'")))
        }
    };

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: SYNTHETIC_TAG,
            },
        },
    };

    let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    if sent == 0 {
        return Err(AppError::Port("SendInput mouse failed".to_string()));
    }
    Ok(())
}