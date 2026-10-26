use super::*;
use crate::{
    app::App,
    domain::{
        auth::{TokenData, UserSession},
        pag::{Pag, PagArg, PagRes},
    },
    dto::*,
    err::{AppErr, AppErrJson},
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    middleware,
    routing::{delete, get, post},
};
use chrono::Duration;
use utoipa::OpenApi;
use validator::Validate;

pub fn router(app: App) -> Router {
    Router::new()
        .route("/make", post(make))
        .route("/all", get(list_rooms))
        .route("/{room_id}", get(room_by_id))
        .route("/{room_id}/delete", delete(room_delete))
        .route("/{room_id}/profile", post(upd_profile))
        .route("/{room_id}/timing", post(upd_timing))
        .route("/join/{token}", post(join))
        .route("/{room_id}/invite/gen", post(gen_invite))
        .route("/{room_id}/new_storage", post(super::storage::make))
        .route("/{room_id}/storages", get(super::storage::list_storages))
        .route("/{room_id}/members", get(super::member::list_members))
        .route(
            "/{room_id}/new_transaction",
            post(super::transaction::new_transaction),
        )
        .route(
            "/{room_id}/new_credit",
            post(super::transaction::new_credit),
        )
        .route(
            "/{room_id}/transactions",
            get(super::transaction::all_transactions),
        )
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware))
        .with_state(app)
}

#[derive(OpenApi)]
#[openapi(paths(
    make,
    list_rooms,
    room_by_id,
    room_delete,
    upd_profile,
    upd_timing,
    join,
    gen_invite,
    super::storage::make,
    super::storage::list_storages,
    super::member::list_members,
    super::transaction::new_transaction,
    super::transaction::new_credit,
    super::transaction::all_transactions
))]
pub struct RoomApi;

#[utoipa::path(post,
    tag = "room",
    path = "/make",
    description = "Make new room",
    request_body = ProfileDto,
    responses(
        (status = 200, body = UuidDto, description = "Room id"),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn make(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Json(profile): Json<ProfileDto>,
) -> Result<Json<UuidDto>, AppErr> {
    profile.validate()?;

    let mut tx = app.pool.begin().await?;

    let room_id = Uuid::now_v7();
    sqlx::query!(
        r#"
        insert into rooms (
            id,
            display_name,
            bio
        ) values (
            $1, $2, $3
        )"#,
        room_id,
        profile.display_name,
        profile.bio
    )
    .execute(&mut *tx)
    .await?;

    let member_id = Uuid::now_v7();
    sqlx::query!(
        r#"
        insert into members (
            id,
            user_id,
            room_id
        ) values (
            $1, $2, $3
        )"#,
        member_id,
        user.user,
        room_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(UuidDto { id: room_id }))
}

#[utoipa::path(get,
    tag = "room",
    path = "/all",
    description = "List of rooms for user",
    params(
        Pag
    ),
    responses(
        (status = 200, body = PagRes<RoomDto>),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn list_rooms(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Query(pag): Query<Pag>,
) -> Result<Json<PagRes<RoomDto>>, AppErr> {
    let f = async |pag: PagArg| -> Result<_, AppErr> {
        let res = sqlx::query_as!(
            RoomDto,
            r#"
        select
            r.id,
            r.display_name,
            r.bio,
            r.review_interval,
            r.expired_notify,
            r.created_at
        from members as m
        join rooms as r on m.room_id = r.id
        where m.user_id = $1
        order by m.created_at desc
        offset $2 limit $3"#,
            user.user,
            pag.offset,
            pag.limit
        )
        .fetch_all(&app.pool)
        .await?;
        Ok(res)
    };

    Ok(Json(PagRes::new_async_with_err(pag, f).await?))
}

#[utoipa::path(get,
    tag = "room",
    path = "/{room_id}",
    description = "Get data about room",
    responses(
        (status = 200, body = RoomDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn room_by_id(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<RoomDto>, AppErr> {
    let res = sqlx::query_as!(
        RoomDto,
        r#"
        select
            id,
            display_name,
            bio,
            review_interval,
            expired_notify,
            created_at
        from rooms
        where id = $2 and permit_room($1, $2)"#,
        user.user,
        room_id
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(delete,
    tag = "room",
    path = "/{room_id}/delete",
    description = "Delete room (not implemented)",
    responses(
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 501, body = AppErrJson, description = "Not implemented"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn room_delete(
    State(_app): State<App>,
    Extension(_user): Extension<UserSession>,
    Path(_room_id): Path<Uuid>,
) -> Result<(), AppErr> {
    Err(AppErr::NotImpl)
}

#[utoipa::path(post,
    tag = "room",
    path = "/{room_id}/profile",
    description = "Update room profile",
    request_body = UpdProfileDto,
    responses(
        (status = 200, body = RoomDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn upd_profile(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Json(profile): Json<UpdProfileDto>,
) -> Result<Json<RoomDto>, AppErr> {
    profile.validate()?;

    let res = sqlx::query_as!(
        RoomDto,
        r#"
        update rooms set
            display_name = coalesce($3, display_name),
            bio = coalesce($4, bio)
        where id = $2 and permit_room($1, $2)
        returning
            id,
            display_name,
            bio,
            review_interval,
            expired_notify,
            created_at"#,
        user.user,
        room_id,
        profile.display_name,
        profile.bio
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(post,
    tag = "room",
    path = "/{room_id}/timing",
    description = "Update room timing",
    request_body = Timing,
    responses(
        (status = 200, body = RoomDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn upd_timing(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Json(tim): Json<Timing>,
) -> Result<Json<RoomDto>, AppErr> {
    tim.validate()?;

    let res = sqlx::query_as!(
        RoomDto,
        r#"
        update rooms set
            review_interval = coalesce($3, review_interval),
            expired_notify = coalesce($4, expired_notify)
        where id = $2 and permit_room($1, $2)
        returning
            id,
            display_name,
            bio,
            review_interval,
            expired_notify,
            created_at"#,
        user.user,
        room_id,
        tim.review_interval,
        tim.expired_notify
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(post,
    tag = "room",
    path = "/join/{token}",
    description = "Invite to room with token",
    responses(
        (status = 200, body = UuidDto, description = "Room id"),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn join(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(token): Path<String>,
) -> Result<Json<UuidDto>, AppErr> {
    let room_id = app.invite.check_invite(&token, user.user)?;

    sqlx::query!(
        r#"
        insert into members (
            id,
            user_id,
            room_id
        ) values (
            $1, $2, $3
        )"#,
        Uuid::now_v7(),
        user.user,
        room_id
    )
    .execute(&app.pool)
    .await?;

    Ok(Json(UuidDto { id: room_id }))
}

#[utoipa::path(post,
    tag = "room",
    path = "/{room_id}/invite/gen",
    description = "Make new invite token",
    params(
      InviteQuery
    ),
    responses(
        (status = 200, body = TokenData),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn gen_invite(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Query(q): Query<InviteQuery>,
) -> Result<Json<TokenData>, AppErr> {
    q.validate()?;

    sqlx::query!(
        r#"
        select id from members
        where user_id = $1 and room_id = $2"#,
        user.user,
        room_id
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    let token = app
        .invite
        .make_invite(room_id, q.for_user, Duration::seconds(q.secs))?;

    Ok(Json(token))
}
