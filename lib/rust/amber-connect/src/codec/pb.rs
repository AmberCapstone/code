use prost::Message;
use thiserror::Error;
use zeromq::{Socket, SocketRecv, SocketSend, ZmqError};

pub trait PbReceiver {
    fn recv_msg<Msg: Message + Default>(&mut self) -> impl Future<Output = Result<Msg, PbSocketError>> + Send;
}

pub trait PbSender {
    fn send_msg<Msg: Message>(&mut self, msg: &Msg) -> impl Future<Output = Result<(), PbSocketError>> + Send;
}

#[derive(Debug, Error)]
pub enum PbSocketError {
    #[error(transparent)]
    Socket(#[from] ZmqError),

    #[error("invalid protobuf: {0}")]
    InvalidProto(#[from] prost::DecodeError),

    #[error("received an empty frame")]
    EmptyFrame,
}

impl<T: Socket + SocketRecv> PbReceiver for T {
    async fn recv_msg<Msg: Message + Default>(&mut self) -> Result<Msg, PbSocketError> {
        let frame = self.recv().await?;
        let bytes = frame.get(0).ok_or(PbSocketError::EmptyFrame)?;
        Ok(Msg::decode(bytes.as_ref())?)
    }
}

impl<T: Socket + SocketSend> PbSender for T {
    async fn send_msg<Msg: Message>(&mut self, msg: &Msg) -> Result<(), PbSocketError> {
        self.send(msg.encode_to_vec().into()).await.map_err(Into::into)
    }
}
