use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thruster::context::typed_hyper_context::TypedHyperContext;

pub mod jwt;

use crate::Session;

#[async_trait]
pub trait SessionStore<S: Serialize + for<'a> Deserialize<'a> + Send + Clone, C: Send> {
    type Error: Debug;

    async fn insert(
        &self,
        context: &mut TypedHyperContext<C>,
        session: Session<S>,
    ) -> Result<(), Self::Error>;
    async fn retrieve(&self, cookie_value: &str) -> Result<S, Self::Error>;
}
