use crate::{config::Config, types::{
    self,
    dto::{QueryType, ResponseDTO},
    structs::{TableRow, ToColumnDate},
}};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct AlesiaClient {
    connection: TcpStream,
}

impl AlesiaClient {
    pub (crate) async fn create(config: Config) -> AlesiaClient {
        let connection = TcpStream::connect(config.path).await.unwrap();
        AlesiaClient { connection }
    }

    pub async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn ToColumnDate + Sync)],
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
        params: &[&(dyn ToColumnDate + Sync)],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        match self.send_request(query, params, QueryType::EXEC).await {
            Ok(response) => Ok(response.rows_affected),
            Err(e) => Err(e),
        }
    }

    pub async fn insert(
        &mut self,
        query: &str,
        params: &[&(dyn ToColumnDate + Sync)],
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.send_request(query, params, QueryType::INSERT).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn send_request(
        &mut self,
        query: &str,
        params: &[&(dyn ToColumnDate + Sync)],
        query_type: QueryType,
    ) -> Result<ResponseDTO, Box<dyn std::error::Error>> {
        let query = types::dto::RequestDTO {
            query_type: query_type,
            query: query.to_string(),
            params: params.iter().map(|p| p.to_sql()).collect(),
        };

        let message = serde_json::to_vec(&query)?;

        self.connection.write_all(&message).await?;
        self.connection.write_all(b"\n").await?;
        self.connection.flush().await?;

        let mut buffer = [0; 20480];
        let n: usize = self.connection.read(&mut buffer).await?;

        let response: ResponseDTO = serde_json::from_slice(&buffer[..n])?;

        Ok(response)
    }
}
