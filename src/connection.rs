use ssh2::Session;
use std::{io::Error, net::TcpStream};

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

#[cfg(test)]
mod tests {
    use super::SshCred;
    use std::env;

    #[test]
    fn create_tcp_connection() {
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
                    assert!(session.unwrap().authenticated())
                }
                None => {
                    println!("skipping create_tcp_connection() test...")
                }
            },
            None => {
                println!("skipping create_tcp_connection() test...")
            }
        };
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
