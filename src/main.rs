use std::env;
use std::fs::File;
use std::io::Read;
use std::error::Error;

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

#[derive(Serialize, Deserialize, Debug)]
struct RemoteMachine {
    usr: String,
    addr: String,
    auth: Option<String>,
    method: ConnectionMethod,
    file_path: String,
}

impl RemoteMachine {
    pub async fn init(&self) {
        match &self.method {
            ConnectionMethod::Ssh => {
                let session = Session::connect(
                    format!("{}@{}", self.usr, self.addr),
                    KnownHosts::Strict
                ).await
                .expect(&self.append_user_data("Failed to start ssh session for "));

                let ls = session.command("ls")
                    .output()
                    .await
                    .expect("Failed to execute command ls");

                println!(
                    "{}",
                    String::from_utf8(ls.stdout)
                        .expect("Failed to get command output")
                );
            }
            ConnectionMethod::Server => {}
        }
    }

    pub fn get_file(&self) -> String {
        self.request_file();
        self.read_file_data()
    }

    fn request_file(&self) {
        // Send request
    }

    fn read_file_data(&self) -> String {
        // Read received data
        String::new()
    }

    fn append_user_data(&self, msg: &str) -> String {
        format!("{} {}@{}", msg, self.usr, self.addr)
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

        for machine in &self.config.remotes {
            machine.init().await;
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
