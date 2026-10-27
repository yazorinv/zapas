mod acc;
mod auth;
mod email;
mod member;
mod room;
mod sse;
mod storage;
mod transaction;
mod user;

use crate::app::App;
use axum::Router;
use utoipa::OpenApi;
use uuid::Uuid;
use validator::ValidationError;

pub fn router(app: App) -> Router {
    let auth = Router::new().nest("/user", auth::router(app.clone()));
    let user = Router::new().nest("/user", user::router(app.clone()));
    let room = Router::new().nest("/room", room::router(app.clone()));
    let sse = Router::new().nest("/sse", sse::router(app.clone()));
    let email = Router::new().nest("/user", email::router(app.clone()));
    let storage = Router::new().nest("/storage", storage::router(app.clone()));
    let member = Router::new().nest("/member", member::router(app.clone()));
    let tr = Router::new().nest("/transaction", transaction::router(app.clone()));
    let acc = Router::new().nest("/acc", acc::router(app.clone()));

    auth.merge(user)
        .merge(room)
        .merge(sse)
        .merge(email)
        .merge(storage)
        .merge(member)
        .merge(tr)
        .merge(acc)
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/user", api = auth::AuthApi),
        (path = "/user", api = user::UserApi),
        (path = "/room", api = room::RoomApi),
        (path = "/sse", api = sse::SseApi),
        (path = "/user", api = email::EmailApi),
        (path = "/storage", api = storage::StorageApi),
        (path = "/member", api = member::MemberApi),
        (path = "/transaction", api = transaction::TrApi),
        (path = "/acc", api = acc::AccApi),
    )
)]
pub struct V1Api;

pub use auth::auth_middleware;

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        Ok(())
    } else {
        Err(ValidationError::new("invalid username chars"))
    }
}

pub fn validate_passwd(passwd: &str) -> Result<(), ValidationError> {
    const SPECIAL_CHARS: &str = "!@#$%^&*()_-+=?/.,><;:";

    if !passwd.chars().any(|c| c.is_ascii_alphabetic()) {
        Err(ValidationError::new("not found alphabetic is passwd"))
    } else if !passwd.chars().any(|c| c.is_ascii_digit()) {
        Err(ValidationError::new("not found digit in passwd"))
    } else if !passwd.chars().any(|c| c.is_ascii_lowercase()) {
        Err(ValidationError::new("not found lowercase in passwd"))
    } else if !passwd.chars().any(|c| c.is_ascii_uppercase()) {
        Err(ValidationError::new("not found uppercase in passwd"))
    } else if !passwd.chars().any(|c| SPECIAL_CHARS.contains(c)) {
        Err(ValidationError::new("not found special chars in passwd"))
    } else {
        Ok(())
    }
}
