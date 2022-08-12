//! A tool to help sync items in your local and remove server, pushing only modifications just like rsync
//!
//! - You can ignore files and folders by creating a `.xsyncignore` file in the base directory similar way you'd write a `.gitignore` file
//!
//!
//!
//! - To sync a file or directory to a remote server
//!
//! ```
//! use std:: path::Path;
//! use xsync::{connection::SshCred, sync};
//!
//! let conn =SshCred::new(
//!     "ssh_username".to_string(),
//!     "ssh_password".to_string(),
//!     "host".to_string(),
//!     "port".to_string(),
//!  );
//!
//! sync(&conn, &Path::new("source_path/"), Some(Path::new("dir_path"))).unwrap()
//! ```
//!
//! - This craetes a `.xsync.toml` file in the base directory which is a snapshot of the latest synced files and directories on the server
//!   This file is how xsync can track what files or dir to update, delete or upload
//!
//! - To clone a directory or file
//!
//! ```
//! use std:: path::Path;
//! use xsync::{connection::SshCred, clone_dir, clone_file};
//!
//! let conn =SshCred::new(
//!     "ssh_username".to_string(),
//!     "ssh_password".to_string(),
//!     "host".to_string(),
//!     "port".to_string(),
//!  );
//!
//! clone_dir(&conn, &Path::new("dir_to_clone"), &Path::new("write_dest")).unwrap()
//!
//! //config_dest is the destination you wish to write your .xsync.toml file which is optional
//! clone_file(&conn, &Path::new("file_to_clone"), &Path::new("write_dest"), Some(&Path::new("config_dest"))).unwrap()
//!
//! ```
//!
use crate::config::*;
use crate::connection::*;
use crate::file_util::*;
use crate::sftp::*;
use std::io::Error;
use std::path::Path;
use std::path::PathBuf;

mod config;
pub mod connection;
mod file_util;
mod sftp;

pub fn clone_dir(ssh: &SshCred, src: &Path, dest: &Path) -> Result<(), Error> {
    let conn = ssh.connect()?;
    let sftp_conn = SftpSync::new(conn)?;
    sftp_conn.download_item(src, dest)?;
    Ok(())
}

pub fn clone_file(
    ssh: &SshCred,
    src: &Path,
    dest: &Path,
    config_dest: Option<&Path>,
) -> Result<(), Error> {
    let conn = ssh.connect()?;
    let sftp_conn = SftpSync::new(conn)?;
    sftp_conn.download_file(src, dest, config_dest)?;
    Ok(())
}

