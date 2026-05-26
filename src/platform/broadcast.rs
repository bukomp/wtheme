use anyhow::Result;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

const BROADCAST_TIMEOUT_MS: u32 = 1000;
const IMMERSIVE_COLOR_SET: &str = "ImmersiveColorSet\0";

// Same notification the Settings app sends; listening apps (Explorer, Edge, UWP shells) repaint immediately.
pub fn broadcast_setting_change() -> Result<()> {
    let param: Vec<u16> = IMMERSIVE_COLOR_SET.encode_utf16().collect();
    let mut result: usize = 0;
    // Safety: `param` outlives the call; `result` is a valid &mut for the duration.
    unsafe {
        SendMessageTimeoutW(
            HWND(HWND_BROADCAST.0),
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(PCWSTR(param.as_ptr()).0 as isize),
            SMTO_ABORTIFHUNG,
            BROADCAST_TIMEOUT_MS,
            Some(&mut result as *mut usize as *mut _),
        );
    }
    Ok(())
}
