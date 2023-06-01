use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thruster::{
    context::typed_hyper_context::TypedHyperContext,
    middleware::cookies::{Cookie, CookieOptions, HasCookies},
};

use crate::{Session, SessionStore};

pub struct JwtSession {}
#[async_trait]
impl<S: Serialize + for<'a> Deserialize<'a> + Send + 'static + Clone, C: Send> SessionStore<S, C>
    for JwtSession
{
    type Error = ();

    async fn insert(
        &self,
        context: &mut TypedHyperContext<C>,
        session: Session<S>,
    ) -> Result<(), Self::Error> {
        let session_str = serde_json::to_string(&session.data.unwrap()).unwrap();

        context.set_cookies(vec![Cookie {
            key: "thruster_session".to_string(),
            value: session_str,
            options: match session.cookie_options {
                Some(x) => x,
                None => CookieOptions::default(),
            },
        }]);

        return Ok(());
    }

    async fn retrieve(&self, cookie_value: &str) -> Result<S, Self::Error> {
        return Ok(serde_json::from_str(cookie_value).unwrap());
    }
}
