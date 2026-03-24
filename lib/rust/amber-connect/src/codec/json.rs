use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use zeromq::{Socket, SocketRecv, SocketSend, ZmqError};

pub trait JsonSender {
    fn send_json<Msg: Serialize + Sync>(
        &mut self,
        msg: &Msg,
    ) -> impl Future<Output = Result<(), JsonSocketError>> + Send;
}

pub trait JsonReceiver {
    fn recv_json<Msg: DeserializeOwned>(&mut self) -> impl Future<Output = Result<Msg, JsonSocketError>> + Send;
}

#[derive(Debug, Error)]
pub enum JsonSocketError {
    #[error(transparent)]
    Socket(#[from] ZmqError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error("received an empty frame")]
    EmptyFrame,
}

impl<T: Socket + SocketSend> JsonSender for T {
    async fn send_json<Msg: Serialize + Sync>(&mut self, msg: &Msg) -> Result<(), JsonSocketError> {
        self.send(serde_json::to_string(&msg)?.into()).await.map_err(Into::into)
    }
}

impl<T: Socket + SocketRecv> JsonReceiver for T {
    async fn recv_json<Msg: DeserializeOwned>(&mut self) -> Result<Msg, JsonSocketError> {
        let msg = self.recv().await?;
        let frame = msg.get(0).ok_or(JsonSocketError::EmptyFrame)?;
        serde_json::from_slice(frame).map_err(Into::into)
    }
}
