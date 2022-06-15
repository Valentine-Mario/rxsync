use crate::connection::*;
use crate::file_util::get_all_subdir;

mod config;
mod connection;
mod file_util;
mod sftp;

fn main() {
    println!("{:?}", get_all_subdir("./app").unwrap());
    let conn = SshCred::new(
        "root".to_string(),
        "password".to_string(),
        "127.0.0.1".to_string(),
        "22".to_string(),
    );
    match conn.connect() {
        Ok(a) => {
            if !a.authenticated() {
                panic!("ssh session not authenticated")
            }
            println!("{}", a.authenticated())
        }
        Err(e) => {
            println!("Error: {}", e)
        }
    }
}
