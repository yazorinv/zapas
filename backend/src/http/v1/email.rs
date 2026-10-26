use super::*;
use crate::{
    domain::auth::UserSession,
    err::{AppErr, AppErrJson},
    http::v1::auth_middleware,
    repo::user::UserRepo,
};
use axum::{Extension, Json, extract::State, middleware, routing::post};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

pub fn router(app: App) -> Router {
    let without = Router::new().route("/forgot", post(forgot));
    let with = Router::new()
        .route("/me/email", post(upd_email))
        .route("/me/email/verify", post(verify))
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware));
    without.merge(with).with_state(app)
}

#[derive(OpenApi)]
#[openapi(paths(forgot, upd_email, verify))]
pub struct EmailApi;

#[utoipa::path(post,
    tag = "email",
    path = "/forgot",
    description = "Send password reset link to email",
    responses(
        (status = 501, body = AppErrJson, description = "Not implemented"),
    )
)]
async fn forgot(State(_app): State<App>, Json(email): Json<EmailReq>) -> Result<(), AppErr> {
    email.validate()?;

    Err(AppErr::NotImpl)
}

#[utoipa::path(post,
    tag = "email",
    path = "/me/email",
    description = "Update email and send verification link (send not implemented)",
    responses(
        (status = 200),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "User not found (what???)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn upd_email(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Json(email): Json<EmailReq>,
) -> Result<(), AppErr> {
    email.validate()?;

    let changed = app.user_repo.set_email(user.user, &email.email).await?;
    if changed {
        // TODO add send verify email
    }

    Ok(())
}

#[utoipa::path(post,
    tag = "email",
    path = "/me/email/verify",
    description = "Send verification link",
    responses(
        (status = 501, body = AppErrJson, description = "Not implemented"),
    )
)]
async fn verify(
    State(_app): State<App>,
    Extension(_user): Extension<UserSession>,
) -> Result<(), AppErr> {
    Err(AppErr::NotImpl)
}

#[derive(Deserialize, Validate, ToSchema)]
struct EmailReq {
    #[validate(email, length(max = 255))]
    email: String,
}
