use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use types::dto::ResponseDTO;
use types::structs::ToSql;
pub mod types;

pub struct AlesiaClient {
    connection: TcpStream,
}

impl AlesiaClient {
    pub async fn new_from_url(url: &str) -> AlesiaClient {
        let connection = TcpStream::connect(url).await.unwrap();
        AlesiaClient { connection }
    }

    pub async fn send_query(
        &mut self,
        query: &str,
        params:  &[&(dyn ToSql + Sync)],
    ) -> Result<ResponseDTO, Box<dyn std::error::Error>> {
        let query = types::dto::QueryDTO {
            query: query.to_string(),
            params: params.iter().map(|p| p.to_sql()).collect(),
        };

        let message = serde_json::to_vec(&query).unwrap();

        self.connection.write_all(&message).await.unwrap();
        self.connection.write_all(b"\n").await.unwrap();

        let mut buffer = [0; 1024];
        let n: usize = self.connection.read(&mut buffer).await.unwrap();

        let response: ResponseDTO = serde_json::from_slice(&buffer[..n]).unwrap();

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_send_query() {
        let runtime: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut client = AlesiaClient::new_from_url("127.0.0.1:8080").await;

            let query = "SELECT * FROM users;";
            let params: Vec<&(dyn ToSql + Sync)> = vec![];

            let result = client.send_query(query, &params).await;
            assert!(result.is_ok());
        });
    }
}
