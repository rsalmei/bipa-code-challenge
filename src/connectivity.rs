use anyhow::Result;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// The Mempool Space API endpoint for fetching Lightning Network nodes connectivity data.
const NODES_CONNECTIVITY_API: &str =
    "https://mempool.space/api/v1/lightning/nodes/rankings/connectivity";

/// Represents the connectivity information of a Lightning Network node (a subset of the full data).
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeConnectivity {
    pub public_key: String,
    pub alias: String,
    pub capacity: u64,
    pub first_seen: i64,
    pub updated_at: i64,
}

/// Updates the local database with the latest connectivity data of Lightning Network nodes.
pub async fn update_nodes_connectivity_task(db: Surreal<Any>) -> Result<()> {
    let nodes = fetch_nodes_connectivity().await?;
    if nodes.is_empty() {
        anyhow::bail!("fetched zero nodes connectivity data from API");
    }

    // yes, I do use unwrap in production code, but very consciously. I even always comment why it's
    // safe to do so like it was an unsafe block. This reassures code reviewers and future me that
    // it's not an oversight but a deliberate choice, that has been thought through and is
    // guaranteed to be safe. I do this for any functions that may panic, like unwraps, arrays
    // indexing, Vec::swap_remove, etc.
    // I consider panics to be embarrassing, so no software of mine will ever panic, at most they
    // will gracefully shut down or degrade functionality.
    // I also use unwraps a lot at compile time, like for LazyLock initializers with Regexes using
    // fixed patterns, that must be correct or are a programming error that can only be fixed by the
    // developer (me :-).
    let max_updated_at = nodes.iter().map(|node| node.updated_at).max().unwrap(); // SAFETY: nodes is not empty.
    println!("fetched nodes connectivity data: {}", max_updated_at);

    // upsert each node's connectivity data into the database.
    // this is necessary because they are ranked by connectivity quality (number of open channels),
    // and the API clips at a fixed number of 100 nodes; this means the returned nodes are likely to
    // change over time, and thus we need to update already seen ones, as well as insert the others.
    for node in nodes {
        let _: Option<NodeConnectivity> = db
            // yeah, I didn't like the surrealdb upsert API either, this discard with a type is
            // mandatory so the compiler know which type the .content() method should return.
            .upsert(("ln_node_connectivity", &node.public_key))
            .content(node)
            .await?;
    }

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
