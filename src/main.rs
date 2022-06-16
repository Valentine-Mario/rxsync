use crate::config::*;
use crate::connection::*;
use std::path::Path;

mod config;
mod connection;
mod file_util;
mod sftp;

fn main() {
    create_checksum_file(Path::new("./app")).unwrap();
    let data = read_checksum_file(Path::new("./app")).unwrap();
    //   println!("{:?}", parse_checksum_config(data).unwrap());
    update_folder_config(
        data,
        "files",
        Path::new("./app"),
        &FolderConfig::Add(String::from("/app/ksd/aaa"), String::from("123456")),
    )
    .unwrap();

    // let conn = SshCred::new(
    //     "root".to_string(),
    //     "password".to_string(),
    //     "127.0.0.1".to_string(),
    //     "22".to_string(),
    // );
    // match conn.connect() {
    //     Ok(a) => {
    //         if !a.authenticated() {
    //             panic!("ssh session not authenticated")
    //         }
    //         println!("{}", a.authenticated())
    //     }
    //     Err(e) => {
    //         println!("Error: {}", e)
    //     }
    // }
}
