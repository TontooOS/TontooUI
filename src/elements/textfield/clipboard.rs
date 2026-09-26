use std::sync::{Mutex, OnceLock};

/// System clipboard for copy/cut/paste, with an in-process fallback.
///
/// `arboard` talks to the display server; where that fails (headless
/// tests, missing server) edits fall back to a process-local buffer
/// so shortcuts keep working inside the app.
fn system() -> &'static Mutex<Option<arboard::Clipboard>> {
    static SYSTEM: OnceLock<Mutex<Option<arboard::Clipboard>>> = OnceLock::new();
    SYSTEM.get_or_init(|| Mutex::new(arboard::Clipboard::new().ok()))
}

fn fallback() -> &'static Mutex<String> {
    static FALLBACK: OnceLock<Mutex<String>> = OnceLock::new();
    FALLBACK.get_or_init(|| Mutex::new(String::new()))
}

/// Read text: system clipboard first, fallback buffer otherwise.
pub(crate) fn get() -> Option<String> {
    if let Ok(mut guard) = system().lock() {
        if let Some(clipboard) = guard.as_mut() {
            if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    fallback().lock().ok().and_then(|guard| {
        if guard.is_empty() {
            None
        } else {
            Some(guard.clone())
        }
    })
}

/// Test serialization: clipboard tests share the global buffers,
/// so each holds this lock for its duration (poison-tolerant).
#[cfg(test)]
pub(crate) fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Write text: always mirrors into the fallback buffer so pastes
/// work even where the system clipboard is unreachable. Returns
/// true when the system clipboard accepted the text.
pub(crate) fn set(text: &str) -> bool {
    if let Ok(mut guard) = fallback().lock() {
        *guard = text.to_string();
    }
    if let Ok(mut guard) = system().lock() {
        if let Some(clipboard) = guard.as_mut() {
            return clipboard.set_text(text.to_string()).is_ok();
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_roundtrip_without_display_server() {
        let _guard = super::test_lock();
        // Headless CI has no clipboard: the fallback keeps the API
        // total (set always stores, get returns what was set).
        set("hello clipboard");
        assert_eq!(get().as_deref(), Some("hello clipboard"));
        set("");
        // Empty writes read back as None (nothing to paste).
        assert_eq!(get(), None);
    }
}
