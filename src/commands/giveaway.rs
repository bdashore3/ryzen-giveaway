use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use tokio::time::delay_for;

use crate::{helpers::giveaway_helper, structures::cmd_data::ConnectionPool};

#[command]
#[owners_only]
async fn setup(ctx: &Context, _msg: &Message) -> CommandResult {
    // Giveaway channel id
    let channel_id = ChannelId::from(779011629710311424);

    let giveaway_text = concat!(
        "Hi there! Welcome to the giveaway! \n",
        "To enter in the giveaway, please react to the 🎉 emoji below! \n\n",
        "Unreacting/leaving the server will make you lose your entry!"
    );

    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?")
        .as_secs();

    let lock_delay = 518400;
    let end_delay = 604800;

    let giveaway_msg = channel_id.say(ctx, giveaway_text).await?;

    sqlx::query!(
        "INSERT INTO info VALUES($1, $2, $3, $4)",
        giveaway_msg.id.0 as i64,
        giveaway_msg.channel_id.0 as i64,
        (lock_delay + current_time) as i64,
        (end_delay + current_time) as i64
    )
    .execute(&pool)
    .await?;

    let emoji = ReactionType::Unicode("🎉".to_string());

    // Entry dump channel id
    let dump_id = ChannelId::from(779851330080342016);

    dump_id
        .say(ctx, "New Givewaway \n-------------------------------------")
        .await?;

    giveaway_msg.react(ctx, emoji).await?;

    let ctx_clone = ctx.clone();
    let giveaway_msg_clone = giveaway_msg.clone();
    tokio::spawn(async move {
        delay_for(Duration::from_secs(lock_delay)).await;
        let _ = giveaway_helper::lock_giveaway(&ctx_clone, &giveaway_msg_clone).await;

        delay_for(Duration::from_secs(end_delay - lock_delay)).await;
        let _ = giveaway_helper::choose_winner(&ctx_clone, &giveaway_msg_clone).await;
    });

    Ok(())
}
