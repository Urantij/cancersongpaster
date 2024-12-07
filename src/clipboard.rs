use arboard::Clipboard;
use std::sync::{Arc, LazyLock, Mutex};

struct ClipboardWrapper {
    clipboard: Arc<Mutex<Clipboard>>,
}

impl ClipboardWrapper {
    fn new() -> ClipboardWrapper {
        ClipboardWrapper {
            clipboard: Arc::new(Mutex::new(Clipboard::new().unwrap())),
        }
    }

    fn clear(&self) -> Result<(), arboard::Error> {
        self.clipboard.lock().unwrap().clear()
    }

    fn paste(&self, content: &str) -> Result<(), arboard::Error> {
        self.clipboard.lock().unwrap().set_text(content)
    }
}

static AR_WRAPPER: LazyLock<ClipboardWrapper> = LazyLock::new(|| ClipboardWrapper::new());

pub fn clear_clipboard() -> Result<(), arboard::Error> {
    AR_WRAPPER.clear()
}

pub fn paste_clipboard(content: &str) -> Result<(), arboard::Error> {
    AR_WRAPPER.paste(content)
}
