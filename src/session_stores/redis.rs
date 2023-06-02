use async_trait::async_trait;
use hmac::{Hmac, Mac};
use nanoid::nanoid;
use redis::{aio::ConnectionManager, AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use super::Errors;
use crate::SessionStore;

#[derive(Clone)]
pub struct RedisSession {
    secret: Hmac<Sha256>,
    connection_manager: ConnectionManager,
}

impl RedisSession {
    pub async fn new(secret: &str, url: &str) -> RedisResult<Self> {
        let client = redis::Client::open(url)?;
        let connection_manager = ConnectionManager::new(client).await?;

        let secret = Hmac::new_from_slice(secret.as_bytes()).unwrap();

        return Ok(Self {
            connection_manager,
            secret,
        });
    }

    fn sign_string(&self, string: &str) -> String {
        let mut secret: Hmac<Sha256> = self.secret.clone();
        secret.update(string.as_bytes());

        // hex string
        return secret
            .finalize()
            .into_bytes()
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<_>>()
            .join("");
    }

    fn verify_string(&self, string: &str) -> Result<(), Errors> {
        let secret: Hmac<Sha256> = self.secret.clone();
        secret
            .verify_slice(&hex::decode(string).map_err(|_| Errors::Hmac)?)
            .map_err(|_| Errors::Hmac)?;

        return Ok(());
    }
}

#[async_trait]
impl<S: Clone + Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static> SessionStore<S>
    for RedisSession
{
    type Error = Errors;

    async fn gen_cookie(&mut self, session: S, expiry_s: usize) -> Result<String, Self::Error> {
        let id_safe = self.sign_string(&nanoid!());

        let _: () = self
            .connection_manager
            .set_ex(
                &id_safe,
                serde_json::to_string(&session).map_err(|_| Errors::Serde)?,
                expiry_s,
            )
            .await
            .map_err(|_| Errors::Redis)?;

        return Ok(id_safe);
    }

    async fn retrieve(&mut self, cookie_value: &str) -> Option<S> {
        if self.verify_string(cookie_value).is_ok() {
            let session_str: Option<String> = self.connection_manager.get(cookie_value).await.ok();
            if let Some(session_str) = session_str {
                let session: Result<S, _> = serde_json::from_str(&session_str);
                if let Ok(session) = session {
                    return Some(session);
                }
            }
        }

        return None;
    }

    async fn remove(&mut self, cookie_value: &str) -> Result<(), Self::Error> {
        self.connection_manager
            .del(cookie_value)
            .await
            .map_err(|_| Errors::Redis)?;

        return Ok(());
    }
}
