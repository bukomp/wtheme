// WM_SETTINGCHANGE broadcast so listening apps repaint immediately after a theme write.

use anyhow::Result;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

// Single, short timeout for the broadcast — we don't want one hung window to stall the CLI.
const BROADCAST_TIMEOUT_MS: u32 = 1000;
// Null-terminated UTF-16 string passed as lParam to identify the changed setting.
const IMMERSIVE_COLOR_SET: &str = "ImmersiveColorSet\0";

// Broadcast WM_SETTINGCHANGE with lParam="ImmersiveColorSet" — same notification the
// Settings app sends. Listening apps (Explorer, Edge, modern UWP shells) repaint immediately.
pub fn broadcast_setting_change() -> Result<()> {
    // Encode the section name as a null-terminated wide string for lParam.
    let param: Vec<u16> = IMMERSIVE_COLOR_SET.encode_utf16().collect();
    // SendMessageTimeoutW writes each recipient's reply here; we ignore the value.
    let mut result: usize = 0;
    // Safety: `param` outlives the call; `result` is a valid &mut for the duration.
    unsafe {
        SendMessageTimeoutW(
            HWND(HWND_BROADCAST.0),                    // deliver to every top-level window
            WM_SETTINGCHANGE,                          // the message Windows uses for setting changes
            WPARAM(0),                                 // wParam is unused for this notification
            LPARAM(PCWSTR(param.as_ptr()).0 as isize), // lParam = pointer to "ImmersiveColorSet"
            SMTO_ABORTIFHUNG,                          // skip windows that are hung instead of blocking
            BROADCAST_TIMEOUT_MS,                      // per-window timeout
            Some(&mut result as *mut usize as *mut _), // out-param we don't consume
        );
    }
    Ok(())
}
