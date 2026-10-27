use crate::app::App;
use std::time::Duration;
use tracing::{error, info};

pub async fn set_portions(app: App) {
    loop {
        let res = sqlx::query!(
            r#"
            insert into portions (
                member_id,
                portion_date,
                cnt
            ) select
                id,
                now()::date,
                default_portions
            from members
            where default_portions != 0
            on conflict (member_id, portion_date) do nothing"#
        )
        .execute(&app.pool)
        .await;

        match res {
            Ok(cnt) => {
                info!("Set portions, cnt: {}", cnt.rows_affected());
            }

            Err(err) => {
                error!("Error for set portions: {}", err);
            }
        }

        tokio::time::sleep(Duration::from_hours(2)).await;
    }
}
