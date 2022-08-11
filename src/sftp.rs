use ssh2::{ErrorCode, File, Session, Sftp};
use std::fs;
use std::io::Error;
use std::path::Path;

use std::io::prelude::*;

use crate::config::*;
use crate::file_util::create_checksum;

pub struct SftpSync {
    pub sftp: Sftp,
    pub sess: Session,
}

impl SftpSync {
    pub fn new(sess: Session) -> Result<Self, Error> {
        let sftp = sess.sftp()?;
        Ok(SftpSync { sftp, sess })
    }

    pub fn create_folder(&self, path: &Path) {
        match self.sftp.mkdir(path, 10) {
            Ok(_) => {}
            Err(err) => {
                match err.code() {
                    ErrorCode::SFTP(num) => {
                        if num == 4 {
                            //do nothing just trying to create an exisiting folder
                        } else {
                            eprintln!("error creating folder {:?} \n {:?}", path, err)
                        }
                    }
                    ErrorCode::Session(_) => {
                        eprintln!("error creating folder {:?} \n {:?}", path, err)
                    }
                }
            }
        }
    }

    pub fn create_file(
        &self,
        path: &Path,
        size: &u64,
        times: Option<(u64, u64)>,
        buf: &[u8],
    ) -> Result<(), Error> {
        let mut remote_file = self.sess.scp_send(path, 0o644, *size, times)?;
        remote_file.write(buf)?;
        // Close the channel and wait for the whole content to be tranferred

        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(())
    }

    pub fn read_file(&self, path: &Path) -> Result<File, Error> {
        let file = self.sftp.open(path)?;
        Ok(file)
    }

    pub fn remove_dir(&self, path: &Path) -> Result<(), Error> {
        self.sftp.rmdir(path)?;

        Ok(())
    }

    pub fn remove_file(&self, path: &Path) -> Result<(), Error> {
        self.sftp.unlink(path)?;
        Ok(())
    }

    pub fn download_item(&self, src: &Path, dest: &Path) -> Result<(), Error> {
        let path = Path::new("").join(dest).join(src);
        match fs::create_dir_all(&path) {
            Ok(_) => {
                create_checksum_file(&dest)?;
                update_folder_config(
                    "folders",
                    &dest,
                    &FolderConfig::Add(src.to_str().unwrap().to_string(), "".to_string()),
                )?;
                let file_list = self.sftp.readdir(src)?;
                for i in file_list {
                    if i.1.is_dir() {
                        self.download_item(Path::new(&i.0), dest)?
                    } else {
                        self.download_file(
                            Path::new(&i.0),
                            &Path::new("").join(dest).join(&i.0),
                            Some(dest),
                        )?
                    }
                }
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }

        Ok(())
    }

    pub fn download_file(
        &self,
        src: &Path,
        dest: &Path,
        config_dest: Option<&Path>,
    ) -> Result<(), Error> {
        let (mut remote_file, stat) = self.sess.scp_recv(src)?;
        println!(
            "...download file of size {} to path {:?}",
            stat.size(),
            dest
        );
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;
        fs::write(dest, &contents)?;
        match config_dest {
            Some(dest) => {
                let checksum_data = create_checksum(&contents[..]);
                update_folder_config(
                    "files",
                    &dest,
                    &FolderConfig::Add(
                        String::from(src.to_str().unwrap()),
                        format!("{}", checksum_data),
                    ),
                )?;
            }
            None => {}
        }

        remote_file.send_eof().unwrap();
        remote_file.wait_eof().unwrap();
        remote_file.close().unwrap();
        remote_file.wait_close().unwrap();
        Ok(())
    }
}
