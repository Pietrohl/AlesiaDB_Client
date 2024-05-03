use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    errors::{AlesiaError, Error},
    types::dto::{QueryType, RequestDTO, ResponseDTO},
};

pub(crate) async fn write_message<Output: Deserialize<Output = Output>>(
    message: &impl Serialize,
    connection: &mut TcpStream,
) -> Result<Output, Error> {
    message.serialize(connection).await?;

    let reponse_message: Output = Output::deserialize(connection).await?;
    Ok(reponse_message)
}

async fn send_message<T: AsyncWriteExt>(
    writer: &mut T,
    message_type: MessageCode,
    message_data: &[u8],
) -> Result<(), Error>
where
    T: Unpin,
{
    writer
        .write_u8(message_type.into())
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
    writer
        .write_u32(message_data.len() as u32)
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
    writer
        .write_all(message_data)
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

    writer
        .flush()
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

    Ok(())
}

async fn read_message<T: AsyncReadExt>(reader: &mut T) -> Result<(MessageCode, Vec<u8>), Error>
where
    T: Unpin,
{
    let message_code = reader
        .read_u8()
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

    let message_length = reader
        .read_u32()
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))? as usize;

    let mut message = vec![0; message_length];

    reader
        .read_exact(&mut message)
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

    match MessageCode::from(message_code) {
        MessageCode::Error(n) => {
            let error_message = String::from_utf8(message.to_vec())
                .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

            let err = match n {
                52 => Error::config(AlesiaError(error_message.into())),
                53 => Error::invalid_query(AlesiaError(error_message.into())),
                54 => Error::RusqliteError(rusqlite::Error::QueryReturnedNoRows),
                55 => Error::RusqliteError(rusqlite::Error::ExecuteReturnedResults),
                _ => Error::io(AlesiaError(error_message.into())),
            };

            Err(err)
        }
        _rest_codes => Ok((_rest_codes, message)),
    }
}

#[async_trait::async_trait]
pub trait Serialize {
    async fn serialize<T: AsyncWriteExt>(&self, connection: &mut T) -> Result<(), Error>
    where
        T: std::marker::Unpin + std::marker::Send + std::marker::Sync;
}
#[async_trait::async_trait]
pub trait Deserialize {
    type Output;
    async fn deserialize<T: AsyncReadExt>(connection: &mut T) -> Result<Self::Output, Error>
    where
        T: std::marker::Unpin + std::marker::Send + std::marker::Sync;
}

#[async_trait::async_trait]
impl Serialize for RequestDTO {
    async fn serialize<T: AsyncWriteExt>(&self, connection: &mut T) -> Result<(), Error>
    where
        T: std::marker::Unpin + std::marker::Send + std::marker::Sync,
    {
        let message =
            &serde_json::to_vec(self).map_err(|e| Error::IoError(AlesiaError(e.into())))?;

        send_message(connection, self.into(), &message).await
    }
}

#[async_trait::async_trait]
impl Serialize for ResponseDTO {
    async fn serialize<T: AsyncWriteExt>(&self, connection: &mut T) -> Result<(), Error>
    where
        T: std::marker::Unpin + std::marker::Send + std::marker::Sync,
    {
        let message =
            &serde_json::to_vec(self).map_err(|e| Error::IoError(AlesiaError(e.into())))?;

        send_message(connection, self.into(), &message).await
    }
}

#[async_trait::async_trait]
impl Serialize for Error {
    async fn serialize<T: AsyncWriteExt>(&self, connection: &mut T) -> Result<(), Error>
    where
        T: std::marker::Unpin + std::marker::Send + std::marker::Sync,
    {
        let error_message: &str = &self.to_string();

        send_message(connection, self.into(), error_message.as_bytes()).await
    }
}

#[async_trait::async_trait]
impl Deserialize for RequestDTO {
    type Output = RequestDTO;

    async fn deserialize<T: AsyncReadExt>(connection: &mut T) -> Result<Self::Output, Error>
    where
        T: std::marker::Unpin + std::marker::Send + std::marker::Sync,
    {
        let (code, message) = read_message(connection).await?;

        match code {
            MessageCode::RequestQuery | MessageCode::RequestExec | MessageCode::RequestInsert => {
                let request: RequestDTO = serde_json::from_slice(&message)
                    .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
                Ok(request)
            }
            _ => {
                let error_message = String::from_utf8(message.to_vec())
                    .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
                Err(Error::io(AlesiaError(error_message.into())))
            }
        }
    }
}

#[async_trait::async_trait]
impl Deserialize for ResponseDTO {
    type Output = ResponseDTO;

    async fn deserialize<T: AsyncReadExt>(connection: &mut T) -> Result<Self::Output, Error>
    where
        T: std::marker::Unpin + std::marker::Send + std::marker::Sync,
    {
        let (code, message) = read_message(connection).await?;

        match code {
            MessageCode::SuccessResponse => {
                let response: ResponseDTO = serde_json::from_slice(&message)
                    .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
                Ok(response)
            }
            _ => {
                let error_message = String::from_utf8(message.to_vec())
                    .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
                Err(Error::io(AlesiaError(error_message.into())))
            }
        }
    }
}

enum MessageCode {
    RequestQuery,
    RequestExec,
    RequestInsert,
    SuccessResponse,
    Error(u8),
}

impl From<MessageCode> for u8 {
    fn from(code: MessageCode) -> u8 {
        match code {
            MessageCode::RequestQuery => 1,
            MessageCode::RequestExec => 2,
            MessageCode::RequestInsert => 3,
            MessageCode::SuccessResponse => 4,
            MessageCode::Error(n) => n,
        }
    }
}

