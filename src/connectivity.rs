use anyhow::Result;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tracing::info;

/// The Mempool Space API endpoint for fetching Lightning Network nodes connectivity data.
const NODES_CONNECTIVITY_API: &str =
    "https://mempool.space/api/v1/lightning/nodes/rankings/connectivity";

/// The Mempool Space API for Lightning Network node connectivity data.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct MempoolNodeConnectivity {
    public_key: String,
    alias: String,
    capacity: u64,
    first_seen: i64,
    updated_at: i64,
}

/// The local Lightning Network node connectivity structure, stored in the database and decoupled
/// from the remote API structure.
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeConnectivity {
    pub public_key: String,
    pub alias: String,
    pub capacity: u64,
    pub first_seen: i64,
    pub updated_at: i64,
}

impl From<MempoolNodeConnectivity> for NodeConnectivity {
    fn from(mempool_node: MempoolNodeConnectivity) -> Self {
        Self {
            public_key: mempool_node.public_key,
            alias: mempool_node.alias,
            capacity: mempool_node.capacity,
            first_seen: mempool_node.first_seen,
            updated_at: mempool_node.updated_at,
        }
    }
}

/// Updates the local database with the latest connectivity data of Lightning Network nodes.
pub async fn update_nodes_connectivity_task(db: Surreal<Any>) -> Result<()> {
    let nodes = reqwest::get(NODES_CONNECTIVITY_API)
        .await?
        .json::<Vec<MempoolNodeConnectivity>>()
        .await?;
    info!("fetched new nodes connectivity data, len: {}", nodes.len());
    if nodes.is_empty() {
        anyhow::bail!("fetched zero nodes connectivity data from API");
    }

    // convert the fetched nodes into our local structure, decoupling from the API structure, then
    // upsert each node's connectivity data into the database.
    // this is necessary because they are ranked by connectivity quality (number of open channels),
    // and the API clips at a fixed number of 100 nodes; this means the returned nodes are likely to
    // change over time, and thus we need to update already seen ones, as well as insert the others.
    for node in nodes.into_iter().map(NodeConnectivity::from) {
        // the query notation does not suffer from the mandatory return type of the upsert method.
        db.query("UPSERT ln_node_connectivity CONTENT $node RETURN NONE")
            .bind(("node", node))
            .await?;
    }

    Ok(())
}
