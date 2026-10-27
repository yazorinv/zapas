//! Модуль с реализацией одноименной структуры.
use crate::{
    domain::{auth::JwtService, invite::InviteService, sse::SseService},
    err::GlobalErr,
};
use sqlx::PgPool;
use tracing::info;

/// Общее состояние программы.
#[derive(Clone)]
pub struct App {
    pub pool: PgPool,

    pub jwt: JwtService,
    pub invite: InviteService,

    pub sse: SseService,

    pub user_repo: crate::repo::user::pg::UserRepoPg,
    pub session_repo: crate::repo::session::pg::SessionRepoPg,
    pub member_repo: crate::repo::member::pg::MemberRepoPg,

    pub acc_repo: crate::repo::acc::pg::AccRepoPg,
}

impl App {
    /// Создание нового состояния.
    pub async fn new(
        db_url: &str,
        auth_secret: &str,
        invite_secret: &str,
    ) -> Result<Self, GlobalErr> {
        let pool = sqlx::Pool::connect(db_url).await?;

        let res = Self {
            pool: pool.clone(),

            jwt: JwtService::new(auth_secret),
            invite: InviteService::new(invite_secret),
            sse: SseService::new(),

            user_repo: crate::repo::user::pg::UserRepoPg::new(pool.clone()),
            session_repo: crate::repo::session::pg::SessionRepoPg::new(pool.clone()),
            member_repo: crate::repo::member::pg::MemberRepoPg::new(pool.clone()),

            acc_repo: crate::repo::acc::pg::AccRepoPg::new(pool.clone()),
        };

        res.migration().await?;
        info!("Create App State OK");
        Ok(res)
    }

    /// Создание нового состояния. Переменные берутся из окружения.
    pub async fn from_env() -> Result<Self, GlobalErr> {
        Self::new(
            &std::env::var("DATABASE_URL")?,
            &std::env::var("AUTH_JWT")?,
            &std::env::var("INVITE_JWT")?,
        )
        .await
    }

    /// Автоматическое применение миграций.
    async fn migration(&self) -> Result<(), GlobalErr> {
        info!("Start migration");
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        info!("Finish migration");
        Ok(())
    }
}
