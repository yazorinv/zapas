mod app;
mod background;
mod domain;
mod dto;
mod err;
mod http;
mod repo;

use crate::{app::App, err::GlobalErr, http::server};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), GlobalErr> {
    init_fmt();
    dotenv();

    let app = App::from_env().await?;

    background::background(app.clone());
    server(app).await?;

    Ok(())
}

/// Инициализация логгирования программы.
fn init_fmt() {
    println!(
        r#"
-----------------------------------------------------------

╺━┓┏━┓┏━┓┏━┓┏━┓
┏━┛┣━┫┣━┛┣━┫┗━┓
┗━╸╹ ╹╹  ╹ ╹┗━┛

version: {}

-----------------------------------------------------------

"#,
        env!("CARGO_PKG_VERSION")
    );

    tracing_subscriber::fmt()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_max_level(tracing::Level::TRACE)
        .pretty()
        .init();
    warn!("Session is coming");
}

/// Загрузка переменных окружения из `.env` файла.
fn dotenv() {
    match dotenvy::dotenv() {
        Ok(_) => {
            info!("Import .env")
        }
        Err(err) => {
            warn!("Import .env failed: {}", err);
        }
    }
}
