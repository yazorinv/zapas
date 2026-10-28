use crate::{
    domain::{pag::PagArg, purchase::*},
    err::AppErr,
};
use uuid::Uuid;

pub trait PurchRepo {
    fn permit_write(
        &self,
        id: Uuid,
        user: Uuid,
    ) -> impl Future<Output = Result<bool, AppErr>> + Send;

    fn permit_read(
        &self,
        id: Uuid,
        user: Uuid,
    ) -> impl Future<Output = Result<bool, AppErr>> + Send;

    fn new_purchase(
        &self,
        new: NewPurchase,
    ) -> impl Future<Output = Result<Purchase, AppErr>> + Send;

    fn delete_purchase(&self, id: Uuid) -> impl Future<Output = Result<Option<()>, AppErr>> + Send;

    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Option<Purchase>, AppErr>> + Send;

    fn upd(
        &self,
        id: Uuid,
        upd: UpdPurchase,
    ) -> impl Future<Output = Result<Purchase, AppErr>> + Send;

    fn list_for_room(
        &self,
        room: Uuid,
        pag: PagArg,
    ) -> impl Future<Output = Result<Vec<Purchase>, AppErr>> + Send;

    fn list_for_member(
        &self,
        member: Uuid,
        pag: PagArg,
    ) -> impl Future<Output = Result<Vec<Purchase>, AppErr>> + Send;

    fn list_for_user(
        &self,
        user: Uuid,
        pag: PagArg,
    ) -> impl Future<Output = Result<Vec<Purchase>, AppErr>> + Send;
}

pub mod pg {
    use super::*;
    use sqlx::PgPool;

    #[derive(Clone)]
    pub struct PurchRepoPg {
        pool: PgPool,
    }

    impl PurchRepoPg {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }
    }

    impl PurchRepo for PurchRepoPg {
        async fn permit_write(&self, id: Uuid, user: Uuid) -> Result<bool, AppErr> {
            let cnt = sqlx::query!(
                r#"
                select p.id
                from purchases as p
                join members as m on m.id = p.member_id
                where p.id = $1 and m.user_id = $2"#,
                id,
                user
            )
            .fetch_optional(&self.pool)
            .await?;
            Ok(cnt.is_some())
        }

        async fn permit_read(&self, id: Uuid, user: Uuid) -> Result<bool, AppErr> {
            let cnt = sqlx::query!(
                r#"
                select p.id
                from purchases as p
                join members as m on m.room_id = p.room_id
                where p.id = $1 and m.user_id = $2"#,
                id,
                user
            )
            .fetch_optional(&self.pool)
            .await?;
            Ok(cnt.is_some())
        }

        async fn get_by_id(&self, id: Uuid) -> Result<Option<Purchase>, AppErr> {
            let res = sqlx::query_as!(
                Purchase,
                r#"
                select
                    id,
                    room_id as room,
                    member_id as member,
                    money,
                    bio,
                    is_food,
                    created_at
                from purchases
                where id = $1"#,
                id
            )
            .fetch_optional(&self.pool)
            .await?;

            Ok(res)
        }

        async fn delete_purchase(&self, id: Uuid) -> Result<Option<()>, AppErr> {
            let cnt = sqlx::query!("delete from purchases where id = $1", id)
                .execute(&self.pool)
                .await?
                .rows_affected();

            Ok(if cnt == 0 { None } else { Some(()) })
        }

        async fn list_for_member(
            &self,
            member: Uuid,
            pag: PagArg,
        ) -> Result<Vec<Purchase>, AppErr> {
            let res = sqlx::query_as!(
                Purchase,
                r#"
                select
                    id,
                    room_id as room,
                    member_id as member,
                    money,
                    bio,
                    is_food,
                    created_at
                from purchases
                where member_id = $1
                order by created_at desc
                offset $2 limit $3"#,
                member,
                pag.offset,
                pag.limit
            )
            .fetch_all(&self.pool)
            .await?;
            Ok(res)
        }

        async fn list_for_room(&self, room: Uuid, pag: PagArg) -> Result<Vec<Purchase>, AppErr> {
            let res = sqlx::query_as!(
                Purchase,
                r#"
                select
                    id,
                    room_id as room,
                    member_id as member,
                    money,
                    bio,
                    is_food,
                    created_at
                from purchases
                where room_id = $1
                order by created_at desc
                offset $2 limit $3"#,
                room,
                pag.offset,
                pag.limit
            )
            .fetch_all(&self.pool)
            .await?;
            Ok(res)
        }

        async fn list_for_user(&self, user: Uuid, pag: PagArg) -> Result<Vec<Purchase>, AppErr> {
            let res = sqlx::query_as!(
                Purchase,
                r#"
                select
                    p.id,
                    p.room_id as room,
                    p.member_id as member,
                    p.money,
                    p.bio,
                    p.is_food,
                    p.created_at
                from purchases as p
                join members as m on p.member_id = m.id
                where m.user_id = $1
                order by created_at desc
                offset $2 limit $3"#,
                user,
                pag.offset,
                pag.limit
            )
            .fetch_all(&self.pool)
            .await?;
            Ok(res)
        }

        async fn new_purchase(&self, new: NewPurchase) -> Result<Purchase, AppErr> {
            let res = sqlx::query_as!(
                Purchase,
                r#"
                insert into purchases as p(
                    id,
                    room_id,
                    member_id,
                    money,
                    bio,
                    is_food
                ) select
                    $1,
                    $2,
                    m.id,
                    $4,
                    $5,
                    $6
                from members as m
                where m.id = $3
                returning
                    id,
                    room_id as room,
                    member_id as member,
                    money,
                    bio,
                    is_food,
                    created_at"#,
                Uuid::now_v7(),
                new.room,
                new.user,
                new.money,
                new.bio,
                new.is_food
            )
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppErr::NotFound)?;
            Ok(res)
        }

        async fn upd(&self, id: Uuid, upd: UpdPurchase) -> Result<Purchase, AppErr> {
            let res = sqlx::query_as!(
                Purchase,
                r#"
                update purchases set
                    money = coalesce($2, money),
                    bio = coalesce($3, bio),
                    is_food = coalesce($4, is_food)
                where id = $1
                returning
                    id,
                    room_id as room,
                    member_id as member,
                    money,
                    bio,
                    is_food,
                    created_at"#,
                id,
                upd.money,
                upd.bio,
                upd.is_food
            )
            .fetch_one(&self.pool)
            .await?;
            Ok(res)
        }
    }
}
