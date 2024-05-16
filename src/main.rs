mod remote_session;
mod remote_machine;
mod monitor;

use std::env;

use crate::monitor::Monitor;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    Monitor::new(args.get(1)
        .expect("Missing argument - Configuration path file not fount"))
        .run()
        .await;
}
