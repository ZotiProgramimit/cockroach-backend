use axum::{routing::get, Router};
use dotenvy::dotenv;
use rand::Rng;                             // still used for HTTP /ready RNG demo
use std::{env, sync::Arc};
use tokio::task;
use tonic::{transport::Server as TonicServer, Request, Response, Status};
use uuid::Uuid;

/* ---------------------------- internal modules ----------------------------- */
mod proto_mod;
mod db;
mod plinko_logic;   //  ← new

use proto_mod::plinko::{
    plinko_service_server::{PlinkoService, PlinkoServiceServer},
    BalanceRequest, BalanceResponse, PlayRequest, PlayResponse,
};

use plinko_logic::{simulate, GameMode};

struct PlinkoSrv {
    crdb: sqlx::PgPool,
    scy:  Arc<scylla::Session>,
}

#[tonic::async_trait]
impl PlinkoService for PlinkoSrv {
    async fn play(
        &self,
        req: Request<PlayRequest>,
    ) -> Result<Response<PlayResponse>, Status> {
        let payload = req.into_inner();

        /* ----------------------------- validate uid ---------------------------- */
        let user_id = Uuid::parse_str(&payload.user_id)
            .map_err(|_| Status::invalid_argument("bad uuid"))?;
        let bet: i64 = payload.bet_u64 as i64;
        let mode = GameMode::try_from(payload.mode)
            .map_err(|_| Status::invalid_argument("invalid mode"))?;

        /* ------------------------------ 1️⃣ TX begin --------------------------- */
        let mut tx = self.crdb.begin().await.map_err(int_err)?;

        /* --------------------------- 2️⃣ fetch balance ------------------------- */
        let (bal,): (i64,) = sqlx::query_as("SELECT balance FROM accounts WHERE id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| Status::not_found("no such user"))?;

        if bal < bet {
            return Err(Status::failed_precondition("insufficient funds"));
        }

        /* --------------------------- 3️⃣ simulate game ------------------------- */
        let (payout, slot) = simulate(mode, bet);

        /* --------------------------- 4️⃣ persist balance ----------------------- */
        let new_bal = bal - bet + payout;
        sqlx::query("UPDATE accounts SET balance = $2 WHERE id = $1")
            .bind(user_id)
            .bind(new_bal)
            .execute(&mut *tx)
            .await
            .map_err(int_err)?;

        tx.commit().await.map_err(int_err)?;

        /* ---------------------------- 5️⃣ async log --------------------------- */
        let scy = Arc::clone(&self.scy);
        task::spawn(async move {
            let _ = scy.query(
                "INSERT INTO casino.plinko_events (user_id, ts, bet, payout, slot)
                 VALUES (?, toTimestamp(now()), ?, ?, ?)",
                (user_id, bet, payout, slot as i32),
            ).await;
        });

        Ok(Response::new(PlayResponse {
            payout_u64:      payout as u64,
            slot_index:      slot as i32,
            new_balance_u64: new_bal as u64,
        }))
    }

    async fn get_balance(
        &self,
        req: Request<BalanceRequest>,
    ) -> Result<Response<BalanceResponse>, Status> {
        let user_id = Uuid::parse_str(&req.into_inner().user_id)
            .map_err(|_| Status::invalid_argument("bad uuid"))?;

        let (bal,): (i64,) = sqlx::query_as("SELECT balance FROM accounts WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.crdb)
            .await
            .map_err(|_| Status::not_found("no such user"))?;

        Ok(Response::new(BalanceResponse {
            balance_u64: bal as u64,
        }))
    }
}

fn int_err<E: std::fmt::Display>(e: E) -> Status {
    Status::internal(e.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let cockroach_url = env::var("COCKROACH_URL")
        .unwrap_or_else(|_| "postgres://root@localhost:26257/casino?sslmode=disable".into());
    let scylla_nodes = env::var("SCYLLA_NODES").unwrap_or_else(|_| "127.0.0.1:9042".into());

    let crdb = db::cockroach::init_cockroach(&cockroach_url).await?;
    let scy  = db::scylla::init_scylla(&scylla_nodes).await?;

    /* ------------------------------- gRPC ------------------------------------ */
    let grpc_srv = PlinkoSrv { crdb: crdb.clone(), scy };
    task::spawn(async move {
        TonicServer::builder()
            .add_service(PlinkoServiceServer::new(grpc_srv))
            .serve("[::]:50051".parse().unwrap())
            .await
            .unwrap();
    });

    /* ------------------------------- /ready ---------------------------------- */
    let app      = Router::new().route("/ready", get(|| async { "ok" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
