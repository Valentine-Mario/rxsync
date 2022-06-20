use crate::config::*;
use crate::connection::*;
use std::io::Error;
use std::path::Path;

pub mod config;
pub mod connection;
pub mod file_util;
pub mod sftp;

pub fn sync(ssh: &SshCred, path: &Path) -> Result<(), Error> {
    let conn = ssh.connect()?;

    Ok(())
}
