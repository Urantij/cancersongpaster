use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CachedDirEntry {
    pub file_name: OsString,
    pub file_path: PathBuf,
    pub is_dir: bool,
    pub is_file: bool,
    pub size: u64,
}

pub fn get_directory_items(path: &Path) -> Result<Vec<CachedDirEntry>, std::io::Error> {
    let entries: Vec<_> = path
        .read_dir()?
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(_) => None,
        })
        .filter_map(|entry| {
            let result = path_to_cache(&entry.path());

            match result {
                Ok(cached) => Some(cached),
                Err(_) => None,
            }
        })
        .collect();

    Ok(entries)
}

pub fn path_to_cache(path: &Path) -> Result<CachedDirEntry, std::io::Error> {
    let metadata = fs::metadata(path)?;

    Ok(CachedDirEntry {
        file_name: path.file_name().unwrap().to_owned(),
        file_path: path.to_path_buf(),
        is_dir: metadata.is_dir(),
        is_file: metadata.is_file(),
        size: metadata.len(),
    })
}
