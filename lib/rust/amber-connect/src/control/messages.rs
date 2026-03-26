use std::time::SystemTime;

use super::lease::{Hold, LeaseHeld, Token, WrongToken};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Request<T> {
    Acquire { name: String },
    Release { token: Token },
    Renew { token: Token },
    Send { token: Token, item: T },
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Acquire(Result<AcquireResponse, LeaseHeld>),
    Release(Result<(), WrongToken>),
    Renew(Result<SystemTime, WrongToken>),
    Send(Result<(), WrongToken>),
    InvalidRequest,
}

#[derive(Serialize, Deserialize)]
pub struct AcquireResponse {
    pub token: Token,
    pub expiry: SystemTime,
}

impl From<Hold> for AcquireResponse {
    fn from(value: Hold) -> Self {
        Self {
            token: value.token,
            expiry: value.expiry,
        }
    }
}