pub fn sync(ssh: &SshCred, src: &Path, dest: Option<&Path>) -> Result<(), Error> {
    //get toml config file
    if check_if_dir(&src)? {
        create_checksum_file(src)?;

        let config_str = read_checksum_file(src)?;
        match parse_checksum_config(&config_str) {
            Ok(parsed_config) => {
                let conn = ssh.connect()?;
                let sftp_conn = SftpSync::new(conn)?;

                //get all sub dir and removed ignored dir
                let ignore_files = get_ignore_file(src)?;

                let mut dir = get_all_subdir(&src.to_str().unwrap())?;

                let mut dyn_path = String::from("");
                let mut dyn_vec = vec![];

                //get all folder component of parent path and add them to dir list
                let path_component = src.components();
                for i in path_component {
                    dyn_path += &format!("{}/", i.as_os_str().to_str().unwrap()).to_string();
                    dyn_vec.push(Path::new(&dyn_path).to_path_buf());
                }

                remove_ignored_path(src, &mut dir, &ignore_files);
                dyn_vec.append(&mut dir);

                //get folders to delete and upload
                let delete_folder = get_items_to_delete(&parsed_config.folders, &dyn_vec);
                let upload_folder = get_items_to_upload(&parsed_config.folders, &dyn_vec);

                let mut file_list = (get_all_files_subdir(&src.to_str().unwrap()))?;
                remove_ignored_path(src, &mut file_list, &ignore_files);
                //get files to be deleted and upload
                let delete_files = get_items_to_delete(&parsed_config.files, &file_list);
                let upload_files = get_items_to_upload(&parsed_config.files, &file_list);

                //check if dest path is set
                match dest {
                    Some(dest_path) => {
                        upload_and_sync(
                            &parsed_config,
                            &dest_path,
                            &src,
                            &sftp_conn,
                            upload_folder,
                            delete_folder,
                            upload_files,
                            delete_files,
                            file_list,
                        )?;
                    }
                    None => {
                        upload_and_sync(
                            &parsed_config,
                            Path::new(""),
                            &src,
                            &sftp_conn,
                            upload_folder,
                            delete_folder,
                            upload_files,
                            delete_files,
                            file_list,
                        )?;
                    }
                }
            }
            Err(err) => {
                eprintln!("error parsing toml config:\n {:?}", err)
            }
        }
    } else {
        //get parent path and parse it
        let parent = src.parent().unwrap().to_str().unwrap();
        let mut config_str = String::from("");
        //create toml config in parent path and read to string
        if parent == "" {
            create_checksum_file(Path::new("./")).unwrap();
            config_str += &read_checksum_file(Path::new("./"))?;
        } else {
            create_checksum_file(Path::new(parent)).unwrap();
            config_str += &read_checksum_file(Path::new(parent))?;
        }
        // let config_str = read_checksum_file(src)?;
        let conn = ssh.connect()?;
        let sftp_conn = SftpSync::new(conn)?;
        match parse_checksum_config(&config_str) {
            Ok(parsed_config) => match dest {
                Some(dest_path) => {
                    sync_file(
                        &parsed_config,
                        &src,
                        &dest_path,
                        &sftp_conn,
                        Path::new(parent),
                    )?;
                    return Ok(());
                }
                None => {
                    sync_file(
                        &parsed_config,
                        &src,
                        &Path::new(""),
                        &sftp_conn,
                        Path::new(parent),
                    )?;
                    return Ok(());
                }
            },
            Err(err) => {
                eprintln!("error parsing toml config:\n {:?}", err)
            }
        }
    }

    Ok(())
}

fn sync_file(
    parsed_config: &Config,
    src: &Path,
    dest_path: &Path,
    sftp_conn: &SftpSync,
    parent: &Path,
) -> Result<(), Error> {
    if parent.to_str().unwrap() != "" {
        let path_component = Path::new(parent).components();
        let mut dyn_path = String::from("");
        for i in path_component {
            dyn_path += &format!("{}/", i.as_os_str().to_str().unwrap()).to_string();
            create_and_add_folder(
                &Path::new(&dyn_path),
                &dest_path,
                &sftp_conn,
                &Path::new(parent),
            )?
        }
    }

    //resolve parent path if none if detected
    if parent.to_str().unwrap() == "" {
        let parent = Path::new("./");
        update_file(&parsed_config, &src, &dest_path, &sftp_conn, &parent)?;
        return Ok(());
    } else {
        update_file(&parsed_config, &src, &dest_path, &sftp_conn, &parent)?;
        return Ok(());
    }
}

fn update_file(
    parsed_config: &Config,
    src: &Path,
    dest_path: &Path,
    sftp_conn: &SftpSync,
    parent: &Path,
) -> Result<(), Error> {
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
                println!("no update made to file. Nothing new to update")
            } else {
                println!("upting file {:?}", src);
                compute_and_add_file(
                    &src,
                    &dest_path,
                    &file_content,
                    checksum_data,
                    &sftp_conn,
                    &parent,
                )?;
            }
            return Ok(());
        }
        None => {
            println!("creating file {:?}", src);
            compute_and_add_file(
                &src,
                &dest_path,
                &file_content,
                checksum_data,
                &sftp_conn,
                &parent,
            )?;
            return Ok(());
        }
    }
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

//TODO: delete folder recursively fails
fn _compute_and_remove_folder(
    src: &Path,
    dest_path: &Path,
    sftp_conn: &SftpSync,
    original_src: &Path,
) -> Result<(), Error> {
    let absolue_path = PathBuf::new().join(dest_path).join(src);
    sftp_conn._remove_dir(&absolue_path)?;
    config::update_folder_config(
        "folders",
        &original_src,
        &&FolderConfig::Remove(src.to_str().unwrap().to_string()),
    )?;
    Ok(())
}

