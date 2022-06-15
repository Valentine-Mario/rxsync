use adler::adler32_slice;
use std::fs;
use std::io;
use std::path::Path;

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
