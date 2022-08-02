use crate::config::*;
use crate::connection::*;
use crate::file_util::*;
use crate::sftp::*;
use std::io::Error;
use std::path::Path;

pub mod config;
pub mod connection;
pub mod file_util;
pub mod sftp;

pub fn sync(ssh: &SshCred, src: &Path, dest: Option<&Path>) -> Result<(), Error> {
    let conn = ssh.connect()?;
    let sftp_conn = SftpSync::new(conn)?;
    create_checksum_file(src)?;
    //get toml config file
    let config_str = read_checksum_file(src)?;
    match parse_checksum_config(&config_str) {
        Ok(parsed_config) => {
            match dest {
                Some(dest_path) => {
                    sftp_conn.create_folder(dest_path);

                    if check_if_file(src)? {
                        //check if file has been uploaded
                        if parsed_config
                            .files
                            .contains_key(&format!("{}", src.to_str().unwrap()))
                        {
                            //get file checksum
                            let file_content = read_file(src)?;
                            let checksum_data = create_checksum(&file_content[..]);
                            //check if found checksum equals config checksum
                            match parsed_config
                                .files
                                .get(&format!("{}", src.to_str().unwrap()))
                            {
                                Some(config_checksum) => {
                                    if &format!("{}", checksum_data) == config_checksum {
                                        eprintln!("no update made to file. Nothing new to update")
                                    } else {
                                        let size = get_file_size(src)?;
                                        let filename = Path::new(src.file_name().unwrap());
                                        let absolue_path =
                                            Path::new("").join(dest_path).join(filename);
                                        sftp_conn.create_file(
                                            &absolue_path,
                                            &size,
                                            None,
                                            &file_content[..],
                                        )?;
                                        //update config file after successful upload
                                        config::update_folder_config(
                                            &config_str,
                                            "files",
                                            &src,
                                            &FolderConfig::Add(
                                                String::from(src.to_str().unwrap()),
                                                format!("{}", checksum_data),
                                            ),
                                        )?;
                                    }
                                }
                                None => {
                                    //do nothing
                                }
                            }
                        } else {
                            //get file size
                            let size = get_file_size(src)?;
                            let file_content = read_file(src)?;
                            //get file checksum
                            let checksum_data = create_checksum(&file_content[..]);
                            let filename = Path::new(src.file_name().unwrap());
                            let absolue_path = Path::new("").join(dest_path).join(filename);
                            sftp_conn.create_file(&absolue_path, &size, None, &file_content[..])?;
                            //update config file after successful upload
                            config::update_folder_config(
                                &config_str,
                                "files",
                                &src,
                                &FolderConfig::Add(
                                    String::from(src.to_str().unwrap()),
                                    format!("{}", checksum_data),
                                ),
                            )?;
                        }
                    } else {
                        //get all sub dir and removed ignored dir
                        let mut dir = get_all_subdir(&src.to_str().unwrap())?;
                        let ignore_files = get_ignore_file(src)?;
                        remove_ignored_path(src, &mut dir, &ignore_files);

                        let mut file_list = (get_all_files_subdir(&src.to_str().unwrap()))?;
                        remove_ignored_path(src, &mut file_list, &ignore_files);
                        //folders need to be created sequentially
                        //don't run with concurrency
                        for i in dir {
                            //resolve path and add to dir
                            let absolue_path = Path::new("").join(dest_path).join(&i);
                            sftp_conn.create_folder(&absolue_path);
                        }
                        //TODO: create files concurrently on muntiple threads
                        for i in file_list {
                            let size = get_file_size(&i)?;
                            let file_content = read_file(&i)?;
                            let filename = Path::new(i.file_name().unwrap());
                            let absolue_path = Path::new("").join(dest_path).join(filename);
                            sftp_conn.create_file(&absolue_path, &size, None, &file_content[..])?;
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
                        //get all sub dir and removed ignored dir
                        let mut dir = get_all_subdir(&src.to_str().unwrap())?;
                        let ignore_files = get_ignore_file(src)?;
                        remove_ignored_path(src, &mut dir, &ignore_files);

                        let mut file_list = (get_all_files_subdir(&src.to_str().unwrap()))?;
                        remove_ignored_path(src, &mut file_list, &ignore_files);
                        //folders need to be created sequentially
                        //don't run with concurrency
                        for i in dir {
                            sftp_conn.create_folder(&i);
                        }
                        //TODO: create files concurrently on muntiple threads
                        for i in file_list {
                            let size = get_file_size(&i)?;
                            let file_content = read_file(&i)?;
                            sftp_conn.create_file(&i, &size, None, &file_content[..])?;
                        }
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("error parsing toml config {:?}", err)
        }
    }

    Ok(())
}
