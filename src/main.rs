mod utils;

use anyhow::Result;
use serde::Deserialize;
use std::time::Duration;

/// The interval at which to update the nodes connectivity data.
const NODES_CONNECTIVITY_UPDATE_INTERVAL: Duration = Duration::from_secs(30);
/// The Mempool Space API endpoint for fetching Lightning Network nodes connectivity data.
const NODES_CONNECTIVITY_API: &str =
    "https://mempool.space/api/v1/lightning/nodes/rankings/connectivity";

#[tokio::main]
async fn main() -> Result<()> {
    // spawn the node connectivity update task.
    tokio::spawn(utils::periodic_task(
        NODES_CONNECTIVITY_UPDATE_INTERVAL,
        update_nodes_connectivity,
    )); // ahh, much better! a generic periodic task spawner.

    tokio::time::sleep(Duration::from_hours(1)).await;

    Ok(())
}

/// Updates the local database with the latest connectivity data of Lightning Network nodes.
async fn update_nodes_connectivity() -> Result<()> {
    let nodes = fetch_nodes_connectivity().await?;
    if nodes.is_empty() {
        anyhow::bail!("fetched zero nodes connectivity data from API");
    }

    // yes, I do use unwrap in production code, but very consciously. I even always comment why it's
    // safe to do so like it was an unsafe block. This reassures code reviewers and future me that
    // it's not an oversight but a deliberate choice, that has been thought through and is guaranteed
    // to be safe. I do this for any functions that may panic, like unwraps, array indexing,
    // Vec::swap_remove, etc.
    // I consider panics to be embarrassing, so no software of mine will ever panic, at most they
    // will gracefully shut down or degrade functionality.
    // I also use unwraps a lot at compile time, like for LazyLock initializers with Regexes, that
    // must be correct or are a programming error that must be fixed before shipping.
    let max_updated_at = nodes.iter().map(|node| node.updated_at).max().unwrap(); // SAFETY: nodes is not empty.
    println!("fetched nodes connectivity data: {}", max_updated_at);

    // db store.

    Ok(())
}

/// Fetches the connectivity data of Lightning Network nodes from the Mempool API.
async fn fetch_nodes_connectivity() -> Result<Vec<NodeConnectivity>> {
    reqwest::get(NODES_CONNECTIVITY_API)
        .await?
        .json()
        .await
        .map_err(Into::into)
}

/// Represents the connectivity information of a Lightning Network node (a subset of the full data).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NodeConnectivity {
    pub public_key: String,
    pub alias: String,
    pub capacity: u64,
    pub first_seen: u64,
    pub updated_at: i64,
}
