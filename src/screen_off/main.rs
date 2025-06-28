#![windows_subsystem = "windows"]

use windows::{core::*, Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

fn main() {
    unsafe {
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("Button"),
            w!("Test"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            None,
        )
        .expect("Failed to create window");
        SendMessageW(hwnd, 0x0112, Some(WPARAM(0xF170)), Some(LPARAM(2)));
        let _ = DestroyWindow(hwnd);
    }
}
