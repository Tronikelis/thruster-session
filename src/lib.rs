#![allow(clippy::needless_return)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thruster::{
    context::typed_hyper_context::TypedHyperContext,
    middleware::cookies::{CookieOptions, HasCookies},
    middleware_fn, ContextState, MiddlewareNext, MiddlewareResult,
};

pub mod session_stores;
use session_stores::SessionStore;

#[derive(Clone)]
pub struct Session<S: Serialize + for<'a> Deserialize<'a> + Clone> {
    cookie_options: Option<CookieOptions>,
    data: Option<S>,
    secret: String,
}

#[middleware_fn]
async fn session_middleware<
    C: Send + ContextState<Session<S>> + ContextState<Box<T>>,
    T: 'static + SessionStore<S, C> + Sync,
    S: 'static + Send + Sync + for<'a> Deserialize<'a> + Serialize + Clone,
>(
    mut context: TypedHyperContext<C>,
    next: MiddlewareNext<TypedHyperContext<C>>,
) -> MiddlewareResult<TypedHyperContext<C>> {
    let session_store: &Box<_> = context.extra.get();
    let session: &Session<_> = context.extra.get();

    // get the session id from the request cookie
    let cookies: HashMap<String, String> = context
        .get_cookies()
        .iter()
        .map(|x| {
            let equal: Vec<_> = x.split('=').map(|x| x.to_string()).collect();
            return (equal.get(0).unwrap().clone(), equal.get(1).unwrap().clone());
        })
        .collect();

    let cookie_value = urlencoding::decode(cookies.get("thruster_session").unwrap())
        .unwrap()
        .to_string();

    let session_data = session_store.retrieve(&cookie_value).await.unwrap();
    *context.extra.get_mut() = Session {
        data: Some(session_data),
        ..session.clone()
    };

    return next(context).await;
}
