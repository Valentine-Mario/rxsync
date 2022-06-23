use crate::config::*;
use crate::connection::*;
use crate::file_util::*;
use crate::sftp::*;
use std::io::Error;
use std::path::Path;
use std::path::PathBuf;

pub mod config;
pub mod connection;
pub mod file_util;
pub mod sftp;

pub fn sync(ssh: &SshCred, src: &Path, dest: Option<&Path>) -> Result<(), Error> {
    let conn = ssh.connect()?;
    let sftp_conn = SftpSync::new(conn)?;

    match dest {
        Some(dest_path) => {
            if check_if_file(src)? {
                //get file size
                let size = get_file_size(src)?;
                let file_content = read_file(src)?;
                let filename = Path::new(src.file_name().unwrap());
                sftp_conn.create_folder(dest_path)?;
                let absolue_path = Path::new("").join(dest_path).join(filename);
                sftp_conn.create_file(&absolue_path, &size, None, &file_content[..])?;
            } else {
                let dir = get_all_subdir(&src.to_str().unwrap())?;
                println!("{:?}", dir);
                for i in dir {
                    sftp_conn.create_folder(&i)?;
                }
            }
        }
        None => {
            if check_if_file(src)? {
                //get file size
                let size = get_file_size(src)?;
                let file_content = read_file(src)?;
                let filename = Path::new(src.file_name().unwrap());
                sftp_conn.create_file(&filename, &size, None, &file_content[..])?;
            } else {
            }
        }
    }

    Ok(())
}
