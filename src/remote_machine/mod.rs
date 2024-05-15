use serde::{Serialize, Deserialize};

use crate::remote_session::RemoteSession;

#[derive(Serialize, Deserialize, Debug)]
enum ConnectionMethod {
    #[serde(alias = "ssh")]
    Ssh,
    #[serde(alias = "server")]
    Server,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteMachine {
    usr: String,
    addr: String,
    auth: Option<String>,
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
                     "cat",
                     vec!(self.file_path.clone())
                ).await);
            }
            ConnectionMethod::Server => {}
        }
    }

    pub async fn read_file_data(&self) -> Option<String> {
        if let Some(session) = &self.session {
            return Some(session.read_output().await)
        }

        None
    }
}

