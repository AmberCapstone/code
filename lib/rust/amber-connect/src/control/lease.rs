use rand::{
    Rng, SeedableRng,
    rngs::{StdRng, SysRng},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    time::{Duration, SystemTime},
};
use thiserror::Error;

pub struct Lease {
    token_lifetime: Duration,
    state: State,
    rng: StdRng,
}

enum State {
    Available,
    Held { token: Token, expiry: SystemTime },
}

impl Lease {
    pub fn new(token_lifetime: Duration) -> Self {
        Self {
            state: State::Available,
            token_lifetime,
            rng: StdRng::try_from_rng(&mut SysRng).unwrap(),
        }
    }

    pub fn acquire(&mut self) -> Result<(Token, SystemTime), LeaseHeld> {
        self.update_expiry();
        match self.state {
            State::Available => {
                let new_token = Token::new(&mut self.rng);
                let expiry = SystemTime::now() + self.token_lifetime;
                self.state = State::Held {
                    token: new_token,
                    expiry,
                };
                Ok((new_token, expiry))
            }
            State::Held {
                token: _, // Do not share this!
                expiry,
            } => Err(LeaseHeld::with_expiry(expiry)),
        }
    }

    pub fn renew(&mut self, token: Token) -> Result<SystemTime, WrongToken> {
        self.update_expiry();
        if self.is_owned_by(token) {
            let new_expiry = SystemTime::now() + self.token_lifetime;
            self.state = State::Held {
                token,
                expiry: new_expiry,
            };
            Ok(new_expiry)
        } else {
            Err(WrongToken)
        }
    }

    pub fn release(&mut self, token: Token) -> Result<(), WrongToken> {
        self.update_expiry();
        if self.is_owned_by(token) {
            self.state = State::Available;
            Ok(())
        } else {
            Err(WrongToken)
        }
    }

    pub fn is_owned_by(&mut self, token_: Token) -> bool {
        self.update_expiry();
        match self.state {
            State::Held { token: t, expiry: _ } => t == token_,
            State::Available => false,
        }
    }

    fn update_expiry(&mut self) {
        if let State::Held { token: _, expiry } = self.state
            && SystemTime::now() > expiry
        {
            self.state = State::Available;
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub struct Token(u64);

impl Token {
    pub fn new(rng: &mut StdRng) -> Self {
        Self(rng.next_u64())
    }
}

#[derive(Debug, Serialize, Deserialize, Error)]
pub struct LeaseHeld {
    pub expiry: SystemTime,
}

impl LeaseHeld {
    fn with_expiry(expiry: SystemTime) -> Self {
        Self { expiry }
    }
}

impl Display for LeaseHeld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sec_until = match self.expiry.duration_since(SystemTime::now()) {
            Ok(d) => d.as_secs_f32(),
            Err(negative_d) => -negative_d.duration().as_secs_f32(),
        };
        write!(f, "lease is already held. expires in {sec_until:.2} seconds")
    }
}

#[derive(Debug, Serialize, Deserialize, Error)]
#[error("this token does not match the lease")]
pub struct WrongToken;
