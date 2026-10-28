use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub room: Uuid,

    pub member_from: Uuid,
    pub member_to: Uuid,

    pub money: i64,
    pub bio: String,

    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct NewTransaction {
    pub member_from: Uuid,
    pub member_to: Uuid,

    pub money: i64,
    pub bio: String,
}

#[derive(Clone)]
pub struct UpdTransaction {
    pub money: Option<i64>,
    pub bio: Option<String>,
}

#[derive(Clone, Copy)]
pub enum TransactionType {
    From,
    To,
    Any,
}
