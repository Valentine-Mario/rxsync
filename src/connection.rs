use ssh2::Session;
use std::io::Error;
use std::net::TcpStream;

pub struct SshCred {
    name: String,
    password: String,
    host: String,
    port: String,
}

impl SshCred {
    pub fn new(name: String, password: String, host: String, port: String) -> Self {
        SshCred {
            name,
            password,
            host,
            port,
        }
    }

    pub fn connect(&self) -> Result<Session, Error> {
        let url_host = format!("{}:{}", self.host, self.port);
        let tcp = TcpStream::connect(url_host)?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        sess.userauth_password(self.name.as_str(), self.password.as_str())?;
        Ok(sess)
    }
}
