use crate::{
    config::Config,
    connection::{self},
    errors::{AlesiaError, Error},
    types::{
        self,
        dto::{QueryType, ResponseDTO},
        structs::{TableRow, ToColumnDate},
    },
};
use tokio::net::TcpStream;

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

        let response_message = connection::write_message(&message, &mut self.connection).await?;

        let response: ResponseDTO = serde_json::from_str(response_message.as_str())
            .map_err(|e: serde_json::Error| Error::IoError(AlesiaError(e.into())))?;

        Ok(response)
    }

    // This can be implemented to close the connection
    // pub async fn close(&mut self) -> Result<(), Error> {
    //     self.connection
    //         .shutdown()
    //         .await
    //         .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
    //     Ok(())
    // }

    // This will be used by bb8 to check if the connection is still valid
    // The connection is valid if the TCP connection is still running
    // and a ping message would have a correct pong response
    // pub async fn is_close(&mut self) -> Result<bool, Error> {
    //     Ok(false)
    // }
}
