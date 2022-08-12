use ssh2::Session;
use std::path::Path;
use std::{io::Error, net::TcpStream};

pub struct SshCred<'a> {
    auth: AuthOption<'a>,
    host: String,
    port: String,
}

///Auth options for ssh credentials
pub enum AuthOption<'a> {
    ///Attempt basic password authentication.
    UserauthPassword(String, String),
    ///authenticate the current connection with the first public key found in an SSH agent
    UserauthAgent(String),
    ///Attempt public key authentication using a PEM encoded private key file stored on disk.
    UserauthPubkeyFile(String, Option<&'a Path>, &'a Path, Option<&'a str>),
}

impl SshCred<'static> {
    pub fn new(auth: AuthOption<'static>, host: String, port: String) -> Self {
        SshCred { auth, host, port }
    }

    pub fn connect(&self) -> Result<Session, Error> {
        let url_host = format!("{}:{}", self.host, self.port);
        let tcp = TcpStream::connect(url_host)?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        match &self.auth {
            AuthOption::UserauthAgent(username) => sess.userauth_agent(&username)?,
            AuthOption::UserauthPassword(username, password) => {
                sess.userauth_password(&username, &password)?
            }
            AuthOption::UserauthPubkeyFile(username, pubkey, privatekey, passphrase) => {
                sess.userauth_pubkey_file(&username, *pubkey, privatekey, *passphrase)?
            }
        };
        Ok(sess)
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthOption, SshCred};
    use std::env;

    #[test]
    fn test_create_tcp_connection() {
        match env::var_os("NAME") {
            Some(u) => match env::var_os("PASS") {
                Some(p) => {
                    let ssh = SshCred::new(
                        AuthOption::UserauthPassword(
                            u.to_str().unwrap().to_string(),
                            p.to_str().unwrap().to_string(),
                        ),
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
    }
}
