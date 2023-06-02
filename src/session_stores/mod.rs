use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(feature = "jwt_session")]
pub mod jwt;
#[cfg(feature = "redis_session")]
pub mod redis;

#[derive(Debug)]
pub enum Errors {
    Serde,
    Jwt,
    Redis,
    Hmac,
}

#[async_trait]
pub trait SessionStore<S: Serialize + for<'a> Deserialize<'a> + Send + Clone> {
    type Error: Debug;

    async fn gen_cookie(&mut self, session: S, expiry_s: usize) -> Result<String, Self::Error>;
    async fn retrieve(&mut self, cookie_value: &str) -> Option<S>;
    async fn remove(&mut self, cookie_value: &str) -> Result<(), Self::Error>;
}
