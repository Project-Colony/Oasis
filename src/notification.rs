use anyhow::{Context, Result};

#[cfg(target_os = "linux")]
pub fn send_notification(title: &str, body: &str) -> Result<()> {
    notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show()
        .context("Notification Linux (notify-rust) échouée")?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn send_notification(title: &str, body: &str) -> Result<()> {
    if let Err(error) = send_windows_toast(title, body) {
        eprintln!("Échec notification Windows, fallback MessageBox: {error:#}");
        show_windows_message_box(title, body);
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn send_windows_toast(title: &str, body: &str) -> Result<()> {
    winrt_notification::Toast::new(winrt_notification::Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .show()
        .context("Notification Windows (winrt-notification) échouée")
}

#[cfg(target_os = "windows")]
fn show_windows_message_box(title: &str, body: &str) {
    use std::ffi::OsStr;
    use std::iter;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK, MessageBoxW};

    let title_wide: Vec<u16> = OsStr::new(title)
        .encode_wide()
        .chain(iter::once(0))
        .collect();
    let body_wide: Vec<u16> = OsStr::new(body)
        .encode_wide()
        .chain(iter::once(0))
        .collect();

    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            body_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

#[cfg(target_os = "macos")]
pub fn send_notification(title: &str, body: &str) -> Result<()> {
    mac_notification_sys::Notification::new()
        .title(title)
        .message(body)
        .send()
        .context("Notification macOS (mac-notification-sys) échouée")?;
    Ok(())
}
