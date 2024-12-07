use std::io;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn clear_clipboard() -> Result<(), io::Error> {
    paste_xclip("")
}

pub fn paste_clipboard(content: &str) -> Result<(), io::Error> {
    paste_xclip(content)
}

fn paste_xclip(content: &str) -> Result<(), io::Error> {
    let mut command = Command::new("xclip")
        .args(["-sel", "clip"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    command
        .stdin
        .take()
        .unwrap()
        .write_all(content.as_bytes())?;

    command.wait()?;

    Ok(())
}
