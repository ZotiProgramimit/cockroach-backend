use sqlx::{PgPool, Executor};
use anyhow::Result;

pub async fn init_cockroach(url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(url).await?;

    // ─ balance is now BIGINT (nano-chips) ─
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            id       UUID PRIMARY KEY,
            username STRING UNIQUE NOT NULL,
            balance  BIGINT NOT NULL DEFAULT 0
        );
        "#,
    )
    .await?;

    pool.execute(
        r#"
        INSERT INTO accounts (id, username, balance)
        VALUES ('00000000-0000-0000-0000-000000000001', 'demo', 100000)
        ON CONFLICT (id) DO NOTHING;
        "#,
    )
    .await?;

    Ok(pool)
}
