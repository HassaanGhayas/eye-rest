use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_NOREPEAT, VK_ESCAPE,
};

pub const ESC_HOTKEY_ID: i32 = 0x5E5C; // Unique ID for Esc key

pub struct HotkeyManager;

impl HotkeyManager {
    /// Registers an ephemeral global Esc hotkey bound to the overlay window
    pub fn register_escape(hwnd: HWND) -> bool {
        unsafe {
            RegisterHotKey(
                hwnd,
                ESC_HOTKEY_ID,
                HOT_KEY_MODIFIERS(MOD_NOREPEAT.0),
                VK_ESCAPE.0 as u32,
            )
            .is_ok()
        }
    }

    /// Unregisters the ephemeral global Esc hotkey
    pub fn unregister_escape(hwnd: HWND) {
        unsafe {
            let _ = UnregisterHotKey(hwnd, ESC_HOTKEY_ID);
        }
    }
}
