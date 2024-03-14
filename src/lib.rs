use config::Config;
pub mod client;
pub mod config;
// pub mod connection;
pub mod types;

pub async fn new_from_url(url: &'static str) -> client::AlesiaClient {
    let config = Config { path: url };
    client::AlesiaClient::create(config).await
}


