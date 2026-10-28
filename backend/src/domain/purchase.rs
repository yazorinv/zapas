use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct Purchase {
    pub id: Uuid,
    pub room: Uuid,
    pub member: Uuid,

    pub money: i64,
    pub is_food: bool,
    pub bio: String,

    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct NewPurchase {
    pub user: Uuid,
    pub room: Uuid,

    pub money: i64,
    pub is_food: bool,
    pub bio: String,
}

#[derive(Clone)]
pub struct UpdPurchase {
    pub money: Option<i64>,
    pub is_food: Option<bool>,
    pub bio: Option<String>,
}
