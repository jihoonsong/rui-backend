mod config;

use config::Config;
use rui_backend_rpc::RpcServer;
use std::{fs, io};
use tracing::info;

#[tokio::main]
async fn main() {
    // Init tracing.
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(io::stdout)
        .init();

    // Run a client and a network of nodes.
    run().await;
}

async fn run() {
    // Read the config file.
    let config_content =
        fs::read_to_string("./backend/app/Config.toml").expect("Failed to read config");
    let configs: Config = toml::from_str(&config_content).expect("Failed to parse config");

    // Start the RPC server.
    let rpc_server = RpcServer::new(configs.address)
        .build()
        .await
        .expect("Failed to build RPC server");
    let mut rpc_server_task = tokio::spawn(rpc_server.stopped());

    info!(address=?configs.address, "RPC server started");

    match tokio::try_join!(&mut rpc_server_task) {
        Ok(_) => {
            info!("All tasks completed");
        }
        Err(e) => {
            info!(error=?e, "An error occured while running tasks");
            rpc_server_task.abort();
        }
    }
}
