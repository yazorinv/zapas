use super::*;
use crate::{
    domain::{auth::UserSession, passwd::*, user::*},
    dto::UuidDto,
    err::{AppErr, AppErrJson},
    repo::user::UserRepo,
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    middleware,
    routing::{delete, get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

pub fn router(app: App) -> Router {
    let without_middleware = Router::new().route("/register", post(register));

    let with = Router::new()
        .route("/delete", delete(delete_user))
        .route("/me", get(me))
        .route("/by_id/{id}", get(other_user_by_id))
        .route("/by_username/{username}", get(other_user_by_username))
        .route("/me/passwd", post(change_passwd))
        .route("/me/profile", post(upd_profile))
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware));

    without_middleware.merge(with).with_state(app)
}

#[derive(OpenApi)]
#[openapi(paths(
    register,
    delete_user,
    me,
    other_user_by_id,
    other_user_by_username,
    change_passwd,
    upd_profile
))]
pub struct UserApi;

#[utoipa::path(post,
    tag = "user",
    path = "/register",
    description = "Register new user",
    request_body = RegisterReq,
    responses(
        (status = 200, body = UuidDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 409, body = AppErrJson, description = "Already exists username or email")
    )
)]
async fn register(
    State(app): State<App>,
    Json(reg): Json<RegisterReq>,
) -> Result<Json<UuidDto>, AppErr> {
    reg.validate()?;

    Ok(Json(UuidDto {
        id: app.user_repo.new_user(reg.try_into()?).await?.id,
    }))
}

#[utoipa::path(delete,
    tag = "user",
    path = "/delete",
    description = "Delete user (not implemented)",
    responses(
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 501, body = AppErrJson, description = "Not implemented"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn delete_user(
    State(_app): State<App>,
    Extension(_user): Extension<UserSession>,
) -> Result<(), AppErr> {
    Err(AppErr::NotImpl)
}

#[utoipa::path(get,
    tag = "user",
    path = "/me",
    description = "All data about yourself",
    responses(
        (status = 200, body = UserMeResp),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "User not found (what?)")
    ),
    security(
        ("jwt" = [])
    )
)]
async fn me(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
) -> Result<Json<UserMeResp>, AppErr> {
    Ok(Json(
        app.user_repo
            .get_by_id(user.user)
            .await?
            .ok_or(AppErr::NotFound)?
            .into(),
    ))
}

#[utoipa::path(get,
    tag = "user",
    path = "/by_id/{id}",
    description = "Data about other user",
    responses(
        (status = 200, body = UserOtherResp),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "User not found")
    ),
    security(
        ("jwt" = [])
    )
)]
async fn other_user_by_id(
    State(app): State<App>,
    Extension(_user): Extension<UserSession>,
    Path(UuidDto { id }): Path<UuidDto>,
) -> Result<Json<UserOtherResp>, AppErr> {
    Ok(Json(
        app.user_repo
            .get_by_id(id)
            .await?
            .ok_or(AppErr::NotFound)?
            .into(),
    ))
}

#[utoipa::path(get,
    tag = "user",
    path = "/by_username/{username}",
    description = "Data about other user",
    responses(
        (status = 200, body = UserOtherResp),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "User not found")
    ),
    security(
        ("jwt" = [])
    )
)]
async fn other_user_by_username(
    State(app): State<App>,
    Extension(_user): Extension<UserSession>,
    Path(username): Path<String>,
) -> Result<Json<UserOtherResp>, AppErr> {
    let path = UserNameReq { username };
    path.validate()?;
    let username = &path.username;

    Ok(Json(
        app.user_repo
            .get_by_username(username)
            .await?
            .ok_or(AppErr::NotFound)?
            .into(),
    ))
}

#[utoipa::path(post,
    tag = "user",
    path = "/me/passwd",
    description = "Change password and logout all",
    request_body = ChangePasswdReq,
    responses(
        (status = 200),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "User not found (what?)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn change_passwd(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Json(cp): Json<ChangePasswdReq>,
) -> Result<(), AppErr> {
    cp.validate()?;
    let new_hash = gen_passwd_hash(&cp.passwd)?;

    app.user_repo
        .upd_passwd(user.user, &cp.old_passwd, &new_hash)
        .await?;
    Ok(())
}

