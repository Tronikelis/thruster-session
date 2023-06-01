#![allow(clippy::needless_return)]
#![allow(clippy::borrowed_box)]

use serde::{Deserialize, Serialize};
use thruster::{
    context::typed_hyper_context::TypedHyperContext, context_state, m,
    middleware::cookies::CookieOptions, middleware_fn, App, HyperRequest, MiddlewareNext,
    MiddlewareResult, Testable,
};

use thruster_session::{
    session_middleware,
    session_stores::{jwt::JwtSession, SessionStore},
    Session,
};

const COOKIE_NAME: &str = "session";
const SECRET: &str = "foo";

struct ServerState;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    username: String,
    email: String,
}

#[context_state]
struct RequestState(Session<User>, Box<JwtSession>);

type Context = TypedHyperContext<RequestState>;

fn generate_context(request: HyperRequest, _state: &ServerState, _path: &str) -> Context {
    return Context::new(
        request,
        RequestState(
            Session {
                cookie_name: COOKIE_NAME.to_string(),
                data: None,
            },
            Box::new(JwtSession::new(SECRET)),
        ),
    );
}

#[middleware_fn]
async fn root(mut context: Context, _next: MiddlewareNext<Context>) -> MiddlewareResult<Context> {
    context.body("yep");
    return Ok(context);
}

fn create_app() -> App<HyperRequest, Context, ServerState> {
    return App::<HyperRequest, Context, ServerState>::create(generate_context, ServerState)
        .middleware("/", m![session_middleware]);
}

#[tokio::test]
async fn hello_world() {
    let app = create_app().get("/", m![root]).commit();

    Testable::get(&app, "/", vec![])
        .await
        .unwrap()
        .expect_status(200, "OK");
}

#[middleware_fn]
async fn login_jwt_route(
    mut context: Context,
    _next: MiddlewareNext<Context>,
) -> MiddlewareResult<Context> {
    let user = User {
        username: "kmr".to_string(),
        email: "foo@bar.com".to_string(),
    };

    let session_store: &Box<JwtSession> = context.extra.get();
    let cookie_value = session_store.gen_cookie(user).await.unwrap();

    context.cookie(COOKIE_NAME, &cookie_value, &CookieOptions::default());

    return Ok(context);
}

#[tokio::test]
async fn login_jwt() {
    let app = create_app().post("/login", m![login_jwt_route]).commit();

    let response = Testable::post(&app, "/login", vec![], "".into())
        .await
        .unwrap()
        .expect_status(200, "OK");

    for (key, value) in response.headers {
        if key != "set-cookie" {
            continue;
        }

        assert!(value.contains("eyJhbGciOiJIUzI1NiJ9.eyJ1c2VybmFtZSI6ImttciIsImVtYWlsIjoiZm9vQGJhci5jb20ifQ.tmk6EDd40FNhJ89vwwGionqvNuxX4GetmC9G2EfIcqs"));
        return;
    }

    panic!("set-cookie don't exist");
}

#[tokio::test]
async fn session_jwt() {
    todo!("session jwt nyi");
}
