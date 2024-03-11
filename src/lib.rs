use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use types::dto::{QueryType, ResponseDTO};
use types::structs::{TableRow, ToSql};
pub mod types;

pub struct AlesiaClient {
    connection: TcpStream,
}

impl AlesiaClient {
    pub async fn new_from_url(url: &str) -> AlesiaClient {
        let connection = TcpStream::connect(url).await.unwrap();
        AlesiaClient { connection }
    }

    pub async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<TableRow>, Box<dyn std::error::Error>> {
        match self.send_request(query, params, QueryType::QUERY).await {
            Ok(response) => Ok(response
                .rows
                .iter()
                .map(|row| TableRow::from(&response, row))
                .collect()),
            Err(e) => Err(e),
        }
    }

    pub async fn exec(
        &mut self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        match self.send_request(query, params, QueryType::EXEC).await {
            Ok(response) => Ok(response.rows_affected),
            Err(e) => Err(e),
        }
    }

    pub async fn insert(
        &mut self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.send_request(query, params, QueryType::INSERT).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn send_request(
        &mut self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
        query_type: QueryType,
    ) -> Result<ResponseDTO, Box<dyn std::error::Error>> {
        let query = types::dto::QueryDTO {
            query_type: query_type,
            query: query.to_string(),
            params: params.iter().map(|p| p.to_sql()).collect(),
        };

        let message = serde_json::to_vec(&query)?;

        self.connection.write_all(&message).await?;
        self.connection.write_all(b"\n").await?;

        let mut buffer = [0; 1024];
        let n: usize = self.connection.read(&mut buffer).await?;

        let response: ResponseDTO = serde_json::from_slice(&buffer[..n])?;

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

            let result = client.query(query, &params).await;
            assert!(result.is_ok());
        });
    }
}