impl From<u8> for MessageCode {
    fn from(code: u8) -> Self {
        match code {
            1 => MessageCode::RequestQuery,
            2 => MessageCode::RequestExec,
            3 => MessageCode::RequestInsert,
            4 => MessageCode::SuccessResponse,
            n => MessageCode::Error(n),
        }
    }
}

impl From<&RequestDTO> for MessageCode {
    fn from(request: &RequestDTO) -> Self {
        match request.query_type {
            QueryType::QUERY => MessageCode::RequestQuery,
            QueryType::EXEC => MessageCode::RequestExec,
            QueryType::INSERT => MessageCode::RequestInsert,
        }
    }
}

impl From<&ResponseDTO> for MessageCode {
    fn from(_self: &ResponseDTO) -> Self {
        MessageCode::SuccessResponse
    }
}

impl From<&Error> for MessageCode {
    // Starting Error codes from 51 to avoid conflict with other message codes
    fn from(error: &Error) -> Self {
        match error {
            Error::IoError(_) => MessageCode::Error(51),
            Error::ConfigError(_) => MessageCode::Error(52),
            Error::InvalidQuery(_) => MessageCode::Error(53),
            Error::RusqliteError(rusqlite_error) => match rusqlite_error {
                rusqlite::Error::QueryReturnedNoRows => MessageCode::Error(54),
                rusqlite::Error::ExecuteReturnedResults => MessageCode::Error(55),
                _ => MessageCode::Error(56),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, str::from_utf8, vec};

    use crate::errors;

    use super::*;

    #[tokio::test]
    async fn test_request_roundtrip() {
        let request = RequestDTO {
            query: "SELECT * FROM test_table".to_string(),
            query_type: QueryType::QUERY,
            params: vec![],
        };

        let mut buffer = vec![];
        request.serialize(&mut buffer).await.unwrap();

        let mut reader = Cursor::new(buffer);
        let mut reader_clone = reader.clone();

        dbg!(&reader);

        let messsage_code = reader_clone.read_u8().await.unwrap();
        let length = reader_clone.read_u32().await.unwrap();

        println!("Code: {}", &messsage_code);
        println!("Length: {}", &length);

        let mut message = vec![0u8; length as usize];
        tokio::io::AsyncReadExt::read_exact(&mut reader_clone, &mut message)
            .await
            .unwrap();

        dbg!(reader_clone);
        dbg!(from_utf8(message.as_slice()).unwrap());

        let deserialized_request = RequestDTO::deserialize(&mut reader).await.unwrap();

        dbg!(&deserialized_request);

        assert_eq!(request.query, deserialized_request.query);
        assert_eq!(request.query_type, deserialized_request.query_type);
        assert_eq!(request.params, deserialized_request.params);
    }

    #[tokio::test]
    async fn test_response_roundtrip() {
        let response = ResponseDTO {
            status: "OK".to_string(),
            rows_affected: 1,
            rows: vec![],
            column_count: 0,
            column_names: vec![],
        };

        let mut buffer = vec![];
        response.serialize(&mut buffer).await.unwrap();

        let mut reader = Cursor::new(buffer);
        let deserialized_response = ResponseDTO::deserialize(&mut reader).await.unwrap();

        assert_eq!(response.status, deserialized_response.status);
        assert_eq!(response.rows_affected, deserialized_response.rows_affected);
        assert_eq!(response.rows, deserialized_response.rows);
        assert_eq!(response.column_count, deserialized_response.column_count);
        assert_eq!(response.column_names, deserialized_response.column_names);
    }

    #[cfg(test)]
    mod tests_deserialize_error_response {
        use super::*;

        #[tokio::test]
        async fn test_deserialize_io_error_response() {
            let io_error = Error::io(errors::AlesiaError("Error message".to_string().into()));
            let mut buffer = vec![];
            io_error.serialize(&mut buffer).await.unwrap();

            let mut reader = Cursor::new(buffer);

            let response_error = ResponseDTO::deserialize(&mut reader).await;
            assert!(response_error.is_err());
        }
        #[tokio::test]
        async fn test_deserialize_config_error() {
            let config_error = Error::ConfigError(errors::AlesiaError(
                "Config error message".to_string().into(),
            ));
            let mut buffer = vec![];
            config_error.serialize(&mut buffer).await.unwrap();

            let mut reader = Cursor::new(buffer);

            let response_error = ResponseDTO::deserialize(&mut reader).await;

            assert!(response_error.is_err());
            assert_eq!(
                response_error.err().unwrap().to_string(),
                "Config error message"
            );
        }

        #[tokio::test]
        async fn test_deserialize_invalid_query_error() {
            let invalid_query_error = Error::InvalidQuery(errors::AlesiaError(
                "Invalid query error message".to_string().into(),
            ));
            let mut buffer = vec![];
            invalid_query_error.serialize(&mut buffer).await.unwrap();

            let mut reader = Cursor::new(buffer);

            let response_error = ResponseDTO::deserialize(&mut reader).await;

            assert!(response_error.is_err());
            assert_eq!(
                response_error.err().unwrap().to_string(),
                "Invalid query error message"
            );
        }

        #[tokio::test]
        async fn test_deserialize_rusqlite_error() {
            let rusqlite_error = Error::RusqliteError(rusqlite::Error::InvalidQuery);
            let mut buffer = vec![];
            rusqlite_error.serialize(&mut buffer).await.unwrap();

            let mut reader = Cursor::new(buffer);

            let response_error = ResponseDTO::deserialize(&mut reader).await;

            assert!(response_error.is_err());
            assert_eq!(
                response_error.err().unwrap().to_string(),
                "Query is not read-only"
            );
        }
    }
}
