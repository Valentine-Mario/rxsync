use adler::adler32_slice;
use glob::glob;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn get_file_size(path: &Path) -> Result<u64, io::Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

pub fn check_if_file(path: &Path) -> Result<bool, io::Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.is_file())
}

pub fn check_if_dir(path: &Path) -> Result<bool, io::Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.is_dir())
}

pub fn compare_checksum(buf1: &[u8], buf2: &[u8]) -> bool {
    adler32_slice(buf1) == adler32_slice(buf2)
}

pub fn get_all_files_subdir(path: &str) -> Result<Vec<PathBuf>, io::Error> {
    let resolved_path = format!("{}/**/*", path);
    let mut file_paths = vec![];
    for entry in glob(&resolved_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(paths) => {
                let check = check_if_file(Path::new(&paths))?;
                if check {
                    file_paths.push(paths);
                }
            }
            Err(_) => {}
        }
    }
    Ok(file_paths)
}

pub fn get_all_subdir(path: &str) -> Result<Vec<PathBuf>, io::Error> {
    let resolved_path = format!("{}/**", path);
    let mut folder_paths = vec![];

    for entry in glob(&resolved_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => folder_paths.push(path),
            Err(_) => {}
        }
    }
    Ok(folder_paths)
}

pub fn read_file(path: &Path) -> Result<Vec<u8>, io::Error> {
    let data = fs::read(path)?;
    Ok(data)
}
