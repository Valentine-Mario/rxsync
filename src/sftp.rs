use ssh2::{Session, Sftp};
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

    pub fn create_file(
        &self,
        path: &Path,
        size: &u64,
        times: Option<(u64, u64)>,
        buf: &[u8],
    ) -> Result<(), io::Error> {
        let mut remote_file = self.sess.scp_send(path, 10, *size, times)?;

        remote_file.write(buf)?;
        // Close the channel and wait for the whole content to be tranferred

        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(())
    }
}
