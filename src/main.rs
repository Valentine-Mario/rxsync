use crate::connection::*;
// use ssh2::Session;
// use std::io::prelude::*;
// use std::net::TcpStream;
// use std::path::Path;

mod connection;
mod file_util;
mod sftp;

fn main() {
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
