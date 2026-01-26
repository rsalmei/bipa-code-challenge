use crate::connectivity::NodeConnectivity;
use crate::errors::AppError;
use axum::Json;
use axum::extract::{Query, State};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
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

/// Filter parameters for the GET /nodes endpoint.
#[derive(Debug, Deserialize)]
pub struct NodeConnectivityFilter {
    min_capacity: Option<f64>, // in BTC, as returned by the API.
}

#[derive(Debug, Deserialize)]
pub struct NodeConnectivitySort {
    order: Option<String>,
}

/// Handler for the GET /nodes endpoint.
#[tracing::instrument(skip(db))] // do not log the database handle.
pub async fn get_nodes_connectivity_handler(
    State(db): State<Surreal<Any>>,
    Query(filter): Query<NodeConnectivityFilter>,
    Query(sort): Query<NodeConnectivitySort>,
) -> Result<Json<Vec<FriendlyNodeConnectivity>>, AppError> {
    if let Some(min_capacity) = filter.min_capacity
        && min_capacity < 0.0
    {
        return Err(AppError::ValueError(
            "min_capacity must be non-negative".to_owned(),
        ));
    }

    // prepare the SQL query based on the filter, unfortunately allocating memory given the dynamic
    // filter and sort parameters; it currently has six possible combinations already, too much for
    // static strings alone (2 filter options * 3 sort options).
    let mut sql = "SELECT * FROM ln_node_connectivity".to_owned();
    if filter.min_capacity.is_some() {
        sql.push_str("\nWHERE capacity >= $min_capacity")
    };
    match sort.order.as_deref() {
        Some("capacity") => sql.push_str("\nORDER BY capacity DESC"),
        Some("first_seen") => sql.push_str("\nORDER BY first_seen ASC"),
        _ => sql.push_str("\nORDER BY alias ASC"),
    }

    // fetch current nodes connectivity data from the database, using the SQL query prepared above.
    let mut response = db
        .query(&sql)
        .bind((
            "min_capacity", // no problem binding vars that are not used in the query.
            filter.min_capacity.map(|c| (c * 100_000_000.0) as u64),
        ))
        .await
        .map_err(AppError::Database)?;

    // extract the result from the first statement in the query.
    let nodes: Vec<NodeConnectivity> = response.take(0).map_err(AppError::Database)?;

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
