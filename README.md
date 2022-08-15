### Xsync

A tool to help sync items in your local and remove server, pushing only modifications just like rsync

To install, add the following to your `Cargo.toml` file

```
rxsync = "0.1.0"
```


 - You can ignore files and folders by creating a `.xsyncignore` file in the base directory similar way you'd write a `.gitignore` file



 - To sync a file or directory to a remote server

 ```rs
 use std:: path::Path;
 use rxsync::{connection::SshCred, sync, connection::AuthOption};
 
 //multiple auth options include
 //Attempt basic password authentication.
 let auth1= AuthOption::UserauthPassword("ssh_username".to_string(), "ssh_password".to_string());
 //authenticate the current connection with the first public key found in an SSH agent
 let auth2= AuthOption::UserauthAgent("ssh_username".to_string());
 //Attempt public key authentication using a PEM encoded private key file stored on disk
 let auth3= AuthOption::UserauthPubkeyFile("ssh_username".to_string(), Some(&Path::new("pub_key")), &Path::new("private_key"), Some("passphrase"));

 let conn =SshCred::new(
     auth1,
     "host".to_string(),
     "port".to_string(),
  );

 sync(&conn, &Path::new("source_path/"), Some(Path::new("dir_path"))).unwrap();
 ```

 - This creates a `.xsync.toml` file in the base directory which is a snapshot of the latest synced files and directories on the server
   This file is how xsync can track what files or dir to update, delete or upload

 - To clone a directory or file

 ```rs
 use std:: path::Path;
 use rxsync::{connection::SshCred, clone_dir, clone_file, connection::AuthOption};

 let auth= AuthOption::UserauthAgent("ssh_username".to_string());
 let conn =SshCred::new(
     auth,
     "host".to_string(),
     "port".to_string(),
  );

 clone_dir(&conn, &Path::new("dir_to_clone"), &Path::new("write_dest")).unwrap();

 //config_dest is the destination you wish to write your .xsync.toml file which is optional
 clone_file(&conn, &Path::new("file_to_clone"), &Path::new("write_dest"), Some(&Path::new("config_dest"))).unwrap();

 ``````