use tracing::{info, instrument, Instrument};
use tracing_subscriber::{
    filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tracing::Level;
    use tracing_subscriber::filter::{self, filter_fn};

    let filter_out_hidden_fn = tracing_subscriber::filter::filter_fn(|metadata| {
        metadata.target() != "sqlx_execute_hidden_query"
    });

    let filter_hidden = tracing_subscriber::filter::Targets::new()
        .with_default(Level::DEBUG)
        .with_target("sqlx_execute_hidden_query", Level::INFO);

    tracing_subscriber::fmt::fmt()
        .with_env_filter("debug,[hidden]=info")
        .init();
    info!("hello from tracing");
    let instance_id = "identifier";
    // let span = tracing::span!(tracing::Level::INFO, "Diesel", instance = instance_id);
    // check_diesel().instrument(span).await?;

    let span = tracing::span!(tracing::Level::INFO, "SQLX", instance = instance_id);
    check_sqlx().instrument(span).await?;
    Ok(())
}

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

async fn check_sqlx() -> anyhow::Result<()> {
    tracing::info!("checking sqlx logging");
    let options = PgConnectOptions::default();
    let pool = PgPoolOptions::new().connect_with(options).await?;
    let span = tracing::span!(tracing::Level::INFO, "background");
    sqlx_execute_hidden_query(&pool).instrument(span).await?;

    sqlx_execute_logged_query(&pool).await?;

    Ok(())
}

async fn sqlx_execute_hidden_query(pool: &PgPool) -> anyhow::Result<()> {
    tracing::info!("this should be logged");
    let _out: String = sqlx::query_scalar("select 'DO NOT log this query'")
        .fetch_one(pool)
        .await?;
    Ok(())
}

async fn sqlx_execute_logged_query(pool: &PgPool) -> anyhow::Result<()> {
    tracing::debug!("debug should work");
    let _out: String = sqlx::query_scalar("select 'log this query'")
        .fetch_one(pool)
        .await?;
    Ok(())
}

use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};

async fn check_diesel() -> anyhow::Result<()> {
    let database_url = "postgres://dimaafanasev:admin@localhost/monolith";
    let mut connection = AsyncPgConnection::establish(&database_url).await?;
    info!("check_diesel");

    let _ = diesel_execute_hidden(&mut connection).await?;
    let _ = diesel_execute_logged(&mut connection).await?;
    Ok(())
}

async fn diesel_execute_hidden(conn: &mut AsyncPgConnection) -> anyhow::Result<()> {
    diesel::dsl::sql_query("select 'DO NOT log this query'")
        .execute(conn)
        .await?;
    Ok(())
}

async fn diesel_execute_logged(conn: &mut AsyncPgConnection) -> anyhow::Result<()> {
    diesel::dsl::sql_query("select 'log this query'")
        .execute(conn)
        .await?;
    Ok(())
}
