use serenity::{
    client::Context,
    framework::standard::CommandResult,
    model::channel::{Reaction, ReactionType},
};

use crate::reactions::giveaway;

pub async fn dispatch_reaction(ctx: &Context, reaction: &Reaction, _remove: bool) -> CommandResult {
    if reaction.user(&ctx).await.unwrap().bot {
        return Ok(());
    }

    if let ReactionType::Unicode(name) = &reaction.emoji {
        match name.as_str() {
            "🎉" => {
                if _remove {
                    giveaway::add_giveaway_entry(ctx, reaction).await?;
                } else {
                    giveaway::remove_giveaway_entry(ctx, reaction).await?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
