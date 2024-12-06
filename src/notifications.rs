use std::io;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn send_notification(content: &str, time: Duration) -> Result<(), io::Error> {
    execute_notify_send(content, time)
}

fn execute_notify_send(content: &str, time: Duration) -> Result<(), io::Error> {
    let mut command = Command::new("notify-send")
        .arg("-t")
        .arg(time.as_millis().to_string())
        .arg(content)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    command.wait()?;

    Ok(())
}
