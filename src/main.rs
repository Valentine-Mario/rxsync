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

    // sess.sftp().unwrap().mkdir(Path::new("app"), 10).unwrap();

    // // // Write the file
    // let mut remote_file = sess
    //     .scp_send(Path::new("app/remotes"), 0o644, 10, None)
    //     .unwrap();
    // remote_file.write(b"1234567890").unwrap();
    // // Close the channel and wait for the whole content to be tranferred
    // remote_file.send_eof().unwrap();
    // remote_file.wait_eof().unwrap();
    // remote_file.close().unwrap();
    // remote_file.wait_close().unwrap();
}
