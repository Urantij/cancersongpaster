use crate::files::{get_directory_items, path_to_cache, CachedDirEntry};
use crate::input;
use crate::input::SelectionError;
use rand::prelude::SliceRandom;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::Path;
use thiserror::Error;

const MAX_FILE_SIZE: u64 = 1000 * 1000;

pub enum SelectionType {
    Random,
    DMenu,
}

#[derive(Error, Debug)]
pub enum SongsError {
    #[error("ПАПКИ НЕТ(((((")]
    NoFolder,
    #[error("ПАПКУ НЕ ЧИТАЕТ(((((")]
    NoReadFolder { inner: Error },
    #[error("ФАЙЛ НЕ ЧИТАЕТ(((((")]
    NoReadFile { inner: Error },
    #[error("ФАЙЛ ПЛОХОЙ((((")]
    BadFile,
}

pub fn get_songs(path: &Path) -> Result<Vec<CachedDirEntry>, SongsError> {
    if !path.exists() {
        return Err(SongsError::NoFolder);
    }

    let main_cache = path_to_cache(path).map_err(|err| SongsError::NoReadFolder { inner: err })?;

    let main_folder_items: Vec<_> =
        get_directory_items(path).map_err(|err| SongsError::NoReadFolder { inner: err })?;

    let mut dirs: Vec<_> = main_folder_items
        .iter()
        .filter(|&entry| entry.is_dir)
        .collect();

    dirs.push(&main_cache);

    let all_files: Vec<CachedDirEntry> = dirs
        .into_iter()
        .flat_map(|dir| {
            // я устал, пусть паникует
            let items = get_directory_items(&dir.file_path).unwrap();

            items.into_iter().filter(|item| item.is_file)
        })
        .collect();

    Ok(all_files)
}

pub fn select_song(
    songs: &Vec<CachedDirEntry>,
    selection_type: SelectionType,
) -> Option<&CachedDirEntry> {
    match selection_type {
        SelectionType::Random => songs.choose(&mut rand::thread_rng()),
        SelectionType::DMenu => {
            let selection = get_song_selection(songs);

            if let Ok(cached) = selection {
                return Some(cached);
            } else if let Err(err) = selection {
                eprintln!("Ошибка при выборе: {}", err);
            }

            None
        }
    }
}

fn get_song_selection(songs: &Vec<CachedDirEntry>) -> Result<&CachedDirEntry, SelectionError> {
    let file_names: Vec<String> = songs
        .iter()
        .map(|entry| {
            let a = &entry.file_name;

            let result = Path::new(&a).file_stem().unwrap().to_str().unwrap();

            return result.to_owned();
        })
        .collect();

    // TODO вот бы разобраться как не делать коллекцию ссылок на строки, чтобы положить строки
    let file_names2: Vec<&str> = file_names.iter().map(|a| a.as_str()).collect();

    let selection = input::get_selection(&file_names2, 4);

    if let Ok(name) = selection {
        let cached = songs
            .iter()
            .find(|entry| Path::new(&entry.file_name).file_stem().unwrap() == name)
            .unwrap();

        return Ok(cached);
    }

    Err(selection.unwrap_err())
}

pub fn check_song_file(entry: &CachedDirEntry) -> Result<(), SongsError> {
    if !entry.is_file {
        return Err(SongsError::BadFile);
    }

    if entry.size > MAX_FILE_SIZE {
        return Err(SongsError::BadFile);
    }

    Ok(())
}

pub fn read_song(entry: &CachedDirEntry) -> Result<Vec<String>, SongsError> {
    let file = File::open(&entry.file_path).map_err(|err| SongsError::NoReadFile { inner: err })?;

    Ok(BufReader::new(file).lines().flatten().collect())
}
