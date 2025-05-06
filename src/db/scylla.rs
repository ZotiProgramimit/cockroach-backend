use scylla::{Session, SessionBuilder};
use anyhow::Result;
use std::sync::Arc;

pub async fn init_scylla(nodes: &str) -> Result<Arc<Session>> {
    let session = SessionBuilder::new()
        .known_nodes(&[nodes])
        .build()
        .await?;

    session
        .query(
            "CREATE KEYSPACE IF NOT EXISTS casino
             WITH replication = {'class':'SimpleStrategy','replication_factor':'1'}",
            &[],
        )
        .await?;

    session
        .query(
            r#"
            CREATE TABLE IF NOT EXISTS casino.plinko_events (
                user_id uuid,
                ts      timestamp,
                bet     bigint,
                payout  bigint,
                slot    int,
                PRIMARY KEY (user_id, ts)
            ) WITH CLUSTERING ORDER BY (ts DESC)
            "#,
            &[],
        )
        .await?;

    Ok(Arc::new(session))
}
