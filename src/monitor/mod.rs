use std::fs::File;
use std::io::Read;

use serde::{Serialize, Deserialize};

use crate::remote_machine::RemoteMachine;

#[derive(Serialize, Deserialize, Debug)]
pub struct MonitorConfig {
    #[serde(skip)]
    pub path: String,
    pub remotes: Vec<RemoteMachine>,
}

pub struct Monitor {
    pub config: MonitorConfig,
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

    pub async fn setup(&mut self) {
        self.read_config();

        for machine in &mut self.config.remotes {
            machine.init().await;
        }
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

    pub fn get_machine_by_name(&self, name: &str) -> Option<&RemoteMachine> {
        for machine in &self.config.remotes {
            if machine.usr == name {
                return Some(&machine);
            }
        }

        None
    }
}

