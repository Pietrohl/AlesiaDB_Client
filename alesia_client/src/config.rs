use std::io::Error;
use url::Url;

use crate::client::AlesiaClient;

#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) path: String,
}

impl Config {
    pub fn from_str(url: &str) -> Result<Self, Box<Error>> {
        if let Err(parsed) = Url::parse(url) {
            return Err(Box::new(Error::new(
                std::io::ErrorKind::InvalidData,
                parsed.to_string(),
            )));
        };

        Ok(Self {
            path: url.to_string(),
        })
    }

    pub async fn connect(self) -> AlesiaClient {
        AlesiaClient::create(self).await
    }
}
