use std::time::{SystemTime, UNIX_EPOCH};

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::structures::cmd_data::ConnectionPool;

#[command]
#[owners_only]
async fn setup(ctx: &Context, msg: &Message) -> CommandResult {
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
        .as_secs() as i64;

    sqlx::query!(
        "INSERT INTO info VALUES($1, $2, $3)",
        msg.id.0 as i64,
        current_time + 518400,
        current_time + 604800
    )
    .execute(&pool)
    .await?;

    let giveaway_msg = channel_id.say(ctx, giveaway_text).await?;

    let emoji = ReactionType::Unicode("🎉".to_string());

    giveaway_msg.react(ctx, emoji).await?;

    //TODO: Add spawn function to start the delay for giveaway locking and giveaway ending

    Ok(())
}
