use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, TranslateMessage, WM_APP,
};

/// Thread message telling the tray whether the overlay runs; WPARAM is 0 or 1.
/// The menu lives on the tray thread and cannot be touched from any other.
const WM_TRAY_RUNNING: u32 = WM_APP + 1;

pub fn current_thread() -> u32 {
    unsafe { GetCurrentThreadId() }
}

/// False when the message could not be queued, for instance because the tray
/// thread is gone.
pub fn post_running(thread: u32, running: bool) -> bool {
    unsafe { PostThreadMessageW(thread, WM_TRAY_RUNNING, usize::from(running), 0) != 0 }
}

pub fn run(mut on_running: impl FnMut(bool)) {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            // Thread messages have no window to be dispatched to.
            if msg.hwnd.is_null() && msg.message == WM_TRAY_RUNNING {
                on_running(msg.wParam != 0);
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