#[utoipa::path(post,
    tag = "user",
    path = "/me/profile",
    description = "Change profile (username, display_name, bio)",
    request_body = UpdUserProfileReq,
    responses(
        (status = 200, body = UserMeResp),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "User not found (what?)"),
        (status = 409, body = AppErrJson, description = "Already exists username")
    ),
    security(
        ("jwt" = [])
    )
)]
async fn upd_profile(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Json(upd): Json<UpdUserProfileReq>,
) -> Result<Json<UserMeResp>, AppErr> {
    upd.validate()?;

    Ok(Json(app.user_repo.upd(user.user, upd.into()).await?.into()))
}

#[derive(Clone, Deserialize, Validate, ToSchema)]
struct RegisterReq {
    #[schema(example = "bob_ross")]
    #[validate(length(min = 4, max = 63), custom(function = validate_username))]
    pub username: String,

    #[schema(example = "bob.ross@example.com")]
    #[validate(email, length(max = 255))]
    pub email: Option<String>,

    #[schema(example = "Super_password!67")]
    #[validate(length(min = 12, max = 127), custom(function = validate_passwd))]
    pub passwd: String,

    #[schema(example = "bob ross")]
    #[validate(length(min = 1, max = 127))]
    pub display_name: String,

    #[schema(example = "")]
    #[validate(length(max = 1024))]
    pub bio: String,
}

#[derive(Serialize, ToSchema)]
struct UserMeResp {
    pub id: Uuid,

    #[schema(example = "bob_ross")]
    pub username: String,

    #[schema(example = "bob.ross@example.com")]
    pub email: Option<String>,

    #[schema(example = false)]
    pub verified_email: bool,

    #[schema(example = "bob ross")]
    pub display_name: String,

    #[schema(example = "")]
    pub bio: String,

    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
struct UserOtherResp {
    pub id: Uuid,

    #[schema(example = "omega")]
    pub username: String,

    #[schema(example = "omega bob")]
    pub display_name: String,

    #[schema(example = "omega plus 1 bob")]
    pub bio: String,

    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate)]
pub struct UserNameReq {
    #[validate(length(min = 4, max = 63), custom(function = validate_username))]
    pub username: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[validate(schema(function = validate_change_passwd))]
pub struct ChangePasswdReq {
    #[schema(example = "Super_password!67")]
    #[validate(length(min = 12, max = 127))]
    pub old_passwd: String,

    #[schema(example = "Super_puper_password!67")]
    #[validate(length(min = 12, max = 127), custom(function = validate_passwd))]
    pub passwd: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdUserProfileReq {
    #[validate(length(min = 4, max = 63), custom(function = validate_username))]
    pub username: Option<String>,

    #[validate(length(min = 1, max = 127))]
    pub display_name: Option<String>,

    #[schema(example = "Bio")]
    #[validate(length(max = 1024))]
    pub bio: Option<String>,
}

impl TryFrom<RegisterReq> for MakeUser {
    type Error = AppErr;

    fn try_from(x: RegisterReq) -> Result<Self, Self::Error> {
        Ok(MakeUser {
            id: Uuid::now_v7(),
            username: x.username,
            email: x.email,
            passwd_hash: gen_passwd_hash(&x.passwd)?,
            display_name: x.display_name,
            bio: x.bio,
        })
    }
}

impl From<User> for UserMeResp {
    fn from(x: User) -> Self {
        Self {
            id: x.id,
            username: x.username,
            email: x.email,
            verified_email: x.verified_email,
            display_name: x.display_name,
            bio: x.bio,
            created_at: x.created_at,
        }
    }
}

impl From<User> for UserOtherResp {
    fn from(x: User) -> Self {
        Self {
            id: x.id,
            username: x.username,
            display_name: x.display_name,
            bio: x.bio,
            created_at: x.created_at,
        }
    }
}

impl From<UpdUserProfileReq> for UpdUser {
    fn from(x: UpdUserProfileReq) -> Self {
        UpdUser {
            username: x.username,
            display_name: x.display_name,
            bio: x.bio,
        }
    }
}

fn validate_change_passwd(cp: &ChangePasswdReq) -> Result<(), ValidationError> {
    if cp.old_passwd == cp.passwd {
        return Err(ValidationError::new("same passwords"));
    }
    Ok(())
}
