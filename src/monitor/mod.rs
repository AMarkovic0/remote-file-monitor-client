use std::fs::File;
use std::io::Read;

use serde::{Serialize, Deserialize};

use crate::remote_machine::RemoteMachine;

#[derive(Serialize, Deserialize, Debug)]
struct MonitorConfig {
    #[serde(skip)]
    pub path: String,
    pub remotes: Vec<RemoteMachine>,
}

pub struct Monitor {
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

