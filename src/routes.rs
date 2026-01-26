use crate::connectivity::NodeConnectivity;
use axum::{Json, extract::State, http::StatusCode};
use chrono::DateTime;
use serde::Serialize;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// The friendly node connectivity structure that will be returned by the API.
#[derive(Debug, Serialize)]
pub struct FriendlyNodeConnectivity {
    pub public_key: String,
    pub alias: String,
    /// Capacity in BTC, converted from satoshi, divided by 100M.
    pub capacity: f64,
    /// First seen datetime in ISO 8601 format.
    pub first_seen: String,
}

/// Handler for the GET /nodes endpoint.
pub async fn get_nodes_connectivity_handler(
    State(db): State<Surreal<Any>>,
) -> Result<Json<Vec<FriendlyNodeConnectivity>>, AppError> {
    // fetch current nodes connectivity data from the database.
    let nodes: Vec<NodeConnectivity> = db
        .select("ln_node_connectivity")
        .await
        .map_err(AppError::Database)?;

    // transform data for the response.
    let response = nodes
        .into_iter()
        .map(|node| {
            let first_seen = DateTime::from_timestamp(node.first_seen, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "Invalid Date".to_string());

            FriendlyNodeConnectivity {
                public_key: node.public_key,
                alias: node.alias,
                capacity: node.capacity as f64 / 100_000_000.0,
                first_seen,
            }
        })
        .collect();

    Ok(Json(response))
}
