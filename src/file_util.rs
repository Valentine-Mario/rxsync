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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_get_all_files_subdir() {
        let file_list = get_all_files_subdir("test_sync").unwrap();
        assert!(file_list.contains(&Path::new("test_sync/keep.txt").to_path_buf()));
        assert!(file_list.contains(&Path::new("test_sync/test2/test3/test_file").to_path_buf()));
    }

    #[test]
    fn test_get_all_subdir() {
        let dir_list = get_all_subdir("test_sync").unwrap();
        assert!(dir_list.contains(&Path::new("test_sync/test2").to_path_buf()));
        assert!(dir_list.contains(&Path::new("test_sync/test2/test3").to_path_buf()));
    }

    #[test]
    fn test_remove_ignored_path() {
        let unignored_path = remove_ignored_path(
            Path::new("test_sync"),
            &mut vec![
                Path::new("path1").to_path_buf(),
                Path::new("path2").to_path_buf(),
                Path::new("path3").to_path_buf(),
            ],
            &vec!["path2".to_string()],
        );
        assert!(unignored_path.len() == 3);

        let path = format!("test_sync/{}", IGNORE_FILE);
        let mut file = File::create(&path).expect("Error encountered while creating file!");
        file.write_all(b"path2")
            .expect("Error while writing to file");
        let unignored_path = remove_ignored_path(
            Path::new("test_sync"),
            &mut vec![
                Path::new("path1").to_path_buf(),
                Path::new("path2").to_path_buf(),
                Path::new("path3").to_path_buf(),
            ],
            &vec!["path2".to_string()],
        );
        assert!(unignored_path.len() == 2);
        assert!(unignored_path.contains(&Path::new("path1").to_path_buf()));
        assert!(unignored_path.contains(&Path::new("path3").to_path_buf()));
    }
}
