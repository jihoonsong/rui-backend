use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: SocketAddr,
    pub package_id: String,
    pub group_id: String,
    pub keystore_path_relative: String,
}
