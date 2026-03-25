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

    /// Attempt to acquire the lease.
    ///
    /// # Returns
    ///
    /// `(Token, SystemTime)`
    /// `Token` is the unique token that this process can use to exercise the lease
    /// `SystemTime` is the token's expiration time.
    ///
    /// Call `lease.renew()` to extend the expiration.
    ///
    /// # Errors
    ///
    /// `LeaseHeld` if the lease is held by an unexpired token.
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

    /// Renew the lease for this token, provided the token holds the lease.
    ///
    /// # Errors
    ///
    /// `WrongToken` if this token does not hold the lease.
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

    /// Withdraw the lease for this token, provided the token holds the lease.
    ///
    /// A nice client will call `.withdraw` so that other clients can acquire the lease sooner,
    /// rather than waiting for the token to expire.
    ///
    /// # Errors
    ///
    /// `WrongToken` if this token does not hold the lease.
    pub fn withdraw(&mut self, token: Token) -> Result<(), WrongToken> {
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

#[cfg(test)]
mod tests {
    use std::{arch::x86_64::_MM_EXCEPT_INVALID, thread::sleep};

    use super::*;

    const LIFETIME: Duration = Duration::from_millis(20);

    #[test]
    fn test_renew() {
        let mut lease = Lease::new(LIFETIME);

        let (token, mut expiry) = lease.acquire().expect("lease is available");

        for _ in 0..5 {
            sleep(LIFETIME * 10 / 11);
            expiry = lease.renew(token).expect("lease has not expired");
        }

        sleep(LIFETIME * 11 / 10);
        assert!(lease.renew(token).is_err(), "lease has expired");
        assert!(lease.withdraw(token).is_err(), "cannot release an unowned lease");
    }

    #[test]
    fn test_owned_by() {
        let mut lease = Lease::new(LIFETIME);

        let wrong_token = Token(0);
        assert!(!lease.is_owned_by(wrong_token));

        let (token, _) = lease.acquire().expect("lease is available");
        assert!(lease.is_owned_by(token));
        lease.withdraw(token).expect("token owns the lease");
        assert!(!lease.is_owned_by(token));
    }
}
