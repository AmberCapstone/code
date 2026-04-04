use std::time::{Duration, SystemTime};

use thiserror::Error;
use tokio::time::timeout;
use zeromq::{ReqSocket, Socket, ZmqError};

use super::{
    lease::{LeaseHeld, Token, WrongToken},
    messages::{Request, Response},
};
use crate::{
    codec::{JsonReceiver, JsonSender, JsonSocketError},
    endpoint,
};
use proto::sensor::Command;

type Req = Request<Command>;

macro_rules! extract_response {
    ($resp:expr, $variant:path, $inner:pat => $ok:expr) => {
        match $resp {
            $variant($inner) => $ok,
            Response::InvalidRequest => Err(ClientError::InvalidRequest),
            _ => Err(ClientError::WrongResponse),
        }
    };
}

pub struct Client {
    socket: ReqSocket,
    token: Token,
    expiry: SystemTime,
}

impl Client {
    /// # Errors
    ///
    /// Socket error on send or receive.
    /// Client could not obtain lease
    /// Invalid JSON
    /// Unexpected response
    pub async fn try_acquire(name: &str) -> Result<Self, ClientError> {
        let mut socket = ReqSocket::new();

        timeout(Duration::from_secs(1), socket.connect(endpoint::COMMAND))
            .await
            .map_err(|_| ClientError::Timeout)??;
        socket
            .send_json::<Req>(&Request::Acquire { name: name.to_string() })
            .await?;

        let resp = socket.recv_json::<Response>().await?;
        extract_response!(resp, Response::Acquire, acq => match acq {
            Ok(a) => Ok(Self{socket, token: a.token, expiry: a.expiry}),
            Err(e) => Err(e.into())
        })
    }

    pub fn expiry(&self) -> SystemTime {
        self.expiry
    }

    /// # Errors
    ///
    /// Socket error on send or receive.
    /// Client does not hold lease
    /// Invalid JSON
    /// Unexpected response
    pub async fn send(&mut self, command: Command) -> Result<(), ClientError> {
        self.socket
            .send_json(&Request::Send {
                token: self.token,
                item: command,
            })
            .await?;

        let resp = self.socket.recv_json::<Response>().await?;
        extract_response!(resp, Response::Send, r => r.map_err(Into::into))
    }

    /// Repeatedly send a message at some interval.
    ///
    /// Useful with `tokio::select` to send a command until some condition is met
    ///
    /// # Errors
    ///
    /// Socket error on send or receive.
    /// Client does not hold lease
    /// Invalid JSON
    /// Unexpected response
    pub async fn send_continuous(&mut self, command: Command, interval: Duration) -> Result<(), ClientError> {
        let mut ticker = tokio::time::interval(interval);

        loop {
            self.send(command.clone()).await?;
            ticker.tick().await;
        }
    }

    /// # Errors
    ///
    /// Socket error on send or receive.
    /// Client does not hold lease
    /// Invalid JSON
    /// Unexpected response
    pub async fn renew(&mut self) -> Result<(), ClientError> {
        self.socket
            .send_json::<Req>(&Request::Renew { token: self.token })
            .await?;

        let resp = self.socket.recv_json::<Response>().await?;
        extract_response!(resp, Response::Renew, r => {
            match r {
                Ok(expiry) => {
                    self.expiry = expiry;
                    Ok(())
                }
                Err(e) => Err(e.into())
            }
        })
    }

    pub async fn release(mut self) {
        // Ignore errors since the client is consumed anyways
        let _ = self
            .socket
            .send_json::<Req>(&Request::Release { token: self.token })
            .await;

        self.socket.close().await;
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    LeaseHeld(#[from] LeaseHeld),

    #[error(transparent)]
    WrongToken(#[from] WrongToken),

    #[error("socket error {0}")]
    Socket(#[from] JsonSocketError),

    #[error("the server responded incorrectly")]
    WrongResponse,

    #[error("the client's request was misformed")]
    InvalidRequest,

    #[error("timed out waiting on server response")]
    Timeout,
}

impl From<ZmqError> for ClientError {
    fn from(value: ZmqError) -> Self {
        Self::Socket(JsonSocketError::Socket(value))
    }
}
