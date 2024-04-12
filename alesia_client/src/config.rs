use std::net::{SocketAddr, ToSocketAddrs};

use crate::{client::AlesiaClient, errors};

#[derive(Clone)]
pub struct Config {
    pub(crate) path: SocketAddr,
}

impl Config {
    pub fn from_str(url: &str) -> Result<Self, errors::Error> {
        let addrs: String = url.to_string();

        let mut addrs_iter = (&addrs[..])
            .to_socket_addrs()
            .map_err(|_| errors::Error::ConfigError("Invalid URL".into()))?;

        let path = addrs_iter.next().unwrap();
        Ok(Self { path })
    }

    pub async fn connect(self) -> Result<AlesiaClient, errors::Error> {
        AlesiaClient::create(self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_str_valid_url() {
        let url = "www.example.com:80";
        let config = Config::from_str(url);
        assert!(config.is_ok());
    }

    #[test]
    fn test_valid_local_url() {
        let url = "localhost:8000";
        let config = Config::from_str(url).unwrap();
        assert_eq!(config.path.to_string(), "[::1]:8000");
    }
    #[test]
    fn test_valid_local_socket_addrs() {
        let url = "127.0.0.1:8080";
        let config = Config::from_str(url).unwrap();
        assert_eq!(config.path.to_string(), url);
    }

    #[test]
    fn test_valid_local_zero_socket_addrs() {
        let url = "0.0.0.0:8080";
        let config = Config::from_str(url).unwrap();
        assert_eq!(config.path.to_string(), url);
    }

    #[test]
    fn test_config_from_str_invalid_url() {
        let url = "invalid_url";
        let result = Config::from_str(url);
        assert!(result.is_err());
    }
}
