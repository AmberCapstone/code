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
    Held(Hold),
}

#[derive(Debug, Clone)]
pub struct Hold {
    pub token: Token,
    pub expiry: SystemTime,
    pub owner: String,
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
    pub fn acquire(&mut self, process: &str) -> Result<Hold, LeaseHeld> {
        tracing::debug!(by = process, "lease requested");
        self.check_expiry();
        match &self.state {
            State::Available => {
                let new_token = Token::new(&mut self.rng);
                let expiry = SystemTime::now() + self.token_lifetime;
                let hold = Hold {
                    token: new_token,
                    expiry,
                    owner: process.to_string(),
                };
                self.state = State::Held(hold.clone());
                tracing::info!(
                    owner = process,
                    remaining_s = self.token_lifetime.as_secs(),
                    "lease acquired"
                );
                Ok(hold)
            }
            State::Held(hold) => {
                tracing::info!(to = process, "lease declined");
                Err(LeaseHeld::with_expiry(hold.expiry))
            }
        }
    }

    /// Renew the lease for this token, provided the token holds the lease.
    ///
    /// # Errors
    ///
    /// `WrongToken` if this token does not hold the lease.
    pub fn renew(&mut self, token: Token) -> Result<Hold, WrongToken> {
        self.check_expiry();
        match &mut self.state {
            State::Held(hold) if hold.token == token => {
                let new_expiry = SystemTime::now() + self.token_lifetime;
                hold.expiry = new_expiry;

                tracing::info!(
                    owner = hold.owner,
                    remaining_s = self.token_lifetime.as_secs(),
                    "lease renewed"
                );
                Ok(hold.clone())
            }
            _ => Err(WrongToken),
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
        self.check_expiry();
        match &mut self.state {
            State::Held(hold) if hold.token == token => {
                tracing::info!(by = hold.owner, "lease withdrawn");
                self.state = State::Available;
                Ok(())
            }
            _ => Err(WrongToken),
        }
    }

    pub fn is_owned_by(&mut self, token: Token) -> bool {
        self.check_expiry();
        match &self.state {
            State::Held(hold) => hold.token == token,
            State::Available => false,
        }
    }

    fn check_expiry(&mut self) {
        if let State::Held(hold) = &self.state
            && SystemTime::now() > hold.expiry
        {
            tracing::info!(previous_owner = hold.owner, "lease expired");
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
        let sec_until = self.expiry.duration_since(SystemTime::now()).map_or(0, |d| d.as_secs());
        write!(f, "lease is already held. expires in {sec_until}s")
    }
}

#[derive(Debug, Serialize, Deserialize, Error)]
#[error("this token does not match the lease")]
pub struct WrongToken;

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    const LIFETIME: Duration = Duration::from_millis(20);

    #[test]
    fn test_renew() {
        let mut lease = Lease::new(LIFETIME);

        let h = lease.acquire("test").expect("lease is available");

        for _ in 0..5 {
            sleep(LIFETIME * 10 / 11);
            lease.renew(h.token).expect("lease has not expired");
        }

        sleep(LIFETIME * 11 / 10);
        assert!(lease.renew(h.token).is_err(), "lease has expired");
        assert!(lease.withdraw(h.token).is_err(), "cannot release an unowned lease");
    }

    #[test]
    fn test_owned_by() {
        let mut lease = Lease::new(LIFETIME);

        let wrong_token = Token(0);
        assert!(!lease.is_owned_by(wrong_token));

        let h = lease.acquire("test").expect("lease is available");
        assert!(lease.is_owned_by(h.token));
        lease.withdraw(h.token).expect("token owns the lease");
        assert!(!lease.is_owned_by(h.token));
    }
}
