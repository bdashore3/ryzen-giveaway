use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serenity::{client::Context, framework::standard::CommandResult};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::time::delay_for;

use crate::structures::cmd_data::ConnectionPool;

use super::giveaway_helper;

pub async fn obtain_db_pool(db_connection: &str) -> CommandResult<PgPool> {
    let connection_string = &db_connection;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&connection_string)
        .await?;

    Ok(pool)
}

pub async fn restore_tasks(ctx: Context) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let giveaway_info = sqlx::query!("SELECT message_id, channel_id, pre_expiry, expiry FROM info")
        .fetch_all(&pool)
        .await?;

    for i in giveaway_info {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards?")
            .as_secs() as i64;

        let msg_id = i.message_id as u64;
        let channel_id = i.channel_id as u64;
        let msg = ctx.http.get_message(channel_id, msg_id).await.unwrap();

        if current_time > i.expiry {
            giveaway_helper::choose_winner(&ctx, &msg).await?;
        } else if current_time > i.pre_expiry && current_time < i.expiry {
            giveaway_helper::lock_giveaway(&ctx, &msg).await?;

            let time_diff = i.expiry - current_time;

            let ctx_clone = ctx.clone();
            tokio::spawn(async move {
                delay_for(Duration::from_secs(time_diff as u64)).await;

                let _ = giveaway_helper::choose_winner(&ctx_clone, &msg).await;
            });
        } else {
            let ctx_clone = ctx.clone();
            let msg_clone = msg.clone();

            tokio::spawn(async move {
                let lock_time_diff = i.pre_expiry - current_time;
                let expire_time_diff = i.expiry - current_time;

                delay_for(Duration::from_secs(lock_time_diff as u64)).await;
                let _ = giveaway_helper::lock_giveaway(&ctx_clone, &msg_clone).await;

                delay_for(Duration::from_secs(
                    (expire_time_diff - lock_time_diff) as u64,
                ))
                .await;
                let _ = giveaway_helper::choose_winner(&ctx_clone, &msg_clone).await;
            });
        }
    }

    Ok(())
}
