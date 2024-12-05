use crate::input;
use rand::prelude::SliceRandom;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::{Path, PathBuf};
use thiserror::Error;

const MAX_FILE_SIZE: u64 = 1000 * 1000;

pub struct CachedDirEntry {
    file_name: OsString,
    file_path: PathBuf,
    is_dir: bool,
    is_file: bool,
    size: u64,
}

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

    let main_folder_pathes: Vec<_> = get_dir_items(path)?;

    let mut dirs: Vec<_> = main_folder_pathes
        .iter()
        .filter(|&entry| entry.is_dir)
        .collect();

    let main_dir = path_to_cache(path)?;
    dirs.push(&main_dir);

    let all_files: Vec<CachedDirEntry> = dirs
        .into_iter()
        .flat_map(|dir| {
            // я устал, пусть паникует
            let items = get_dir_items(&dir.file_path).unwrap();

            items.into_iter().filter(|item| item.is_file)
        })
        .collect();

    Ok(all_files)
}

fn path_to_cache(path: &Path) -> Result<CachedDirEntry, SongsError> {
    let metadata = fs::metadata(path).map_err(|err| SongsError::NoReadFile { inner: err })?;

    Ok(CachedDirEntry {
        file_name: path.file_name().unwrap().to_owned(),
        file_path: path.to_path_buf(),
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file(),
        size: metadata.len(),
    })
}

fn get_dir_items(path: &Path) -> Result<Vec<CachedDirEntry>, SongsError> {
    let mut pathes: Vec<_> = path
        .read_dir()
        .map_err(|err| SongsError::NoReadFolder { inner: err })?
        .collect();

    // Ищет еррорнутный путь, чтобы дальше ерроров не было, хотя у нас резальтик
    if let Some(index) = pathes.iter().position(|path| path.is_err()) {
        let bad_item = pathes.remove(index);

        return Err(SongsError::NoReadFile {
            inner: bad_item.unwrap_err(),
        });
    }

    let mut cached: Vec<_> = pathes
        .into_iter()
        .map(|entry| {
            let entry = entry.unwrap();

            path_to_cache(&entry.path())
        })
        .collect();

    if let Some(index) = cached.iter().position(|path| path.is_err()) {
        let bad_item = cached.remove(index);

        // я не понял почему не дало unwrap_err сделать, но да ладно
        if let Err(err) = bad_item {
            return Err(err);
        }
    }

    Ok(cached.into_iter().map(|cache| cache.unwrap()).collect())
}

pub fn select_song(
    songs: &Vec<CachedDirEntry>,
    selection_type: SelectionType,
) -> Option<&CachedDirEntry> {
    match selection_type {
        SelectionType::Random => songs.choose(&mut rand::thread_rng()),
        SelectionType::DMenu => {
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
                return Some(
                    songs
                        .iter()
                        .find(|entry| Path::new(&entry.file_name).file_stem().unwrap() == name)
                        .unwrap(),
                );
            }

            eprintln!("Ошибка при выборе: {}", selection.unwrap_err());

            None
        }
    }
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
