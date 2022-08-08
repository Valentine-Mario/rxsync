use std::io::prelude::*;
use std::{fs, path::Path};
use syncer::{sftp::SftpSync, *};

fn main() {
    // config::create_checksum_file(Path::new("./app")).unwrap();
    // let data = config::read_checksum_file(Path::new("./app")).unwrap();
    // //   println!("{:?}", parse_checksum_config(data).unwrap());
    // config::update_folder_config(
    //     &data,
    //     "files",
    //     Path::new("./app"),
    //     &config::FolderConfig::Add(String::from("/app/ksd/aaa"), String::from("123456")),
    // )
    // .unwrap();

    let conn = connection::SshCred::new(
        "root".to_string(),
        "realboy22".to_string(),
        "127.0.0.1".to_string(),
        "22".to_string(),
    );
    // let conn = conn.connect().unwrap();
    // let sftp_conn = SftpSync::new(conn).unwrap();
    // sftp_conn
    //     .download_item(Path::new("app"), Path::new("hey"))
    //     .unwrap();
    // let link = sftp_conn.sftp.readdir(Path::new(".config")).unwrap();
    // println!("{:?}", link);
    // println!("{:?}", link[0].1.is_dir());
    // fs::create_dir("app").unwrap();
    // println!("remote file size: {}", stat.size());
    // let mut contents = Vec::new();
    // remote_file.read_to_end(&mut contents).unwrap();
    // fs::write("remote", contents).unwrap();

    // // Close the channel and wait for the whole content to be tranferred
    // remote_file.send_eof().unwrap();
    // remote_file.wait_eof().unwrap();
    // remote_file.close().unwrap();
    // remote_file.wait_close().unwrap();
    // println!("{:?}", Path::new("app2/amazon").components());
    // let a= Path::new("app2/amazon").components();
    // let mut path=String::from("");
    // for i in a{
    //     path+=&format!("{}/", i.as_os_str().to_str().unwrap()).to_string();
    //     println!("{:?}", path)
    // }
    match sync(&conn, Path::new("elx/"), Some(Path::new("elixir"))) {
        Ok(_) => println!("okay"),
        Err(e) => println!("{:?}", e),
    }
}
