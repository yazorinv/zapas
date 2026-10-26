use super::*;
use crate::domain::pag::{PagArg, PagRes};
use crate::dto::*;
use crate::err::AppErrJson;
use crate::{domain::auth::UserSession, err::AppErr};
use axum::extract::Query;
use axum::middleware;
use axum::routing::{delete, get, post};
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use validator::Validate;

pub fn router(app: App) -> Router {
    Router::new()
        .route("/{transaction_id}", get(get_tr))
        .route("/{transaction_id}/delete", delete(delete_transaction))
        .route("/{transaction_id}/upd", post(upd_transaction))
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware))
        .with_state(app)
}

#[derive(OpenApi)]
#[openapi(paths(delete_transaction, get_tr, upd_transaction))]
pub struct TrApi;

#[utoipa::path(post,
    tag = "transaction",
    path = "/{room_id}/new_transaction",
    description = "Make new transaction (from --(money)-> to)",
    request_body = MakeTransactionDto,
    responses(
        (status = 200, body = UuidDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room or member not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn new_transaction(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Json(val): Json<MakeTransactionDto>,
) -> Result<Json<UuidDto>, AppErr> {
    val.validate()?;

    let res = sqlx::query!(
        r#"
        insert into transactions (
            id,
            room_id,
            from_member_id,
            to_member_id,
            money,
            bio
        ) select
            $6, $2, m.id, $3, $4, $5
        from members as m
        where m.user_id = $1 and m.room_id = $2 and exists (
            select 1 from members as v where v.room_id = $2 and v.id = $3
        )
        returning id"#,
        user.user,
        room_id,
        val.member_to,
        val.money,
        val.bio,
        Uuid::now_v7()
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(UuidDto { id: res.id }))
}

#[utoipa::path(post,
    tag = "transaction",
    path = "/{room_id}/new_credit",
    description = "Make new transaction (from <-(money)-- to). Changed the direction relative to 'new_transaction'",
    request_body = MakeTransactionDto,
    responses(
        (status = 200, body = UuidDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room or member not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn new_credit(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Json(val): Json<MakeTransactionDto>,
) -> Result<Json<UuidDto>, AppErr> {
    val.validate()?;

    // Swap to_member and from_member
    let res = sqlx::query!(
        r#"
        insert into transactions (
            id,
            room_id,
            to_member_id,
            from_member_id,
            money,
            bio
        ) select
            $6, $2, m.id, $3, $4, $5
        from members as m
        where m.user_id = $1 and m.room_id = $2 and exists (
            select 1 from members as v where v.room_id = $2 and v.id = $3
        )
        returning id"#,
        user.user,
        room_id,
        val.member_to,
        val.money,
        val.bio,
        Uuid::now_v7()
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(UuidDto { id: res.id }))
}

#[utoipa::path(delete,
    tag = "transaction",
    path = "/{transaction_id}/delete",
    description = "Delete transaction. For use only 'from_member' or 'to_member'.",
    responses(
        (status = 200),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Transaction not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn delete_transaction(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(transaction_id): Path<Uuid>,
) -> Result<(), AppErr> {
    sqlx::query!(
        r#"
        delete from transactions as t
        where
            t.id = $2 and
            permit_room($1, t.room_id) and
            exists (
                select 1
                from members as m
                where
                    (m.id = t.from_member_id and m.user_id = $1)
                    or
                    (m.id = t.to_member_id and m.user_id = $1)
            )
        returning id"#,
        user.user,
        transaction_id
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(())
}

#[utoipa::path(get,
    tag = "transaction",
    path = "/{room_id}",
    description = "Get data about transaction",
    responses(
        (status = 200, body = TransactionDto),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Trasnaction not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn get_tr(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<TransactionDto>, AppErr> {
    let res = sqlx::query_as!(
        TransactionDto,
        r#"
        select
            id,
            room_id,
            from_member_id as member_from,
            to_member_id as member_to,
            money,
            bio,
            created_at
        from transactions
        where id = $2 and permit_room($1, room_id)"#,
        user.user,
        transaction_id
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    Ok(Json(res))
}

#[utoipa::path(get,
    tag = "transaction",
    path = "/{room_id}/transactions",
    description = "List of transactions for room",
    params(
        TransactionQueryDto
    ),
    responses(
        (status = 200, body = PagRes<TransactionDto>),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn all_transactions(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Query(pag): Query<TransactionQueryDto>,
) -> Result<Json<PagRes<TransactionDto>>, AppErr> {
    pag.validate()?;

    let f = async |pg: PagArg| -> Result<_, AppErr> {
        let res = sqlx::query_as!(
            TransactionDto,
            r#"
        select
            id,
            room_id,
            from_member_id as member_from,
            to_member_id as member_to,
            money,
            bio,
            created_at
        from transactions
        where
            room_id = $2 and
            permit_room($1, $2) and
            (from_member_id = $3 or $3 is null) and
            (to_member_id = $4 or $4 is null)
        order by created_at
        offset $5 limit $6"#,
            user.user,
            room_id,
            pag.member_from,
            pag.member_to,
            pg.offset,
            pg.limit
        )
        .fetch_all(&app.pool)
        .await?;

        Ok(res)
    };

    Ok(Json(PagRes::new_async_with_err(pag.pag, f).await?))
}

#[utoipa::path(post,
    tag = "transaction",
    path = "/{room_id}/upd",
    description = "Change money or bio for transaction. For use only 'from_member' or 'to_member'. And (now - created_at) < 10d",
    request_body = MakeTransactionDto,
    responses(
        (status = 200),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room or member not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn upd_transaction(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(transaction_id): Path<Uuid>,
    Json(upd): Json<UpdTransactionDto>,
) -> Result<(), AppErr> {
    upd.validate()?;

    let res = sqlx::query!(
        r#"
        update transactions as t set
            money = case
                when created_at > now() - interval '10 days'
                    then coalesce($3, money)
                    else money
                end,
            bio = case
                when created_at > now() - interval '10 days'
                    then coalesce($4, bio)
                    else bio
                end
        where
            t.id = $2 and
            permit_room($1, t.room_id) and
            exists (
                select 1
                from members as m
                where
                    (m.id = t.from_member_id and m.user_id = $1)
                    or
                    (m.id = t.to_member_id and m.user_id = $1)
            )
        returning created_at > now() - interval '10 days' as "updated!"
        "#,
        user.user,
        transaction_id,
        upd.money,
        upd.bio
    )
    .fetch_optional(&app.pool)
    .await?
    .ok_or(AppErr::NotFound)?;

    if res.updated {
        Ok(())
    } else {
        Err(AppErr::CustomValidate("very old transaction"))
    }
}
