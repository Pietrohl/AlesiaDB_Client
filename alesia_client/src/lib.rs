use config::Config;
use errors::Error;
pub mod client;
pub mod config;
pub mod errors;
pub mod connection;
pub mod types;

pub async fn new_from_url(url: &str) -> Result<client::AlesiaClient, Error> {
    let config = Config::from_str(url)?;
    client::AlesiaClient::create(config).await
}
