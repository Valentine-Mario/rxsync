use crate::connection::*;
use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

mod connection;

fn main() {
    // Connect to the local SSH server
    let tcp = TcpStream::connect("127.0.0.1:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("root", "realboy22").unwrap();

    println!("{}", sess.authenticated());

    sess.sftp().unwrap().mkdir(Path::new("app"), 10).unwrap();

    // // Write the file
    let mut remote_file = sess
        .scp_send(Path::new("app/remotes"), 0o644, 10, None)
        .unwrap();
    remote_file.write(b"1234567890").unwrap();
    // Close the channel and wait for the whole content to be tranferred
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();
}
