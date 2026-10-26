use super::*;
use crate::{
    domain::{
        auth::{AuthToken, Session, TokenData, UserSession},
        pag::{Pag, PagArg, PagRes},
    },
    err::{AppErr, AppErrJson, ErrorLog},
    repo::session::SessionRepo,
};
use axum::{
    Extension, Json,
    body::Body,
    extract::{Query, Request, State},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

pub fn router(app: App) -> Router {
    let with_ip = Router::new()
        .route("/login", post(login))
        .layer(middleware::from_fn(ip_middleware));

    let with_user = Router::new()
        .route("/me/logout", post(logout))
        .route("/me/sessions", get(list_sessions))
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware));

    let with_token = Router::new()
        .route("/prolong", post(prolong))
        .route("/gen_access", post(gen_access))
        .layer(middleware::from_fn_with_state(
            app.clone(),
            simple_auth_middleware,
        ));

    Router::new()
        .merge(with_ip)
        .merge(with_user)
        .merge(with_token)
        .with_state(app)
}

#[derive(OpenApi)]
#[openapi(paths(login, logout, list_sessions, prolong, gen_access))]
pub struct AuthApi;

#[utoipa::path(post,
    tag = "auth",
    path = "/login",
    description = "Creating session and token using username and password",
    request_body = LoginReq,
    responses(
        (status = 200, body = TokenData),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Wrong username or password")
    )
)]
async fn login(
    State(app): State<App>,
    Extension(ip): Extension<IpAddr>,
    Json(login): Json<LoginReq>,
) -> Result<Json<TokenData>, AppErr> {
    login.validate()?;

    let token = app
        .session_repo
        .new_session(&login.username, &login.passwd, ip)
        .await?;

    let token = app.jwt.encode(token)?;
    Ok(Json(token))
}

#[utoipa::path(post,
    tag = "auth",
    path = "/me/logout",
    description = "Delete current or all sessions for user",
    params(
        LogoutQuery
    ),
    responses(
        (status = 200),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "User or session not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn logout(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Query(LogoutQuery { all }): Query<LogoutQuery>,
) -> Result<(), AppErr> {
    if all {
        app.session_repo.delete_all_sessions(user.user).await?;
    } else {
        app.session_repo.delete_session(user.session).await?;
    }
    Ok(())
}

