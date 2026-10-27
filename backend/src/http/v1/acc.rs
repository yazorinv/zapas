use super::*;
use crate::{
    domain::{acc::*, auth::UserSession},
    err::{AppErr, AppErrJson},
    repo::{acc::AccRepo, member::MemberRepo},
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    middleware,
    routing::get,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub fn router(app: App) -> Router {
    Router::new()
        .route("/{room_id}/balance", get(balance))
        .route("/{room_id}/recommend", get(recommend))
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware))
        .with_state(app)
}

#[derive(OpenApi)]
#[openapi(paths(balance, recommend))]
pub struct AccApi;

#[utoipa::path(get,
    tag = "accounting",
    path = "/{room_id}/balance",
    description = "Get stats in room for [l, r] times",
    params(
        TwoDateQuery,
    ),
    responses(
        (status = 200, body = RoomStatResp),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn balance(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    Query(dates): Query<TwoDateQuery>,
) -> Result<Json<RoomStatResp>, AppErr> {
    if !app.member_repo.permit_room(room_id, user.user).await? {
        return Err(AppErr::NotFound);
    }

    let l = dates.l.unwrap_or(min_date());
    let r = dates.r.unwrap_or(max_date());

    let raw = app.acc_repo.raw_stats(room_id, l, r).await?;
    let stats = calc_stats(raw.members, raw.portions, &raw.trs, raw.prs);

    let mut res = RoomStatResp {
        id: room_id,
        portions: stats.iter().map(|x| x.portions).sum(),
        food_spent: stats.iter().map(|x| x.food_spent).sum(),
        food_purchases: stats.iter().map(|x| x.food_purchases).sum(),
        not_food_purchases: stats.iter().map(|x| x.not_food_purchases).sum(),
        purchases: 0,
        transactions: stats.iter().map(|x| x.transactions).sum(),
        members: stats.into_iter().map(|x| x.into()).collect(),
    };
    res.purchases = res.food_purchases + res.not_food_purchases;

    Ok(Json(res))
}

#[utoipa::path(get,
    tag = "accounting",
    path = "/{room_id}/recommend",
    description = "List of transactions for averaging expenses",
    responses(
        (status = 200, body = RecomendTrsResp),
        (status = 500, body = AppErrJson, description = "Internal server error"),
        (status = 400, body = AppErrJson, description = "Invalid any data format (validation error)"),
        (status = 401, body = AppErrJson, description = "Token is missing or not verified (maybe invalid, expired, deleted or ip changed)"),
        (status = 404, body = AppErrJson, description = "Room not found"),
    ),
    security(
        ("jwt" = [])
    )
)]
async fn recommend(
    State(app): State<App>,
    Extension(user): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<RecomendTrsResp>, AppErr> {
    if !app.member_repo.permit_room(room_id, user.user).await? {
        return Err(AppErr::NotFound);
    }

    let raw = app
        .acc_repo
        .raw_stats(room_id, min_date(), max_date())
        .await?;

    let stats = calc_stats(raw.members, raw.portions, &raw.trs, raw.prs);
    let trs = accounting(&stats);

    Ok(Json(RecomendTrsResp {
        trs: trs.into_iter().map(|x| x.into()).collect(),
    }))
}

#[derive(Clone, Serialize, ToSchema)]
struct MemberStatResp {
    id: Uuid,

    portions: i64,
    food_spent: i64,

    food_purchases: i64,
    not_food_purchases: i64,

    transactions: i64,
}

#[derive(Clone, Serialize, ToSchema)]
struct RoomStatResp {
    id: Uuid,

    portions: i64,
    food_spent: i64,

    food_purchases: i64,
    not_food_purchases: i64,
    purchases: i64,

    transactions: i64,

    members: Vec<MemberStatResp>,
}

#[derive(Clone, Serialize, ToSchema)]
struct RecommendTrResp {
    member_from: Uuid,
    member_to: Uuid,
    money: i64,
}

#[derive(Clone, Serialize, ToSchema)]
struct RecomendTrsResp {
    trs: Vec<RecommendTrResp>,
}

#[derive(Clone, Deserialize, IntoParams)]
pub struct TwoDateQuery {
    l: Option<NaiveDate>,
    r: Option<NaiveDate>,
}

impl From<MemberStat> for MemberStatResp {
    fn from(x: MemberStat) -> Self {
        Self {
            id: x.id,
            portions: x.portions,
            food_spent: x.food_spent,
            food_purchases: x.food_purchases,
            not_food_purchases: x.not_food_purchases,
            transactions: x.transactions,
        }
    }
}

impl From<NeedTransaction> for RecommendTrResp {
    fn from(x: NeedTransaction) -> Self {
        Self {
            member_from: x.member_from,
            member_to: x.member_to,
            money: x.money,
        }
    }
}

fn min_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap()
}

fn max_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(9999, 1, 1).unwrap()
}
