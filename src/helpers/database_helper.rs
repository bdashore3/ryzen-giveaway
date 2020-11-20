use serenity::framework::standard::CommandResult;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn obtain_db_pool(db_connection: &str) -> CommandResult<PgPool> {
    let connection_string = &db_connection;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

    Ok(pool)
}
