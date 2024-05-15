mod remote_session;
mod remote_machine;
mod monitor;

use std::env;

use tokio::runtime::Runtime;

use crate::monitor::Monitor;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut monitor = Monitor::new(args.get(1)
        .expect("Missing argument - Configuration path file not fount"));

    let rt = Runtime::new().expect("Failed to create async runtime");
    let _ = rt.block_on(async {
        monitor.run().await;
    });
}
