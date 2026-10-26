use super::*;
use crate::{
    domain::{
        auth::UserSession,
        pag::{Pag, PagArg, PagRes},
    },
    dto::*,
    err::{AppErr, AppErrJson},
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    middleware,
    routing::{delete, get, post},
};
use validator::Validate;

pub fn router(app: App) -> Router {
    Router::new()
        .route("/{member_id}", get(get_profile))
        .route("/{member_id}/default_portions", get(get_default_portions))
        .route("/{member_id}/default_portions", post(set_default_portions))
        .route("/{member_id}/delete", delete(delete_member))
        .route("/{member_id}/origin", get(get_origin))
        .route("/{member_id}/profile", post(set_profile))
        .route("/{member_id}/portion", post(set_portion))
        .route("/{member_id}/portions", get(portions_calendar))
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware))
        .with_state(app)
}

#[derive(OpenApi)]
#[openapi(paths(
    get_profile,
    get_default_portions,
    set_default_portions,
    delete_member,
    get_origin,
    set_profile,
    set_portion,
    portions_calendar
))]
pub struct MemberApi;

#[utoipa::path(get,
    tag = "member",
    path = "/{member_id}",
    description = "Get data about room",
    responses(
        (status = 200, body = MemberDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Member not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn get_profile(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(member_id): Path<Uuid>,
) -> Result<Json<MemberDto>, AppErr> {
    let res = sqlx::query_as!(
        MemberDto,
        r#"
        select
            m.id,
            m.user_id,
            m.room_id,
            coalesce(m.display_name, u.display_name, 'deleted') as "display_name!",
            coalesce(m.bio, u.bio, '') as "bio!",
            m.created_at
        from members as m
        left join users as u on m.user_id = u.id
        where m.id = $2 and permit_room($1, m.room_id)"#,
        user.user,
        member_id
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(get,
    tag = "member",
    path = "/{member_id}/default_portions",
    description = "Get var 'default_portions' for this member",
    responses(
        (status = 200, body = DefPortionsDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Member not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn get_default_portions(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(member_id): Path<Uuid>,
) -> Result<Json<DefPortionsDto>, AppErr> {
    let res = sqlx::query_as!(
        DefPortionsDto,
        r#"
        select
            default_portions as portions
        from members
        where id = $2 and permit_room($1, room_id)"#,
        user.user,
        member_id
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(post,
    tag = "member",
    path = "/{member_id}/default_portions",
    description = "Set 'default_portions' for member",
    request_body = DefPortionsDto,
    responses(
        (status = 200, body = DefPortionsDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Member not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn set_default_portions(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(member_id): Path<Uuid>,
    Json(p): Json<DefPortionsDto>,
) -> Result<Json<DefPortionsDto>, AppErr> {
    p.validate()?;

    let res = sqlx::query_as!(
        DefPortionsDto,
        r#"
        update members set
            default_portions = $3
        where id = $2 and user_id = $1
        returning
            default_portions as portions"#,
        user.user,
        member_id,
        p.portions
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(delete,
    tag = "member",
    path = "/{member_id}/delete",
    description = "Delete (soft delete) member in room (not implemented)",
    responses(
        (status = 501, body = AppErrJson, description = "Not implemented")
    ),
    security(
        ("jwt" = [])
    )
)]
async fn delete_member(
    State(_app): State<App>,
    Extension(_user): Extension<UserSession>,
    Path(_member_id): Path<Uuid>,
) -> Result<(), AppErr> {
    Err(AppErr::NotImpl)
}

#[utoipa::path(get,
    tag = "member",
    path = "/{member_id}/origin",
    description = "Get origin profile for member (without replacing deleted or display_name/bio for user)",
    responses(
        (status = 200, body = OriginMemberDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Member not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn get_origin(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(member_id): Path<Uuid>,
) -> Result<Json<OriginMemberDto>, AppErr> {
    let res = sqlx::query_as!(
        OriginMemberDto,
        r#"
        select
            id,
            user_id,
            room_id,
            display_name,
            bio,
            created_at
        from members
        where id = $2 and permit_room($1, room_id)"#,
        user.user,
        member_id
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}
#[utoipa::path(post,
    tag = "member",
    path = "/{member_id}/profile",
    description = "Set profile for member",
    responses(
        (status = 200, body = OriginMemberDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Member not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn set_profile(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(member_id): Path<Uuid>,
    Json(upd): Json<UpdProfileDto>,
) -> Result<Json<OriginMemberDto>, AppErr> {
    upd.validate()?;

    let res = sqlx::query_as!(
        OriginMemberDto,
        r#"
        update members set
            display_name = $3,
            bio = $4
        where id = $2 and user_id = $1
        returning
            id,
            user_id,
            room_id,
            display_name,
            bio,
            created_at"#,
        user.user,
        member_id,
        upd.display_name,
        upd.bio
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(get,
    tag = "member",
    path = "/{room_id}/members",
    description = "List of active (not deleted) members in room",
    responses(
        (status = 200, body = PagRes<MemberDto>),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn list_members(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Query(pag): Query<Pag>,
) -> Result<Json<PagRes<MemberDto>>, AppErr> {
    pag.validate()?;

    let f = async |pag: PagArg| -> Result<_, AppErr> {
        let res = sqlx::query_as!(
            MemberDto,
            r#"
        select
            m.id,
            m.user_id,
            m.room_id,
            coalesce(m.display_name, u.display_name) as "display_name!",
            coalesce(m.bio, u.bio) as "bio!",
            m.created_at
        from members as m
        join users as u on m.user_id = u.id
        where m.room_id = $2 and permit_room($1, $2)
        order by m.created_at desc
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

#[utoipa::path(post,
    tag = "portion",
    path = "/{member_id}/portion",
    description = "Set portions cnt for specific day for member",
    request_body = PortionDateDto,
    responses(
        (status = 200),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Member not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn set_portion(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(member_id): Path<Uuid>,
    Json(p): Json<PortionDateDto>,
) -> Result<(), AppErr> {
    p.validate()?;

    sqlx::query!(
        r#"
        insert into portions (
            member_id,
            portion_date,
            cnt
        ) select
            id,
            $3,
            $4
        from members
        where id = $2 and user_id = $1
        on conflict (member_id, portion_date)
        do update set
            cnt = excluded.cnt
        returning member_id"#,
        user.user,
        member_id,
        p.date,
        p.portions
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(())
}

#[utoipa::path(get,
    tag = "portion",
    path = "/{member_id}/portions",
    description = "Get all portions of member within a centrain period of time",
    request_body = TwoDateDto,
    responses(
        (status = 200, body = Vec<PortionDateDto>),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn portions_calendar(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(member_id): Path<Uuid>,
    Json(ord): Json<TwoDateDto>,
) -> Result<Json<Vec<PortionDateDto>>, AppErr> {
    ord.validate()?;

    let res = sqlx::query_as!(
        PortionDateDto,
        r#"
        select
            p.portion_date as date,
            p.cnt as portions
        from portions as p
        join members as m on p.member_id = m.id
        where
            m.id = $2 and
            permit_room($1, m.room_id) and
            $3::date <= p.portion_date and
            p.portion_date <= $4::date"#,
        user.user,
        member_id,
        ord.begin,
        ord.end
    )
    .fetch_all(&app.pool)
    .await?;

    Ok(Json(res))
}
