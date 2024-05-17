use std::process::Output;

use openssh::{Session, KnownHosts};

#[derive(Debug)]
pub struct RemoteSession {
    session: Session,
}

impl RemoteSession {
    pub async fn new(usr: &str, addr: &str) -> Self {
        let session = Session::connect(
            format!("{}@{}", usr, addr),
            KnownHosts::Strict
        ).await
        .expect(&format!("Failed to start ssh session for {}@{}", usr, addr));


        RemoteSession {
            session: session,
        }
    }

    async fn execute_command(&self, cmd: &str, args: &Vec<&str>) -> Output {
        self.session.command(cmd)
            .raw_args(args)
            .output()
            .await
            .expect(&format!("Session failed to execute command"))
    }

    pub async fn read_file(&self, file_path: &str) -> String {
        let args = vec![file_path];

        let cmd = self.execute_command("cat", &args).await;
        String::from_utf8(cmd.stdout).expect("Failed to get command output")
    }

    pub async fn write_file(&self, file_path: &str, file_ctx: &str) {
        let args = vec![file_ctx, ">>", file_path];
        let cmd = self.execute_command("echo", &args).await;
    }
}

