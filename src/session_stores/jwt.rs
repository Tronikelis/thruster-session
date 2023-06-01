use async_trait::async_trait;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::SessionStore;

#[derive(Debug)]
pub enum Errors {
    Serde,
    Jwt,
}

pub struct JwtSession {
    secret: Hmac<Sha256>,
}

impl JwtSession {
    pub fn new(secret: &str) -> Self {
        return Self {
            secret: Hmac::new_from_slice(secret.as_bytes()).unwrap(),
        };
    }
}

#[async_trait]
impl<S: Serialize + for<'a> Deserialize<'a> + Send + 'static + Clone> SessionStore<S>
    for JwtSession
{
    type Error = Errors;

    async fn gen_cookie(&self, session: S) -> Result<String, Self::Error> {
        let jwt = session
            .sign_with_key(&self.secret)
            .map_err(|_| Errors::Jwt)?;

        return Ok(jwt);
    }

    async fn retrieve(&self, cookie_value: &str) -> Option<S> {
        let session: Option<S> = cookie_value.verify_with_key(&self.secret).ok();
        return session;
    }
}
