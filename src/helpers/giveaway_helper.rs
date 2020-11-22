use rand::prelude::*;
use serenity::{framework::standard::CommandResult, model::prelude::*, prelude::*};

use crate::structures::cmd_data::ConnectionPool;

pub async fn lock_giveaway(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            ctx,
            concat!("The giveaway is now locked. The winner will be announced in 1 day from this message. \n",
                "Note: You can still leave the giveaway, but you cannot enter again once you leave!")
        )
        .await?;

    Ok(())
}

pub async fn choose_winner(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let giveaway_data = sqlx::query!(
        "SELECT * FROM entries WHERE message_id = $1",
        msg.id.0 as i64
    )
    .fetch_all(&pool)
    .await?;

    let mut rng = StdRng::from_entropy();

    if giveaway_data.len() <= 0 as usize {
        msg.channel_id
            .say(
                ctx,
                "Looks like this giveaway had no entries, so there's no winner!",
            )
            .await?;

        return Ok(());
    }

    let val = rng.gen_range(0, giveaway_data.len());

    let winning_data = giveaway_data.get(val).unwrap();
    let winning_user = UserId::from(winning_data.user_id as u64);

    msg.channel_id
        .say(
            ctx,
            format!(
                "{} has won the giveaway! Congratulations!",
                winning_user.mention()
            ),
        )
        .await?;

    sqlx::query!("DELETE FROM info WHERE message_id = $1", msg.id.0 as i64)
        .execute(&pool)
        .await?;

    Ok(())
}

pub async fn remove_giveaway_entries(ctx: &Context, user_id: &UserId) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<ConnectionPool>()
        .cloned()
        .unwrap();

    let check = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM entries WHERE user_id = $1)",
        user_id.0 as i64
    )
    .fetch_one(&pool)
    .await?;

    if check.exists.unwrap() {
        // Entry dump channel id
        let dump_id = ChannelId::from(779858404290068541);

        dump_id
            .say(ctx, format!("{} has left the giveaway", user_id.mention()))
            .await?;
    }

    sqlx::query!("DELETE FROM entries WHERE user_id = $1", user_id.0 as i64)
        .execute(&pool)
        .await?;

    Ok(())
}
