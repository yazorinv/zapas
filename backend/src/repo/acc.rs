use crate::{domain::acc::*, err::AppErr};
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Clone)]
pub struct RawStats {
    pub members: Vec<Uuid>,
    pub portions: Vec<Portion>,
    pub trs: Vec<Transaction>,
    pub prs: Vec<Purchase>,
}

pub trait AccRepo {
    fn raw_stats(
        &self,
        room: Uuid,
        l: NaiveDate,
        r: NaiveDate,
    ) -> impl Future<Output = Result<RawStats, AppErr>> + Send;
}

pub mod pg {
    use super::*;
    use sqlx::PgPool;

    #[derive(Clone)]
    pub struct AccRepoPg {
        pool: PgPool,
    }

    impl AccRepoPg {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }
    }

    impl AccRepo for AccRepoPg {
        async fn raw_stats(
            &self,
            room: Uuid,
            l: NaiveDate,
            r: NaiveDate,
        ) -> Result<RawStats, AppErr> {
            let mut tx = self.pool.begin().await?;

            let members = sqlx::query_scalar!("select id from members where room_id = $1", room)
                .fetch_all(&mut *tx)
                .await?;

            let portions = sqlx::query_as!(
                Portion,
                r#"
                select
                    p.member_id as member,
                    p.portion_date as date,
                    p.cnt
                from portions as p
                join members as m on m.id = p.member_id
                where m.room_id = $1 and
                $2::date <= p.portion_date and
                p.portion_date <= $3::date"#,
                room,
                l,
                r
            )
            .fetch_all(&mut *tx)
            .await?;

            let trs = sqlx::query_as!(
                Transaction,
                r#"
                select
                    to_member_id as member_to,
                    from_member_id as member_from,
                    money
                from transactions
                where room_id = $1 and
                $2::date <= created_at::date and
                created_at::date <= $3::date"#,
                room,
                l,
                r
            )
            .fetch_all(&mut *tx)
            .await?;

            let prs = sqlx::query_as!(
                Purchase,
                r#"
                select
                    member_id as member,
                    money,
                    is_food,
                    created_at::date as "date!"
                from purchases
                where room_id = $1 and
                $2::date <= created_at::date and
                created_at::date <= $3::date"#,
                room,
                l,
                r
            )
            .fetch_all(&mut *tx)
            .await?;

            Ok(RawStats {
                members,
                portions,
                trs,
                prs,
            })
        }
    }
}