#[utoipa::path(get,
    tag = "auth",
    path = "/me/sessions",
    description = "List of sessions for user",
    params(
        Pag
    ),
    responses(
        (status = 200, body = PagRes<SessionResp>),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn list_sessions(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Query(pag): Query<Pag>,
) -> Result<Json<PagRes<SessionResp>>, AppErr> {
    pag.validate()?;

    let f = async |pag: PagArg| -> Result<_, AppErr> {
        let res = app
            .session_repo
            .list_sessions(user.user, pag)
            .await?
            .into_iter()
            .map(|x| x.into())
            .collect();

        Ok(res)
    };

    Ok(Json(PagRes::new_async_with_err(pag, f).await?))
}

#[utoipa::path(post,
    tag = "auth",
    path = "/prolong",
    description = "Creating new refresh token based on the old one",
    responses(
        (status = 200, body = TokenData),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn prolong(
    State(app): State<App>,
    Extension(ip): Extension<IpAddr>,
    Extension(token): Extension<AuthToken>,
) -> Result<Json<TokenData>, AppErr> {
    let refresh = match token {
        AuthToken::Access(_) => {
            return Err(AppErr::InvalidToken);
        }

        AuthToken::Refresh(x) => x,
    };

    let mut session = app
        .session_repo
        .get_by_id(refresh.session())
        .await?
        .ok_or(AppErr::Unauthorized)?;

    let token = session.prolong(refresh, ip)?;
    let token = app.jwt.encode(token)?;

    Ok(Json(token))
}

#[utoipa::path(post,
    tag = "auth",
    path = "/gen_access",
    description = "Creating new access token based on refresh",
    responses(
        (status = 200, body = TokenData),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn gen_access(
    State(app): State<App>,
    Extension(ip): Extension<IpAddr>,
    Extension(token): Extension<AuthToken>,
) -> Result<Json<TokenData>, AppErr> {
    let refresh = match token {
        AuthToken::Access(_) => {
            return Err(AppErr::InvalidToken);
        }

        AuthToken::Refresh(x) => x,
    };

    let mut session = app
        .session_repo
        .get_by_id(refresh.session())
        .await?
        .ok_or(AppErr::Unauthorized)?;

    let token = session.make_access(refresh, ip)?;
    let token = app.jwt.encode(token)?;

    app.session_repo.upd(session).await?;
    Ok(Json(token))
}

/// Middleware с проверкой токена и генерацией итогового `UserSession`.
///
/// Дает полностью все данные, необходимые большинству функциям.
/// Скрывает всю логику с токенами и сессиями.
pub async fn auth_middleware(
    State(app): State<App>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppErr> {
    let ip: IpAddr = req
        .headers()
        .get("X-Real-IP")
        .and_then(|x| x.to_str().ok())
        .unwrap_or("127.0.0.1")
        .parse()
        .log_err()?;

    let header = req
        .headers()
        .get("Authorization")
        .and_then(|x| x.to_str().ok())
        .ok_or(AppErr::Unauthorized)?;

    let session = match app.jwt.decode(header)? {
        crate::domain::auth::AuthToken::Access(access) => Session::validate_access(access, ip)?,

        crate::domain::auth::AuthToken::Refresh(refresh) => {
            let mut session = app
                .session_repo
                .get_by_id(refresh.session())
                .await?
                .ok_or(AppErr::Unauthorized)?;

            let res = session.validate_refresh(refresh, ip)?;

            app.session_repo.upd(session).await?;
            res
        }
    };

    req.extensions_mut().insert(session);
    Ok(next.run(req).await)
}

/// Middleware для детальной работы с токенами и сессиями - дает `ip`
/// и непроверенный `auth_token`.
async fn simple_auth_middleware(
    State(app): State<App>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppErr> {
    let ip: IpAddr = req
        .headers()
        .get("X-Real-IP")
        .and_then(|x| x.to_str().ok())
        .unwrap_or("127.0.0.1")
        .parse()
        .log_err()?;

    let header = req
        .headers()
        .get("Authorization")
        .and_then(|x| x.to_str().ok())
        .ok_or(AppErr::Unauthorized)?;

    let res = app.jwt.decode(header)?;
    req.extensions_mut().insert(ip);
    req.extensions_mut().insert(res);
    Ok(next.run(req).await)
}

/// Middleware для получения `ip`.
async fn ip_middleware(mut req: Request<Body>, next: Next) -> Result<Response, AppErr> {
    let ip: IpAddr = req
        .headers()
        .get("X-Real-IP")
        .and_then(|x| x.to_str().ok())
        .unwrap_or("127.0.0.1")
        .parse()
        .log_err()?;

    req.extensions_mut().insert(ip);
    Ok(next.run(req).await)
}

#[derive(Deserialize, Validate, ToSchema)]
struct LoginReq {
    #[schema(example = "bob_ross")]
    #[validate(length(min = 4, max = 63))]
    username: String,

    #[schema(example = "Super_password!67")]
    #[validate(length(min = 1, max = 127))]
    passwd: String,
}

#[derive(Deserialize, IntoParams, ToSchema)]
struct LogoutQuery {
    #[serde(default)]
    all: bool,
}

#[derive(Clone, Serialize, ToSchema)]
struct SessionResp {
    id: Uuid,
    ip_addr: String,
    last_active_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<Session> for SessionResp {
    fn from(x: Session) -> Self {
        Self {
            id: x.id,
            ip_addr: x.ip.to_string(),
            last_active_at: x.last_active_at,
            created_at: x.created_at,
        }
    }
}
