use serde::{Serialize, Deserialize};

use crate::remote_session::RemoteSession;
use crate::remote_session::BoxResult;

#[derive(Serialize, Deserialize, Debug)]
enum ConnectionMethod {
    #[serde(alias = "ssh")]
    Ssh,
    #[serde(alias = "server")]
    Server,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteMachine {
    pub usr: String,
    addr: String,
    method: ConnectionMethod,
    file_path: String,
    #[serde(skip)]
    session: Option<RemoteSession>,
}

impl RemoteMachine {
    pub async fn init(&mut self) {
        match &self.method {
            ConnectionMethod::Ssh => {
                self.session = Some(RemoteSession::new(
                     &self.usr,
                     &self.addr,
                ).await);
            }
            ConnectionMethod::Server => {}
        }
    }

    pub async fn read_file_data(&self) -> Option<String> {
        if let Some(session) = &self.session {
            if let Ok(data) = session.read_file(&self.file_path).await {
                return Some(data);
            } else {
                println!("Failed to read file {}", self.file_path);
            }
        }

        None
    }

    pub async fn write_file(&self, file_ctx: &str) -> BoxResult<()> {
        if let Some(session) = &self.session {
            if let Err(e) = session.write_file(&self.file_path, file_ctx).await {
                return Err(e);
            }
        }

        Ok(())
    }
}


