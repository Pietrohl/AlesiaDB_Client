use std::str::from_utf8;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::errors::{AlesiaError, Error};

#[derive(Debug)]
pub struct BackendMessage(String);

impl BackendMessage {
    fn read_from_slice<'d>(slice: &'d [u8]) -> Result<BackendMessage, Error> {
        match slice.split_first() {
            Some((&b'S', message)) => {
                let one_str = String::from_utf8(message.into()).map_err(|_| {
                    Error::IoError(AlesiaError(
                        format!("Invalid UTF-8 sequence: {:?}", message).into(),
                    ))
                })?;

                return Ok(Self(one_str));
            }
            Some((&b'E', message)) => {
                return Err(Error::InvalidQuery(AlesiaError(
                    format!("{}", from_utf8(message).unwrap_or("Invalid Query")).into(),
                )))
            }
            _ => {
                return Err(Error::IoError(AlesiaError(
                    format!("Invalid UTF-8 sequence: {:?}", from_utf8(slice).unwrap()).into(),
                )));
            }
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn error(error: Error) -> BackendMessage {
        BackendMessage::from(&error)
    }

    pub fn success(mut data: String) -> BackendMessage {
        data.insert(0, 'S');
        BackendMessage(data)
    }
}

impl From<&Error> for BackendMessage {
    fn from(e: &Error) -> Self {
        BackendMessage(format!("E{}", e))
    }
}

impl Into<Error> for BackendMessage {
    fn into(self) -> Error {
        Error::InvalidQuery(AlesiaError(self.0.into()))
    }
}

pub(crate) async fn write_message(
    message: &[u8],
    connection: &mut TcpStream,
) -> Result<String, Error> {
    let mut buffer = [0; 20480];

    connection
        .write_all(message)
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

    connection
        .flush()
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

    let n: usize = connection
        .read(&mut buffer)
        .await
        .map_err(|e| Error::IoError(AlesiaError(e.into())))?;

    let reponse_message: BackendMessage = BackendMessage::read_from_slice(&buffer[..n])?;

    Ok(reponse_message.0)
}
