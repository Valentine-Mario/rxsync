struct SshCred {
    name: String,
    password: String,
}

impl SshCred {
    pub fn new(self, name: String, password: String) -> Self {
        SshCred { name, password }
    }
}
