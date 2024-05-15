use std::process::Output;

use openssh::{Session, KnownHosts};

#[derive(Debug)]
pub struct RemoteSession {
    session: Session,
    command: String,
    arguments: Vec<String>,
}

impl RemoteSession {
    pub async fn new(usr: &str, addr: &str, cmd: &str, args: Vec<String>) -> Self {
        let session = Session::connect(
            format!("{}@{}", usr, addr),
            KnownHosts::Strict
        ).await
        .expect(&format!("Failed to start ssh session for {}@{}", usr, addr));


        RemoteSession {
            session: session,
            command: cmd.to_string(),
            arguments: args
        }
    }

    async fn execute_command(&self) -> Output {
        self.session.command(&self.command)
            .args(&self.arguments)
            .output()
            .await
            .expect(&format!("Session failed to execute command"))
    }

    pub async fn read_output(&self) -> String {
        let cmd = self.execute_command().await;
        String::from_utf8(cmd.stdout).expect("Failed to get command output")
    }
}

