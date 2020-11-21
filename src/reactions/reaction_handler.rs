use serenity::{
    client::Context,
    framework::standard::CommandResult,
    model::channel::{Reaction, ReactionType},
};

use crate::{helpers::giveaway_helper, reactions::giveaway};

pub async fn dispatch_reaction(ctx: &Context, reaction: &Reaction, _remove: bool) -> CommandResult {
    if reaction.user(&ctx).await.unwrap().bot {
        return Ok(());
    }

    if let ReactionType::Unicode(name) = &reaction.emoji {
        match name.as_str() {
            "🎉" => {
                if _remove {
                    let user_id = reaction.user_id.unwrap();

                    giveaway_helper::remove_giveaway_entries(ctx, &user_id).await?;
                } else {
                    giveaway::add_giveaway_entries(ctx, reaction).await?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
