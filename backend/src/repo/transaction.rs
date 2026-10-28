use crate::{
    domain::{pag::PagArg, transaction::*},
    err::AppErr,
};
use uuid::Uuid;

pub trait TrRepo {
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

    fn new_transaction(
        &self,
        new: NewTransaction,
    ) -> impl Future<Output = Result<Transaction, AppErr>> + Send;

    fn delete_transaction(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<()>, AppErr>> + Send;

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Transaction>, AppErr>> + Send;

    fn upd_transaction(
        &self,
        id: Uuid,
        upd: UpdTransaction,
    ) -> impl Future<Output = Result<Transaction, AppErr>> + Send;

    fn list_transactions_for_room(
        &self,
        room: Uuid,
        pag: PagArg,
    ) -> impl Future<Output = Result<Vec<Transaction>, AppErr>> + Send;

    fn list_transactions_for_user(
        &self,
        user: Uuid,
        tp: TransactionType,
        pag: PagArg,
    ) -> impl Future<Output = Result<Vec<Transaction>, AppErr>> + Send;

    fn list_transactions_for_member(
        &self,
        member: Uuid,
        tp: TransactionType,
        pag: PagArg,
    ) -> impl Future<Output = Result<Vec<Transaction>, AppErr>> + Send;
}

pub mod pg {
    use super::*;
    use sqlx::PgPool;

    #[derive(Clone)]
    pub struct TrRepoPg {
        pool: PgPool,
    }

    impl TrRepo for TrRepoPg {
        async fn permit_write(&self, id: Uuid, user: Uuid) -> Result<bool, AppErr> {
            let cnt = sqlx::query!(
                r#"
                select t.id
                from transactions as t
                join members as m on (m.id = t.from_member_id or m.id = t.to_member_id)
                where t.id = $1 and m.user_id = $2"#,
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
                select t.id
                from transactions as t
                join members as m on m.room_id = t.room_id
                where t.id = $1 and m.user_id = $2"#,
                id,
                user
            )
            .fetch_optional(&self.pool)
            .await?;
            Ok(cnt.is_some())
        }

        async fn new_transaction(&self, new: NewTransaction) -> Result<Transaction, AppErr> {
            let res = sqlx::query_as!(
                Transaction,
                r#"
                insert into transactions (
                    id,
                    to_member_id,
                    from_member_id,
                    money,
                    bio
                ) values (
                    $1, $2, $3, $4, $5
                ) returning
                    id,
                    room_id as room,
                    to_member_id as member_to,
                    from_member_id as member_from,
                    money,
                    bio,
                    created_at"#,
                Uuid::now_v7(),
                new.member_to,
                new.member_from,
                new.money,
                new.bio
            )
            .fetch_one(&self.pool)
            .await?;

            Ok(res)
        }

        async fn delete_transaction(&self, id: Uuid) -> Result<Option<()>, AppErr> {
            let cnt = sqlx::query!("delete from transactions where id = $1", id)
                .execute(&self.pool)
                .await?
                .rows_affected();

            Ok(if cnt == 0 { None } else { Some(()) })
        }

        async fn get_by_id(&self, id: Uuid) -> Result<Option<Transaction>, AppErr> {
            let res = sqlx::query_as!(
                Transaction,
                r#"
                select
                    id,
                    room_id as room,
                    to_member_id as member_to,
                    from_member_id as member_from,
                    money,
                    bio,
                    created_at
                from transactions
                where id = $1"#,
                id
            )
            .fetch_optional(&self.pool)
            .await?;
            Ok(res)
        }

