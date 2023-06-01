use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod jwt;

#[async_trait]
pub trait SessionStore<S: Serialize + for<'a> Deserialize<'a> + Send + Clone> {
    type Error: Debug;

    async fn gen_cookie(&self, session: S) -> Result<String, Self::Error>;
    async fn retrieve(&self, cookie_value: &str) -> Option<S>;
}
