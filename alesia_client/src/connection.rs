use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

use crate::{
    errors::{AlesiaError, Error},
    types::dto::{QueryType, RequestDTO, ResponseDTO},
};

pub(crate) async fn write_message<Output: Deserialize<Output = Output>>(
    message: &impl Serialize,
    connection: &mut CodecConnection,
) -> Result<Output, Error> {
    message.serialize(connection).await?;

    let reponse_message: Output = Output::deserialize(connection).await?;
    Ok(reponse_message)
}

pub struct CodecConnection {
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
}

impl CodecConnection {
    pub async fn new(connection: TcpStream) -> Self {
        let (reader, writer) = connection.into_split();
        Self { reader, writer }
    }

    async fn send_message(
        &mut self,
        message_type: MessageCode,
        message_data: &[u8],
    ) -> Result<(), Error> {
        self.writer
            .write_u8(message_type.into())
            .await
            .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
        self.writer
            .write_u32(message_data.len() as u32)
            .await
            .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
        self.writer
            .write_all(message_data)
            .await
            .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

        self.writer
            .flush()
            .await
            .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

        Ok(())
    }

    async fn read_message(&mut self) -> Result<(MessageCode, Vec<u8>), Error> {
        let message_code = self
            .reader
            .read_u8()
            .await
            .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

        let message_length =
            self.reader
                .read_u16()
                .await
                .map_err(|e| Error::IoError(AlesiaError(e.into())))? as usize;

        let mut message = vec![0; message_length];

        self.reader
            .read_exact(&mut message)
            .await
            .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

        match MessageCode::from(message_code) {
            MessageCode::Error => {
                let error_message = String::from_utf8(message.to_vec())
                    .map_err(|e| Error::IoError(AlesiaError(e.into())))?;
                Err(Error::io(AlesiaError(error_message.into())))
            }
            _ => Ok((MessageCode::from(message_code), message)),
        }
    }
}

pub trait Serialize {
    fn serialize(
        &self,
        connection: &mut CodecConnection,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}
pub trait Deserialize {
    type Output;
    fn deserialize(
        connection: &mut CodecConnection,
    ) -> impl std::future::Future<Output = Result<Self::Output, Error>> + Send;
}

impl Serialize for RequestDTO {
    async fn serialize(&self, connection: &mut CodecConnection) -> Result<(), Error> {
        let message =
            &serde_json::to_vec(self).map_err(|e| Error::IoError(AlesiaError(e.into())))?;

        connection.send_message(self.into(), &message).await
    }
}

impl Deserialize for ResponseDTO {
    type Output = ResponseDTO;

    async fn deserialize(connection: &mut CodecConnection) -> Result<Self::Output, Error> {
        let (code, message) = connection.read_message().await?;

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
    Error,
}

impl From<MessageCode> for u8 {
    fn from(code: MessageCode) -> u8 {
        match code {
            MessageCode::RequestQuery => 1,
            MessageCode::RequestExec => 2,
            MessageCode::RequestInsert => 3,
            MessageCode::SuccessResponse => 4,
            MessageCode::Error => 5,
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
            5 => MessageCode::Error,
            _ => panic!("Invalid message code"),
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
