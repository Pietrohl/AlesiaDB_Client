use alesia_client::{client::AlesiaClient, errors::Error};

pub struct AlesiaConnectionManager {
    config: alesia_client::config::Config,
}

impl AlesiaConnectionManager {
    pub fn new(config: alesia_client::config::Config) -> Self {
        Self { config }
    }

    pub fn new_from_url(url: &str) -> Result<Self, Box<Error>> {
        let config = alesia_client::config::Config::from_str(url)?;

        Ok(Self { config })
    }
}

#[async_trait::async_trait]
impl bb8::ManageConnection for AlesiaConnectionManager {
    type Connection = AlesiaClient;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let client = self.config.clone().connect().await;
        client
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        tokio::task::block_in_place(|| async move { conn.query("SELECT 1;", &[]).await }).await?;
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}