        async fn list_transactions_for_member(
            &self,
            member: Uuid,
            tp: TransactionType,
            pag: PagArg,
        ) -> Result<Vec<Transaction>, AppErr> {
            let res = match tp {
                TransactionType::From => {
                    sqlx::query_as!(
                        Transaction,
                        r#"
                    select
                        id,
                        room_id as room,
                        to_member_id as member_to,
                        from_member_id as member_from,
                        money,
                        bio,
                        created_at
                    from transactions
                    where from_member_id = $1
                    order by created_at desc
                    offset $2 limit $3"#,
                        member,
                        pag.offset,
                        pag.limit
                    )
                    .fetch_all(&self.pool)
                    .await?
                }
                TransactionType::To => {
                    sqlx::query_as!(
                        Transaction,
                        r#"
                    select
                        id,
                        room_id as room,
                        to_member_id as member_to,
                        from_member_id as member_from,
                        money,
                        bio,
                        created_at
                    from transactions
                    where to_member_id = $1
                    order by created_at desc
                    offset $2 limit $3"#,
                        member,
                        pag.offset,
                        pag.limit
                    )
                    .fetch_all(&self.pool)
                    .await?
                }
                TransactionType::Any => {
                    sqlx::query_as!(
                        Transaction,
                        r#"
                    select
                        id,
                        room_id as room,
                        to_member_id as member_to,
                        from_member_id as member_from,
                        money,
                        bio,
                        created_at
                    from transactions
                    where (from_member_id = $1 or to_member_id = $1)
                    order by created_at desc
                    offset $2 limit $3"#,
                        member,
                        pag.offset,
                        pag.limit
                    )
                    .fetch_all(&self.pool)
                    .await?
                }
            };

            Ok(res)
        }

        async fn list_transactions_for_room(
            &self,
            room: Uuid,
            pag: PagArg,
        ) -> Result<Vec<Transaction>, AppErr> {
            let res = sqlx::query_as!(
                Transaction,
                r#"
                select
                    id,
                    room_id as room,
                    to_member_id as member_to,
                    from_member_id as member_from,
                    money,
                    bio,
                    created_at
                from transactions
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

        async fn list_transactions_for_user(
            &self,
            user: Uuid,
            tp: TransactionType,
            pag: PagArg,
        ) -> Result<Vec<Transaction>, AppErr> {
            let res = match tp {
                TransactionType::From => {
                    sqlx::query_as!(
                        Transaction,
                        r#"
                    select
                        t.id,
                        t.room_id as room,
                        t.to_member_id as member_to,
                        t.from_member_id as member_from,
                        t.money,
                        t.bio,
                        t.created_at
                    from transactions as t
                    join members as m on m.id = t.from_member_id
                    where m.user_id = $1
                    order by created_at desc
                    offset $2 limit $3"#,
                        user,
                        pag.offset,
                        pag.limit
                    )
                    .fetch_all(&self.pool)
                    .await?
                }
                TransactionType::To => {
                    sqlx::query_as!(
                        Transaction,
                        r#"
                    select
                        t.id,
                        t.room_id as room,
                        t.to_member_id as member_to,
                        t.from_member_id as member_from,
                        t.money,
                        t.bio,
                        t.created_at
                    from transactions as t
                    join members as m on m.id = t.to_member_id
                    where m.user_id = $1
                    order by created_at desc
                    offset $2 limit $3"#,
                        user,
                        pag.offset,
                        pag.limit
                    )
                    .fetch_all(&self.pool)
                    .await?
                }
                TransactionType::Any => {
                    sqlx::query_as!(
                        Transaction,
                        r#"
                    select
                        t.id,
                        t.room_id as room,
                        t.to_member_id as member_to,
                        t.from_member_id as member_from,
                        t.money,
                        t.bio,
                        t.created_at
                    from transactions as t
                    join members as m on (m.id = t.to_member_id or m.id = t.from_member_id)
                    where m.user_id = $1
                    order by created_at desc
                    offset $2 limit $3"#,
                        user,
                        pag.offset,
                        pag.limit
                    )
                    .fetch_all(&self.pool)
                    .await?
                }
            };

            Ok(res)
        }

        async fn upd_transaction(
            &self,
            id: Uuid,
            upd: UpdTransaction,
        ) -> Result<Transaction, AppErr> {
            let res = sqlx::query_as!(
                Transaction,
                r#"
                update transactions set
                    money = coalesce($2, money),
                    bio = coalesce($3, bio)
                where id = $1
                returning
                    id,
                    room_id as room,
                    to_member_id as member_to,
                    from_member_id as member_from,
                    money,
                    bio,
                    created_at"#,
                id,
                upd.money,
                upd.bio
            )
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppErr::NotFound)?;
            Ok(res)
        }
    }
}
