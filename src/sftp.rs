use ssh2::{Session, Sftp};
use std::fs;
use std::io;
use std::path::Path;

use std::io::prelude::*;

pub struct SftpSync {
    sftp: Sftp,
    sess: Session,
}

impl SftpSync {
    pub fn new(sess: Session) -> Result<Self, io::Error> {
        let sftp = sess.sftp()?;
        Ok(SftpSync { sftp, sess })
    }

    pub fn create_folder(&self, path: &Path) -> Result<(), io::Error> {
        self.sftp.mkdir(path, 10)?;
        Ok(())
    }

    pub fn get_file_size(path: &Path) -> Result<u64, io::Error> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }

    pub fn create_file(
        &self,
        path: &Path,
        size: &u64,
        times: Option<(u64, u64)>,
        buf: &[u8],
    ) -> Result<(), io::Error> {
        let mut remote_file = self.sess.scp_send(path, 10, *size, times)?;

        remote_file.write(buf)?;
        Ok(())
    }
}
