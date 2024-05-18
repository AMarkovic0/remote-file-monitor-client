use std::process::Output;
use std::error::Error;

use openssh::{Session, KnownHosts};

pub type BoxResult<T> = Result<T, Box<(dyn Error + 'static)>>;

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

    async fn execute_command(&self, cmd: &str, args: &Vec<&str>) -> BoxResult<Output> {
        Ok(self.session.command(cmd)
            .raw_args(args)
            .output()
            .await?)
    }

    pub async fn read_file(&self, file_path: &str) -> BoxResult<String> {
        let args = vec![file_path];

        let cmd = self.execute_command("cat", &args).await?;
        Ok(String::from_utf8(cmd.stdout)?)
    }

    pub async fn write_file(&self, file_path: &str, file_ctx: &str) -> BoxResult<()> {
        let args = vec![file_ctx, ">", file_path];
        let _cmd = self.execute_command("echo", &args).await?;

        Ok(())
    }
}

