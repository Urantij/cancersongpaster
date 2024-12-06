use std::io;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SelectionError {
    #[error("НЕ УДАЛОСЬ ВЫПОЛНИТЬ(((((")]
    BadExecution { inner: io::Error },
    #[error("ОТКАЗАЛСЯ ПИСАТЬ ПРОСТО")]
    Cancelled,
    #[error("НАСРАЛ ПРОСТО В ВВОД")]
    BadInput,
}

// TODO вот бы разобраться как брать любую итер инто коллекцию, но 10 миллиардов вариантов в гугле все не подошли.
pub fn get_selection<'a>(options: &'a Vec<&str>, lines: usize) -> Result<&'a str, SelectionError> {
    let mut command = Command::new("dmenu")
        .arg("-l")
        .arg(lines.to_string())
        .arg("-i")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| SelectionError::BadExecution { inner: err })?;

    {
        let mut iter = options.iter().peekable();
        let mut in_pipe = command.stdin.take().unwrap();

        while let Some(line) = iter.next() {
            in_pipe
                .write_all(line.as_bytes())
                .map_err(|err| SelectionError::BadExecution { inner: err })?;

            if iter.peek().is_some() {
                in_pipe
                    .write_all("\n".as_bytes())
                    .map_err(|err| SelectionError::BadExecution { inner: err })?;
            }
        }
    }

    let output = command
        .wait_with_output()
        .map_err(|err| SelectionError::BadExecution { inner: err })?;

    if !output.status.success() {
        return Err(SelectionError::Cancelled);
    }

    // трим енд потому что дменю возвращает строку + \n
    let output = str::from_utf8(&output.stdout)
        .map_err(|_| SelectionError::BadInput)?
        .trim_end();

    match options.iter().find(|&&option| option == output) {
        Some(out) => Ok(&out),
        None => Err(SelectionError::BadInput),
    }

    // if !options.contains(&output) {
    //     return Err(SelectionError::BadInput);
    // }
    //
    // Ok(output.to_owned())
}
