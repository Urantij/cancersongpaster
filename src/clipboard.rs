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
    // paste_xclip(content)
    paste_arboard(content)
}

fn paste_arboard(content: &str) -> Result<(), arboard::Error> {
    // Если так делать, он почему-то вместо текста вставляет коды символов из текста
    // let mut clipboard = Clipboard::new()?;
    // clipboard.set_text(content)?;

    AR_WRAPPER.paste(content)
}

// fn paste_xclip(content: &str) -> Result<(), io::Error> {
//     let mut command = Command::new("xclip")
//         .args(["-sel", "clip"])
//         .stdin(Stdio::piped())
//         .stdout(Stdio::null())
//         .stderr(Stdio::null())
//         .spawn()?;
//
//     command
//         .stdin
//         .take()
//         .unwrap()
//         .write_all(content.as_bytes())?;
//
//     command.wait()?;
//
//     Ok(())
// }
