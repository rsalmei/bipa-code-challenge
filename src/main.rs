mod connectivity;
mod errors;
mod routes;
mod utils;

use anyhow::Result;
use axum::Router;
use axum::routing::get;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// The interval at which to update the nodes connectivity data.
const NODES_CONNECTIVITY_UPDATE_PERIOD: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing subscriber for logging.
    tracing_subscriber::registry()
        .with(fmt::layer()) // full format.
        // .with(fmt::layer().compact())
        // .with(fmt::layer().pretty())
        .with(tracing_subscriber::EnvFilter::new("info,tower_http=debug")) // default to info, but http requests at debug level.
        .init();

    // use the endpoint specified in an environment variable, or default to `memory`.
    let endpoint = std::env::var("SURREALDB").unwrap_or_else(|_| "memory".to_owned());
    let db = surrealdb::engine::any::connect(endpoint).await?;
    db.use_ns("namespace").use_db("database").await?;

    // spawn the node connectivity update task.
    tokio::spawn(utils::periodic_task(NODES_CONNECTIVITY_UPDATE_PERIOD, {
        let db = db.clone(); // the surreal db handle is cheap to clone, just an Arc internally.
        move || connectivity::update_nodes_connectivity_task(db.clone())
    })); // ahh, much better! a generic periodic task spawner.

    // start an axum server, so we can query the local database from outside.
    let listener = TcpListener::bind("0.0.0.0:3123").await?;
    info!("listening on {}", listener.local_addr()?);

    let app = Router::new()
        .route("/nodes", get(routes::get_nodes_connectivity_handler))
        .layer(TraceLayer::new_for_http()) // automatically logs requests/responses (must be after all routes are defined).
        .with_state(db);

    axum::serve(listener, app).await?;
    Ok(())
}
