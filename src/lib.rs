#![allow(clippy::needless_return)]
#![allow(clippy::borrowed_box)]

use serde::{Deserialize, Serialize};
use thruster::{
    context::typed_hyper_context::TypedHyperContext, middleware_fn, ContextState, MiddlewareNext,
    MiddlewareResult,
};

pub mod session_stores;
use session_stores::SessionStore;

#[derive(Clone)]
pub struct Session<S> {
    pub cookie_name: String,
    pub data: Option<S>,
}

#[middleware_fn]
pub async fn session_middleware<
    C: Send + ContextState<Session<S>> + ContextState<Box<T>>,
    T: 'static + SessionStore<S> + Sync,
    S: 'static + Send + Sync + for<'a> Deserialize<'a> + Serialize + Clone,
>(
    mut context: TypedHyperContext<C>,
    next: MiddlewareNext<TypedHyperContext<C>>,
) -> MiddlewareResult<TypedHyperContext<C>> {
    let session_store: &Box<_> = context.extra.get();
    let current_session: &Session<_> = context.extra.get();

    let cookie_value = context.cookies.get(&current_session.cookie_name);
    if let Some(cookie_value) = cookie_value {
        let session = session_store.retrieve(&cookie_value.value).await;

        *context.extra.get_mut() = Session {
            data: session,
            ..current_session.clone()
        };
    } else {
        return next(context).await;
    }

    return next(context).await;
}
