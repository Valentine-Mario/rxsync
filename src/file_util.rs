use crate::config::{CHECKSUM_FILE, IGNORE_FILE};
use adler::adler32_slice;
use glob::glob;
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};

pub fn get_file_size(path: &Path) -> Result<u64, Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

pub fn check_if_file(path: &Path) -> Result<bool, Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.is_file())
}

pub fn check_if_dir(path: &Path) -> Result<bool, Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.is_dir())
}

pub fn compare_checksum(buf1: &[u8], buf2: &[u8]) -> bool {
    adler32_slice(buf1) == adler32_slice(buf2)
}

pub fn create_checksum(buf: &[u8]) -> u32 {
    adler32_slice(buf)
}

pub fn get_all_files_subdir(path: &str) -> Result<Vec<PathBuf>, Error> {
    let resolved_path = format!("{}/**/*", path);
    let mut file_paths = vec![];
    for entry in glob(&resolved_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(paths) => {
                let check = check_if_file(Path::new(&paths))?;
                //ignore folders amd config files
                if check
                    && !Path::new(&paths).ends_with(IGNORE_FILE)
                    && !Path::new(&paths).ends_with(CHECKSUM_FILE)
                {
                    file_paths.push(paths);
                }
            }
            Err(_) => {}
        }
    }
    Ok(file_paths)
}

pub fn get_all_subdir(path: &str) -> Result<Vec<PathBuf>, Error> {
    let resolved_path = format!("{}/**", path);
    let mut folder_paths = vec![];
    folder_paths.push(PathBuf::from(path));
    for entry in glob(&resolved_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => folder_paths.push(path),
            Err(_) => {}
        }
    }
    Ok(folder_paths)
}

pub fn read_file(path: &Path) -> Result<Vec<u8>, Error> {
    let data = fs::read(path)?;
    Ok(data)
}

pub fn remove_ignored_path(
    src_path: &Path,
    src: &mut Vec<PathBuf>,
    ignore: &Vec<String>,
) -> Vec<PathBuf> {
    let folder_path = format!("{}/{}", src_path.to_str().unwrap(), IGNORE_FILE);
    if Path::new(&folder_path).exists() {
        for i in 0..src.len() {
            for j in ignore {
                if src[i].starts_with(j) {
                    src[i] = PathBuf::from("");
                }
            }
        }
    } else {
        return src.to_vec();
    }

    src.retain(|x| x.to_str().unwrap() != "");
    src.to_vec()
}