fn upload_and_sync(
    parsed_config: &Config,
    dest_path: &Path,
    src: &Path,
    sftp_conn: &SftpSync,
    upload_folder: Vec<String>,
    _delete_folder: Vec<String>,
    upload_files: Vec<String>,
    delete_files: Vec<String>,
    file_list: Vec<PathBuf>,
) -> Result<(), Error> {
    //create destination path if not found in config
    if !parsed_config
        .folders
        .contains_key(&String::from(dest_path.to_str().unwrap()))
        && dest_path.to_str().unwrap() != ""
    {
        sftp_conn.create_folder(dest_path);
    }

    //folders need to be created sequentially
    //don't run with concurrency
    for i in upload_folder {
        create_and_add_folder(Path::new(&i), dest_path, &sftp_conn, &src)?;
    }

    // delete marked folder
    // for i in delete_folder {
    //     compute_and_remove_folder(Path::new(&i), dest_path, &sftp_conn, &src)?;
    // }

    //delete marked files
    for i in delete_files {
        compute_and_remove_file(Path::new(&i), dest_path, &sftp_conn, &src)?;
    }

    //new files to upload
    //TODO: create files concurrently on muntiple threads
    for i in upload_files {
        let file_content = read_file(&Path::new(&i))?;
        let checksum_data = create_checksum(&file_content[..]);
        println!("creating file {:?}", i);
        compute_and_add_file(
            &Path::new(&i),
            &dest_path,
            &file_content,
            checksum_data,
            &sftp_conn,
            &src,
        )?;
    }
    for i in &file_list {
        let file_content = read_file(&i)?;
        let checksum_data = create_checksum(&file_content[..]);
        match parsed_config.files.get(&format!("{}", i.to_str().unwrap())) {
            Some(config_checksum) => {
                if &format!("{}", checksum_data) != config_checksum {
                    println!("updating file {:?}", i);
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_sync() {
        match env::var_os("NAME") {
            Some(u) => match env::var_os("PASS") {
                Some(p) => {
                    let ssh = SshCred::new(
                        u.to_str().unwrap().to_string(),
                        p.to_str().unwrap().to_string(),
                        "127.0.0.1".to_string(),
                        "22".to_string(),
                    );
                    let session = ssh.connect();
                    assert!(session.unwrap().authenticated());
                    //sync test file
                    sync(&ssh, Path::new("test_sync_file"), None).unwrap();
                    //clone file back to a dir
                    clone_file(
                        &ssh,
                        Path::new("test_sync_file"),
                        Path::new("dummy_file"),
                        None,
                    )
                    .unwrap();
                    assert!(Path::new("dummy_file").exists());
                    let data = fs::read_to_string("dummy_file").expect("Unable to read file");
                    assert_eq!(data, "this is a test sync file");
                    //delete toml config
                    fs::remove_file(config::CHECKSUM_FILE).unwrap();
                    //delete dummy file
                    fs::remove_file("dummy_file").unwrap();

                    //sync test dir
                    sync(&ssh, Path::new("test_sync"), None).unwrap();
                    //clone folder back to dir
                    clone_dir(&ssh, Path::new("test_sync"), Path::new("test_sync_2")).unwrap();
                    let dir = get_all_subdir("test_sync_2").unwrap();
                    assert!(dir.contains(&Path::new("test_sync_2/test_sync").to_path_buf()));
                    assert!(dir.contains(&Path::new("test_sync_2/test_sync/test2").to_path_buf()));
                    assert!(
                        dir.contains(&Path::new("test_sync_2/test_sync/test2/test3").to_path_buf())
                    );

                    let files = get_all_files_subdir("test_sync_2").unwrap();
                    assert!(
                        files.contains(&Path::new("test_sync_2/test_sync/keep.txt").to_path_buf())
                    );
                    assert!(files.contains(
                        &Path::new("test_sync_2/test_sync/test2/test3/test_file").to_path_buf()
                    ));
                    fs::remove_dir_all("test_sync_2").unwrap();
                    fs::remove_file(format!("test_sync/{}", config::CHECKSUM_FILE)).unwrap();
                }
                None => {
                    println!("skipping create_tcp_connection() test...")
                }
            },
            None => {
                println!("skipping create_tcp_connection() test...")
            }
        };
    }
}
