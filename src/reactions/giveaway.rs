use std::time::{SystemTime, UNIX_EPOCH};

use serenity::{framework::standard::CommandResult, model::prelude::*, prelude::*};
use uuid::Uuid;

use crate::structures::cmd_data::ConnectionPool;

pub async fn add_giveaway_entries(ctx: &Context, reaction: &Reaction) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let giveaway_data = sqlx::query!(
        "SELECT pre_expiry FROM info WHERE message_id = $1",
        reaction.message_id.0 as i64
    )
    .fetch_optional(&pool)
    .await?;

    match giveaway_data {
        Some(giveaway_data) => {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards?")
                .as_secs();

            if current_time > giveaway_data.pre_expiry as u64 {
                return Ok(());
            }
        }
        None => return Ok(()),
    }

    let guild = ctx.cache.guild(reaction.guild_id.unwrap()).await.unwrap();
    let user_id = reaction.user_id.unwrap();

    // Bias role ID
    let noble_id = RoleId::from(554090254664073237);

    let member = guild.member(ctx, user_id).await?;

    if member.roles.contains(&noble_id) {
        sqlx::query!(
            "INSERT INTO entries VALUES($1, $2, $3)",
            Uuid::new_v4(),
            user_id.0 as i64,
            reaction.message_id.0 as i64
        )
        .execute(&pool)
        .await?;
    }

    sqlx::query!(
        "INSERT INTO entries VALUES($1, $2, $3)",
        Uuid::new_v4(),
        user_id.0 as i64,
        reaction.message_id.0 as i64
    )
    .execute(&pool)
    .await?;

    // Entry dump channel id
    let dump_id = ChannelId::from(779858404290068541);

    dump_id
        .say(
            ctx,
            format!("{} has entered the giveaway", user_id.mention()),
        )
        .await?;

    println!("Sucessfully added entry/ies!");

    Ok(())
}
