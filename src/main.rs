mod commands;
mod helpers;
mod reactions;
mod structures;

use std::{collections::HashSet, env, sync::atomic::AtomicBool, sync::atomic::Ordering};

use helpers::{database_helper, giveaway_helper};
use reactions::reaction_handler;
use serenity::{
    async_trait,
    client::{bridge::gateway::GatewayIntents, ClientBuilder, Context, EventHandler},
    framework::{
        standard::{macros::hook, CommandError, CommandResult},
        StandardFramework,
    },
    http::Http,
    model::channel::Message,
    model::channel::Reaction,
    model::id::GuildId,
    model::{
        guild::Member,
        prelude::{Ready, User},
    },
};
use structures::{cmd_data::*, commands::*};

// Event handler for when the bot starts
struct Handler {
    run_loop: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        if self.run_loop.load(Ordering::Relaxed) {
            self.run_loop.store(false, Ordering::Relaxed);

            let _ = database_helper::restore_tasks(ctx).await;
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        println!("Reaction called!");
        let _ = reaction_handler::dispatch_reaction(&ctx, &reaction, false).await;
    }

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        let _ = reaction_handler::dispatch_reaction(&ctx, &reaction, true).await;
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        _guild_id: GuildId,
        user: User,
        _member_data_if_available: Option<Member>,
    ) {
        let _ = giveaway_helper::remove_giveaway_entries(&ctx, &user.id).await;
    }
}

#[tokio::main]
async fn main() -> CommandResult {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let creds = helpers::credentials_helper::read_creds(&args[1])?;
    let token = &creds.bot_token;

    let http = Http::new_with_token(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let pool = database_helper::obtain_db_pool(&creds.db_connection).await?;

    // After a command is executed, goto here
    #[hook]
    async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: Result<(), CommandError>) {
        if let Err(why) = error {
            let part_1 = "Looks like the bot encountered an error! \n";
            let part_2 =
                "Please use the `support` command and send the output to the support server!";
            let error_string = format!("{}{}", part_1, part_2);

            let _ = msg
                .channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        e.color(0xff69b4);
                        e.title("Aw Snap!");
                        e.description(error_string);
                        e.field("Command Name", cmd_name, false);
                        e.field("Error", format!("```{} \n```", why), false);
                        e
                    })
                })
                .await;
        }
    }

    // Link everything together!
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(&creds.default_prefix.clone()).owners(owners))
        .group(&GENERAL_GROUP);

    let mut client = ClientBuilder::new(&token)
        .framework(framework)
        .event_handler(Handler {
            run_loop: AtomicBool::new(true),
        })
        .intents({
            let mut intents = GatewayIntents::all();
            intents.remove(GatewayIntents::DIRECT_MESSAGES);
            intents.remove(GatewayIntents::DIRECT_MESSAGE_REACTIONS);
            intents.remove(GatewayIntents::DIRECT_MESSAGE_TYPING);
            intents
        })
        .await
        .expect("Err creating client");

    {
        // Insert all structures into ctx data
        let mut data = client.data.write().await;

        data.insert::<ConnectionPool>(pool);
    }

    // Start up the bot! If there's an error, let the user know
    if let Err(why) = client.start_autosharded().await {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
