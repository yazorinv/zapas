use super::*;
use crate::{
    app::App,
    domain::{auth::UserSession, sse::EventMsg},
    err::{AppErr, AppErrJson},
};
use axum::{
    Extension, Router,
    extract::State,
    middleware,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
    routing::get,
};
use futures_util::{Stream, stream};
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast::error::RecvError;
use utoipa::OpenApi;

pub fn router(app: App) -> Router {
    Router::new()
        .route("/notify", get(notify))
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware))
        .with_state(app)
}

#[derive(OpenApi)]
#[openapi(paths(notify))]
pub struct SseApi;

#[utoipa::path(get,
    tag = "sse",
    path = "/notify",
    description = "Sse for notification",
    responses(
        (status = 200, body = EventMsg, content_type = "text/event-stream"),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn notify(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppErr> {
    let guard = app.sse.guard(user.user);

    let stream = stream::unfold(guard, |mut guard| async move {
        loop {
            match guard.receiver().recv().await {
                Ok(msg) => {
                    let json = serde_json::to_string(&msg).expect("Serialize sse event error");

                    let event = Event::default().event("message").data(json);
                    return Some((Ok(event), guard));
                }
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return None,
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    ))
}
