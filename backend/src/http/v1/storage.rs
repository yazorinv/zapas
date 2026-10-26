use super::*;
use crate::{
    app::App,
    domain::{
        auth::UserSession,
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
use utoipa::OpenApi;
use validator::Validate;

pub fn router(app: App) -> Router {
    Router::new()
        .route("/{storage_id}", get(get_profile))
        .route("/{storage_id}/delete", delete(delete_storage))
        .route("/{storage_id}/profile", post(upd_profile))
        .route("/{storage_id}/timing", post(set_timing))
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware))
        .with_state(app)
}

#[derive(OpenApi)]
#[openapi(paths(get_profile, delete_storage, upd_profile, set_timing))]
pub struct StorageApi;

#[utoipa::path(get,
    tag = "storage",
    path = "/{stroage_id}",
    description = "Get storage profile",
    responses(
        (status = 200, body = StorageDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Storage not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn get_profile(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(storage_id): Path<Uuid>,
) -> Result<Json<StorageDto>, AppErr> {
    let res = sqlx::query_as!(
        StorageDto,
        r#"
        select
            id,
            room_id,
            display_name,
            bio,
            review_interval,
            expired_notify,
            created_at
        from storages
        where id = $2 and permit_room($1, room_id)"#,
        user.user,
        storage_id
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(delete,
    tag = "storage",
    path = "/{storage_id}/delete",
    description = "Delete storage with items in this storage",
    responses(
        (status = 200),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Storage not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn delete_storage(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(storage_id): Path<Uuid>,
) -> Result<(), AppErr> {
    sqlx::query!(
        r#"
        delete from storages
        where id = $2 and permit_room($1, room_id)
        returning id"#,
        user.user,
        storage_id
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(())
}

#[utoipa::path(post,
    tag = "storage",
    path = "/{storage_id}/profile",
    description = "Update storage profile",
    request_body = UpdProfileDto,
    responses(
        (status = 200, body = StorageDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Storage not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn upd_profile(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(storage_id): Path<Uuid>,
    Json(upd): Json<UpdProfileDto>,
) -> Result<Json<StorageDto>, AppErr> {
    upd.validate()?;

    let res = sqlx::query_as!(
        StorageDto,
        r#"
        update storages set
            display_name = coalesce($3, display_name),
            bio = coalesce($4, bio)
        where id = $2 and permit_room($1, room_id)
        returning
            id,
            room_id,
            display_name,
            bio,
            review_interval,
            expired_notify,
            created_at"#,
        user.user,
        storage_id,
        upd.display_name,
        upd.bio
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(post,
    tag = "storage",
    path = "/{storage_id}/timing",
    description = "Set storage timing",
    request_body = Timing,
    responses(
        (status = 200, body = StorageDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Storage not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn set_timing(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(storage_id): Path<Uuid>,
    Json(upd): Json<Timing>,
) -> Result<Json<StorageDto>, AppErr> {
    upd.validate()?;

    let res = sqlx::query_as!(
        StorageDto,
        r#"
        update storages set
            review_interval = $3,
            expired_notify = $4
        where id = $2 and permit_room($1, room_id)
        returning
            id,
            room_id,
            display_name,
            bio,
            review_interval,
            expired_notify,
            created_at"#,
        user.user,
        storage_id,
        upd.review_interval,
        upd.expired_notify
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(post,
    tag = "storage",
    path = "/{room_id}/new_storage",
    description = "Make new storage in room",
    request_body = ProfileDto,
    responses(
        (status = 200, body = UuidDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn make(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Json(profile): Json<ProfileDto>,
) -> Result<Json<UuidDto>, AppErr> {
    profile.validate()?;

    let res = sqlx::query!(
        r#"
        insert into storages (
            id,
            room_id,
            display_name,
            bio
        ) select
            $5,
            room_id,
            $3,
            $4
        from members
        where user_id = $1 and room_id = $2
        returning id"#,
        user.user,
        room_id,
        profile.display_name,
        profile.bio,
        Uuid::now_v7(),
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(UuidDto { id: res.id }))
}

#[utoipa::path(get,
    tag = "storage",
    path = "/{room_id}/storages",
    description = "List of storages in room",
    params(
        Pag,
    ),
    responses(
        (status = 200, body = PagRes<StorageDto>),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn list_storages(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Query(pag): Query<Pag>,
) -> Result<Json<PagRes<StorageDto>>, AppErr> {
    pag.validate()?;

    let f = async |pag: PagArg| -> Result<_, AppErr> {
        let res = sqlx::query_as!(
            StorageDto,
            r#"
            select
                id,
                room_id,
                display_name,
                bio,
                review_interval,
                expired_notify,
                created_at
            from storages
            where room_id = $2 and permit_room($1, $2)
            order by created_at desc
            offset $3 limit $4"#,
            user.user,
            room_id,
            pag.offset,
            pag.limit
        )
        .fetch_all(&app.pool)
        .await?;

        Ok(res)
    };

    Ok(Json(PagRes::new_async_with_err(pag, f).await?))
}
