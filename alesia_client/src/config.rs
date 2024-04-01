use url::Url;

use crate::{client::AlesiaClient, errors::Error};

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) path: String,
}

impl Config {
    pub fn from_str(url: &str) -> Result<Self, Error> {
        if !is_url(url) && !is_ip_addr(url) {
            return Err(Error::ConfigError("Invalid URL".into()));
        }

        Ok(Self {
            path: url.to_string(),
        })
    }

    pub async fn connect(self) -> Result<AlesiaClient, Error> {
        AlesiaClient::create(self).await
    }
}

fn is_url(url: &str) -> bool {
    Url::parse(url).is_ok()
}

fn is_ip_addr(url: &str) -> bool {
    let mut url_parts = url.rsplitn(2, ':');
    let tcp_port = url_parts.next();
    let ip = url_parts.next();

    if None == tcp_port || None == ip {
        return false;
    }

    let ip_parts: Vec<&str> = ip.unwrap().split('.').collect();

    if ip_parts.len() != 4 {
        return false;
    }

    for part in ip_parts {
        if let Err(_) = part.parse::<u8>() {
            return false;
        }
    }

    if let Err(_) = tcp_port.unwrap().parse::<u16>() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

     #[test]
     fn test_config_from_str_valid_url() {
         let url = "https://example.com";
         let config = Config::from_str(url).unwrap();
         assert_eq!(config.path, url);
     }
    
      #[test]
      fn test_valid_local_url() {
          let url = "http://localhost:8000";
          let config = Config::from_str(url).unwrap();
          assert_eq!(config.path, url);
          let url = "127.0.0.1:8080";
          let config = Config::from_str(url).unwrap();
          assert_eq!(config.path, url);
      }
    
      #[test]
      fn test_config_from_str_invalid_url() {
          let url = "invalid_url";
          let result = Config::from_str(url);
          assert!(result.is_err());
      }
}
