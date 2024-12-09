mod clipboard;
mod files;
mod input;
mod keyboard;
mod notifications;
mod songs;

use crate::keyboard::KeyActionType;
use crate::notifications::send_notification;
use crate::songs::SelectionType;
use clap::{Parser, ValueHint};
use rdev::Key;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const DEFAULT_SONGS_PATH: &str = "Songs";
const TIMEOUT_IN_SECONDS: u64 = 7;
const DEFAULT_SELECTION: bool = false;
const PASTE_WAIT_TIME_IN_MILLIS: u64 = 50;
const DEFAULT_NOTIFY: bool = true;
const NOTIFY_TIMEOUT_IN_MILLIS: u64 = 2000;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to song folder
    #[arg(long, default_value_t = DEFAULT_SONGS_PATH.to_string(), value_hint = ValueHint::DirPath )]
    songs_path: String,
    /// Use dmenu for selection
    #[arg(short, default_value_t = DEFAULT_SELECTION )]
    select: bool,
    /// Timeout for inactivity in seconds
    #[arg(short, default_value_t = TIMEOUT_IN_SECONDS )]
    timeout: u64,
    /// Notify when program ends
    #[arg(short, default_value_t = DEFAULT_NOTIFY )]
    notify: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let args = Args::parse();

    let songs = songs::get_songs(Path::new(&args.songs_path))?;

    let selection_type = match args.select {
        true => SelectionType::DMenu,
        false => SelectionType::Random,
    };

    let song = songs::select_song(&songs, selection_type).unwrap();
    // TODO разобраться что такое еррор

    songs::check_song_file(song)?;

    let lines = songs::read_song(song)?;

    if lines.len() == 0 {
        panic!("Пустой вектор строк");
    }

    let single_instance = single_instance::SingleInstance::new("cancersongpaster")?;

    if !single_instance.is_single() {
        if args.notify {
            let _ = send_notification(
                "Строчки уже запущены",
                Duration::from_millis(NOTIFY_TIMEOUT_IN_MILLIS),
            );
        }
        drop(single_instance);
        std::process::exit(2);
    }

    let single_instance = Arc::new(Mutex::new(Some(single_instance)));

    // На Press плохо работает, иногда просто не вставляет строку в буфер обмена.
    // Да и если подумать... оно и должно на релизе срабатывать, на пресс же сам пейст срабатывает, ептыть
    let control = keyboard::ListenControl::create(Key::KeyV);

    let last_activity = Arc::new(Mutex::new(std::time::Instant::now()));

    let last_activity_intimer = last_activity.clone();

    let single_instance_intimer = single_instance.clone();

    thread::spawn(move || {
        let timeout_limit = Duration::from_secs(args.timeout);

        loop {
            let elapsed = last_activity_intimer.lock().unwrap().elapsed();

            if elapsed > timeout_limit {
                println!("долго думал");

                if args.notify {
                    let _ = send_notification(
                        "Строчки таймаут",
                        Duration::from_millis(NOTIFY_TIMEOUT_IN_MILLIS),
                    );
                }

                if let Some(instance) = single_instance_intimer.lock().unwrap().take() {
                    drop(instance);
                }

                let _ = clipboard::clear_clipboard();
                // Если не подождать, оно не вставит строку в буфер обмена.
                thread::sleep(Duration::from_millis(PASTE_WAIT_TIME_IN_MILLIS));
                std::process::exit(1);
            }

            thread::sleep(Duration::from_millis(100));
        }
    });

    let mut iter = lines.iter();

    while let Some(line) = iter.next() {
        println!("пишем строку");

        clipboard::paste_clipboard(line)?;

        // Пейст происходит, когда нажата v при зажатом ctrl
        // Нужно менять буфер обмена, когда v отжата после пейста
        let mut used_paste = false;
        loop {
            let event = control.receiver.recv()?;

            if event.action_type == KeyActionType::Press {
                if event.is_ctrl {
                    used_paste = true;
                }
            } else if used_paste {
                break;
            }
        }

        *last_activity.lock().unwrap() = std::time::Instant::now();
    }

    control.stop();

    if let Some(instance) = single_instance.lock().unwrap().take() {
        drop(instance);
    }

    clipboard::clear_clipboard()?;
    // Если не подождать, оно не вставит строку в буфер обмена.
    thread::sleep(Duration::from_millis(PASTE_WAIT_TIME_IN_MILLIS));

    if args.notify {
        let _ = send_notification(
            "Строчки всё!",
            Duration::from_millis(NOTIFY_TIMEOUT_IN_MILLIS),
        );
    }

    println!("конец :)");
    Ok(())
}
