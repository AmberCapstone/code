use prost::Message;
use zeromq::{Socket, SocketRecv, SocketSend, ZmqResult};

pub trait ZmqMsgReceiver {
    fn recv_msg<Msg: Message + Default>(&mut self) -> impl std::future::Future<Output = ZmqResult<Msg>> + Send;
}

pub trait ZmqMsgSender {
    fn send_msg<Msg: Message>(&mut self, msg: &Msg) -> impl std::future::Future<Output = ZmqResult<()>> + Send;
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
