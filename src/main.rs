use std:: path::Path;
use xsync::{connection::SshCred, sync};

fn main() {
    let conn =SshCred::new(
        "root".to_string(),
        "realboy22".to_string(),
        "127.0.0.1".to_string(),
        "22".to_string(),
    );

    match sync(&conn, Path::new("test_sync/"), Some(Path::new("elixir"))) {
        Ok(_) => println!("okay"),
        Err(e) => println!("{:?}", e),
    }

}
