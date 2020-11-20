use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use sqlx::*;
use uuid::Uuid;

use crate::structures::cmd_data::ConnectionPool;

pub async fn add_giveaway_entry(ctx: &Context, reaction: &Reaction) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let check = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM info WHERE message_id = $1)",
        reaction.message_id.0 as i64
    )
    .fetch_one(&pool)
    .await?;

    if !check.exists.unwrap() {
        println!("Doesn't exist!");
        return Ok(());
    }

    println!("Exists in db!");

    let guild = ctx.cache.guild(reaction.guild_id.unwrap()).await.unwrap();
    let user_id = reaction.user_id.unwrap();
    let noble_id = RoleId::from(779011688132771921);

    let member = guild.member(ctx, user_id).await?;

    if member.roles.contains(&noble_id) {
        sqlx::query!(
            "INSERT INTO entries VALUES($1, $2)",
            Uuid::new_v4(),
            user_id.0 as i64
        )
        .execute(&pool)
        .await?;
    }

    sqlx::query!(
        "INSERT INTO entries VALUES($1, $2)",
        Uuid::new_v4(),
        user_id.0 as i64
    )
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn remove_giveaway_entry(ctx: &Context, reaction: &Reaction) -> CommandResult {
    Ok(())
}
