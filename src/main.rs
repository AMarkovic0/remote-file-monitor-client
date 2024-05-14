use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Output;

use serde::{Serialize, Deserialize};
use openssh::{Session, KnownHosts};
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, Debug)]
enum ConnectionMethod {
    #[serde(alias = "ssh")]
    Ssh,
    #[serde(alias = "server")]
    Server,
}

#[derive(Debug)]
struct RemoteSession {
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

    pub async fn execute_command(&self) -> Output {
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
