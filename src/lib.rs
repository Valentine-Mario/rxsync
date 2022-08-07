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
    create_checksum_file(src)?;
    //get toml config file
    let config_str = read_checksum_file(src)?;
    match parse_checksum_config(&config_str) {
        Ok(parsed_config) => {
            //get all sub dir and removed ignored dir
            let ignore_files = get_ignore_file(src)?;
            let mut dir = get_all_subdir(&src.to_str().unwrap())?;

            remove_ignored_path(src, &mut dir, &ignore_files);

            //get folders to delete and upload
            let delete_folder = get_items_to_delete(&parsed_config.folders, &dir);
            let upload_folder = get_items_to_upload(&parsed_config.folders, &dir);

            let mut file_list = (get_all_files_subdir(&src.to_str().unwrap()))?;
            remove_ignored_path(src, &mut file_list, &ignore_files);

            //get files to be deleted and upload
            let delete_files = get_items_to_delete(&parsed_config.files, &file_list);
            let upload_files = get_items_to_upload(&parsed_config.files, &file_list);

            //check if dest path is set
            match dest {
                Some(dest_path) => {
                    //create destination path
                    if !parsed_config
                        .folders
                        .contains_key(&String::from(dest_path.to_str().unwrap()))
                    {
                        sftp_conn.create_folder(dest_path);
                    }

                    if check_if_file(src)? {
                        //get file checksum
                        let file_content = read_file(src)?;
                        let checksum_data = create_checksum(&file_content[..]);

                        //check if tpml config has file
                        match parsed_config
                            .files
                            .get(&format!("{}", src.to_str().unwrap()))
                        {
                            Some(config_checksum) => {
                                //check if found checksum equals config checksum
                                if &format!("{}", checksum_data) == config_checksum {
                                    eprintln!("no update made to file. Nothing new to update")
                                } else {
                                    compute_and_add_file(
                                        &src,
                                        &dest_path,
                                        &file_content,
                                        checksum_data,
                                        &sftp_conn,
                                        &src,
                                    )?;
                                }
                            }
                            None => {
                                compute_and_add_file(
                                    &src,
                                    &dest_path,
                                    &file_content,
                                    checksum_data,
                                    &sftp_conn,
                                    &src,
                                )?;
                            }
                        }
                    } else {
                        //folders need to be created sequentially
                        //don't run with concurrency
                        for i in upload_folder {
                            create_and_add_folder(Path::new(&i), dest_path, &sftp_conn, &src)?;
                        }

                        // delete marked folder
                        for i in delete_folder {
                            compute_and_remove_folder(Path::new(&i), dest_path, &sftp_conn, &src)?;
                        }
                        //delete marked files
                        for i in delete_files {
                            compute_and_remove_file(Path::new(&i), dest_path, &sftp_conn, &src)?;
                        }

                        //new files to upload
                        for i in upload_files.iter() {
                            let file_content = read_file(&Path::new(&i))?;
                            let checksum_data = create_checksum(&file_content[..]);

                            compute_and_add_file(
                                &Path::new(i),
                                &dest_path,
                                &file_content,
                                checksum_data,
                                &sftp_conn,
                                &src,
                            )?;
                        }
                        //TODO: create files concurrently on muntiple threads
                        for i in file_list {
                            println!("content {:?}", i);

                            let file_content = read_file(&i)?;
                            let checksum_data = create_checksum(&file_content[..]);
                            match parsed_config.files.get(&format!("{}", i.to_str().unwrap())) {
                                Some(config_checksum) => {
                                    if &format!("{}", checksum_data) != config_checksum {
                                        compute_and_add_file(
                                            &i,
                                            &dest_path,
                                            &file_content,
                                            checksum_data,
                                            &sftp_conn,
                                            &src,
                                        )?;
                                    }
                                }
                                None => {
                                    //do nothing
                                    //already taken care of by the get_item_to_upload function above
                                }
                            }
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
            eprintln!("error parsing toml config:\n {:?}", err)
        }
    }

    Ok(())
}

fn compute_and_add_file(
    src: &Path,
    dest_path: &Path,
    file_content: &Vec<u8>,
    checksum_data: u32,
    sftp_conn: &SftpSync,
    original_src: &Path,
) -> Result<(), Error> {
    let size = get_file_size(src)?;
    let filename = Path::new(src.to_str().unwrap());
    let absolue_path = PathBuf::new().join(dest_path).join(filename);
    sftp_conn.create_file(&absolue_path, &size, None, &file_content[..])?;
    //update config file after successful upload
    config::update_folder_config(
        "files",
        &original_src,
        &FolderConfig::Add(
            String::from(src.to_str().unwrap()),
            format!("{}", checksum_data),
        ),
    )?;
    Ok(())
}

fn compute_and_remove_file(
    src: &Path,
    dest_path: &Path,
    sftp_conn: &SftpSync,
    original_src: &Path,
) -> Result<(), Error> {
    let absolue_path = PathBuf::new().join(dest_path).join(&src);
    sftp_conn.remove_file(&absolue_path)?;
    config::update_folder_config(
        "files",
        &original_src,
        &&FolderConfig::Remove(src.to_str().unwrap().to_string()),
    )?;
    Ok(())
}

fn create_and_add_folder(
    src: &Path,
    dest_path: &Path,
    sftp_conn: &SftpSync,
    original_src: &Path,
) -> Result<(), Error> {
    let absolue_path = PathBuf::new().join(dest_path).join(src);
    sftp_conn.create_folder(&absolue_path);
    update_folder_config(
        "folders",
        &original_src,
        &FolderConfig::Add(src.to_str().unwrap().to_string(), "".to_string()),
    )?;
    Ok(())
}

fn compute_and_remove_folder(
    src: &Path,
    dest_path: &Path,
    sftp_conn: &SftpSync,
    original_src: &Path,
) -> Result<(), Error> {
    let absolue_path = PathBuf::new().join(dest_path).join(src);
    sftp_conn.remove_dir(&absolue_path)?;
    config::update_folder_config(
        "folders",
        &original_src,
        &&FolderConfig::Remove(src.to_str().unwrap().to_string()),
    )?;
    Ok(())
}
