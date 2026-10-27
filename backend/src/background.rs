mod portions;
mod sse;

use crate::app::App;

pub fn background(app: App) {
    tokio::spawn(sse::sse_listener(app.clone()));
    tokio::spawn(portions::set_portions(app.clone()));
}
