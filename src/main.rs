use std::path::Path;
use syncer::*;

fn main() {
    config::create_checksum_file(Path::new("./app")).unwrap();
    let data = config::read_checksum_file(Path::new("./app")).unwrap();
    //   println!("{:?}", parse_checksum_config(data).unwrap());
    config::update_folder_config(
        &data,
        "files",
        Path::new("./app"),
        &config::FolderConfig::Add(String::from("/app/ksd/aaa"), String::from("123456")),
    )
    .unwrap();

    let conn = connection::SshCred::new(
        "root".to_string(),
        "password".to_string(),
        "127.0.0.1".to_string(),
        "22".to_string(),
    );
    match sync(&conn, Path::new("./app"), Some(Path::new("./app"))) {
        Ok(_) => println!("okay"),
        Err(e) => println!("{:?}", e),
    }
}
