use anyhow::{Context, Result};

#[cfg(target_os = "linux")]
pub fn send_notification(title: &str, body: &str) -> Result<()> {
    notify_rust::Notification::new()
        .summary(title)
        .body(&escape_markup(body))
        .show()
        .context("Notification Linux (notify-rust) échouée")?;
    Ok(())
}

/// The freedesktop spec reads the body as markup, and its place names come from the
/// IP lookup services. The summary is plain text, so it stays as is.
#[cfg(target_os = "linux")]
fn escape_markup(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// Windows needs no escaping: tauri-winrt-notification sets the toast text through
// the XML DOM (SetInnerText). macOS notifications take plain strings.
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
    tauri_winrt_notification::Toast::new(tauri_winrt_notification::Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .show()
        .context("Notification Windows (tauri-winrt-notification) échouée")
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

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn escape_markup_escapes_markup_characters() {
        assert_eq!(
            escape_markup("<b>Tom & Jerry</b>"),
            "&lt;b&gt;Tom &amp; Jerry&lt;/b&gt;"
        );
        assert_eq!(escape_markup("&lt;"), "&amp;lt;");
        assert_eq!(escape_markup("Paris, France"), "Paris, France");
    }
}
