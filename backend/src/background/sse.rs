use crate::{app::App, domain::sse::EventMsg};
use sqlx::postgres::PgListener;
use std::time::Duration;
use tracing::{error, info};

pub async fn sse_listener(app: App) {
    loop {
        if let Err(err) = iter(app.clone()).await {
            error!("Error sse listener iteration: {}, retry...", err);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn worker(app: App, msg: EventMsg) {
    if let Some(user) = msg.user_id {
        app.sse.send(user, msg);
    } else if let Some(room) = msg.room_id {
        let users = match sqlx::query!(
            r#"
            select id
            from users
            where permit_room(id, $1)"#,
            room
        )
        .fetch_all(&app.pool)
        .await
        {
            Ok(x) => x,
            Err(err) => {
                error!("Error sqlx in sse_notify: {}", err);
                return;
            }
        };

        for u in users {
            app.sse.send(u.id, msg);
        }
    } else {
        error!("Error EventMsg - not found user_id and room_id");
    }
}

async fn iter(app: App) -> Result<(), sqlx::Error> {
    let mut listener = PgListener::connect_with(&app.pool).await?;
    listener.listen("sse_notify").await?;

    info!("Start listen sse_notify");
    loop {
        let pg = listener.recv().await?;
        match serde_json::from_str(pg.payload()) {
            Ok(msg) => {
                tokio::spawn(worker(app.clone(), msg));
            }

            Err(err) => {
                error!("Error parse msg from sse_notify: {}", err);
                continue;
            }
        };
    }
}
