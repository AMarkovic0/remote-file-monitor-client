mod remote_session;

use std::env;
use std::fs::File;
use std::io::Read;

use serde::{Serialize, Deserialize};
use tokio::runtime::Runtime;

use crate::remote_session::RemoteSession;

#[derive(Serialize, Deserialize, Debug)]
enum ConnectionMethod {
    #[serde(alias = "ssh")]
    Ssh,
    #[serde(alias = "server")]
    Server,
}

#[derive(Serialize, Deserialize, Debug)]
struct RemoteMachine {
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

#[derive(Serialize, Deserialize, Debug)]
struct MonitorConfig {
    #[serde(skip)]
    pub path: String,
    pub remotes: Vec<RemoteMachine>,
}

struct Monitor {
    config: MonitorConfig,
}

impl Monitor {
    pub fn new(config_path: &String) -> Self {
        Monitor {
            config: MonitorConfig {
                path: config_path.clone(),
                remotes: Vec::new(),
            }
        }
    }

    pub async fn run(&mut self) {
        self.setup();

        for machine in &mut self.config.remotes {
            machine.init().await;
            println!("{}", machine.read_file_data().await.expect("Cannnot obtain machine data"));
        }
    }

    fn setup(&mut self) {
        self.read_config();
    }

    fn read_config(&mut self) {
        let mut data = String::new();
        let tmp_path = self.config.path.clone();

        let mut file = File::open(&self.config.path)
            .expect(&format!("Failed to opena a file {}", self.config.path));
        file.read_to_string(&mut data)
            .expect(&format!("Failed reading reading file {}", self.config.path));

        self.config = serde_json::from_str(&data)
            .expect("Configuration file JSON was not well-formatted");
        self.config.path = tmp_path;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut monitor = Monitor::new(args.get(1)
        .expect("Missing argument - Configuration path file not fount"));

    let rt = Runtime::new().expect("Failed to create async runtime");
    let _ = rt.block_on(async {
        monitor.run().await;
    });
}
