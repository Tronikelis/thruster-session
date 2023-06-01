#![allow(clippy::needless_return)]

use async_trait::async_trait;
use std::{collections::HashMap, fmt::Debug};
use thruster::{
    context::typed_hyper_context::TypedHyperContext, middleware::cookies::HasCookies,
    middleware_fn, ContextState, MiddlewareNext, MiddlewareResult,
};

pub struct Session<S>(Option<S>);

#[async_trait]
pub trait SessionStore<S> {
    type Error: Send + Debug;

    async fn insert(&self, session: Session<S>) -> Result<(), Self::Error>;
    async fn retrieve(&self, cookie_value: &str) -> Result<Session<S>, Self::Error>;
}

#[middleware_fn]
async fn session_middleware<
    C: Send + ContextState<Session<S>> + ContextState<Box<T>>,
    T: 'static + SessionStore<S> + Sync,
    S: 'static + Send + Sync,
>(
    mut context: TypedHyperContext<C>,
    next: MiddlewareNext<TypedHyperContext<C>>,
) -> MiddlewareResult<TypedHyperContext<C>> {
    let session_store: &Box<_> = context.extra.get();
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

    let session = session_store.retrieve(&cookie_value).await.unwrap();
    *context.extra.get_mut() = session;

    return next(context).await;
}
