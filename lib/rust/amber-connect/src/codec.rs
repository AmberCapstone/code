use prost::Message;
use serde::{Serialize, de::DeserializeOwned};
use zeromq::{Socket, SocketRecv, SocketSend, ZmqError, ZmqResult};

pub trait ZmqMsgReceiver {
    fn recv_msg<Msg: Message + Default>(&mut self) -> impl Future<Output = ZmqResult<Msg>> + Send;
}

pub trait ZmqMsgSender {
    fn send_msg<Msg: Message>(&mut self, msg: &Msg) -> impl Future<Output = ZmqResult<()>> + Send;
}

pub trait JsonSender {
    fn send_json<Msg: Serialize + Sync>(
        &mut self,
        msg: &Msg,
    ) -> impl Future<Output = Result<(), JsonSocketError>> + Send;
}

pub trait JsonReceiver {
    fn recv_json<Msg: DeserializeOwned>(&mut self) -> impl Future<Output = Result<Msg, JsonSocketError>> + Send;
}

impl<T: Socket + SocketRecv> ZmqMsgReceiver for T {
    async fn recv_msg<Msg: Message + Default>(&mut self) -> ZmqResult<Msg> {
        let frame = self.recv().await?;
        let bytes = frame.get(0).unwrap();
        Ok(Msg::decode(bytes.as_ref()).unwrap())
    }
}
impl<T: Socket + SocketSend> ZmqMsgSender for T {
    async fn send_msg<Msg: Message>(&mut self, msg: &Msg) -> ZmqResult<()> {
        self.send(msg.encode_to_vec().into()).await
    }
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

#[derive(Debug)]
pub enum JsonSocketError {
    Zmq(ZmqError),
    Serde(serde_json::Error),
    EmptyFrame,
}

impl From<ZmqError> for JsonSocketError {
    fn from(val: ZmqError) -> Self {
        JsonSocketError::Zmq(val)
    }
}

impl From<serde_json::Error> for JsonSocketError {
    fn from(val: serde_json::Error) -> Self {
        JsonSocketError::Serde(val)
    }
}
