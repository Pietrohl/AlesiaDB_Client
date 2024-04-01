use crate::{
    config::Config,
    errors::{AlesiaError, Error},
    types::{
        self,
        dto::{QueryType, ResponseDTO},
        structs::{TableRow, ToColumnDate},
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct AlesiaClient {
    connection: TcpStream,
}

impl AlesiaClient {
    pub(crate) async fn create(config: Config) -> Result<AlesiaClient, Error> {
        let connection = TcpStream::connect(config.path)
            .await
            .map_err(|e: std::io::Error| Error::IoError(AlesiaError(e.into())))?;
        Ok(AlesiaClient { connection })
    }

    pub async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn ToColumnDate + Sync)],
    ) -> Result<Vec<TableRow>, Error> {
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
    ) -> Result<usize, Error> {
        match self.send_request(query, params, QueryType::EXEC).await {
            Ok(response) => Ok(response.rows_affected),
            Err(e) => Err(e),
        }
    }

    pub async fn insert(
        &mut self,
        query: &str,
        params: &[&(dyn ToColumnDate + Sync)],
    ) -> Result<(), Error> {
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
    ) -> Result<ResponseDTO, Error> {
        let query = types::dto::RequestDTO {
            query_type: query_type,
            query: query.to_string(),
            params: params.iter().map(|p| p.to_sql()).collect(),
        };

        let message = serde_json::to_vec(&query)
            .map_err(|e: serde_json::Error| Error::invalid_query(AlesiaError(e.into())))?;

        self.connection
            .write_all(&message)
            .await
            .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
        self.connection
            .flush()
            .await
            .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

        let mut buffer = [0; 20480];
        let n: usize = self
            .connection
            .read(&mut buffer)
            .await
            .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

        let response: ResponseDTO = serde_json::from_slice(&buffer[..n])
            .map_err(|e: serde_json::Error| Error::IoError(AlesiaError(e.into())))?;

        Ok(response)
    }
}
