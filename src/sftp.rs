use ssh2::{File, FileStat, Session, Sftp};
use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};

use std::io::prelude::*;

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
                eprintln!("error creating folder {:?} \n {:?}", path, err)
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
        match fs::create_dir_all(Path::new("").join(dest).join(src)){
            Ok(_)=>{
                let file_list= self.sftp.readdir(src)?;
                for i in file_list{
                    if i.1.is_dir(){
                        self.download_item(Path::new(&i.0), dest)?
                    }else{
                        self.download_file(Path::new(&i.0), &Path::new("").join(dest).join(&i.0))?
                    }
                }
            }
            Err(err)=>{
                println!("{:?}", err);
            }
        }

        Ok(())
    }


    pub fn download_file(&self, src: &Path, dest: &Path) -> Result<(), Error> {
        let (mut remote_file, stat) = self.sess.scp_recv(src)?;
        println!("...download file of size {} to path {:?}", stat.size(), dest);
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;
        fs::write(dest, contents)?;
        Ok(())
    }
}
