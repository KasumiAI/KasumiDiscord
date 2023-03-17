use std::borrow::BorrowMut;
use std::sync::Arc;

use chrono::prelude::*;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use crate::database::{Database, DbMessage};

mod bot;
mod database;
mod envs;
mod gpt;
mod prompts;
mod summarizer;
mod translate;

struct DatabaseContainer;

impl TypeMapKey for DatabaseContainer {
    type Value = Database;
}

struct BotContainer;

impl TypeMapKey for BotContainer {
    type Value = bot::Bot;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        #[cfg(debug_assertions)]
        if msg.channel_id.0 != 1085910605799633007 {
            return;
        }

        if msg.author.bot {
            return;
        }

        let database = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<DatabaseContainer>()
                .expect("Expected DatabaseContainer in TypeMap.")
                .clone()
        };

        let mut bot = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<BotContainer>()
                .expect("Expected DatabaseContainer in TypeMap.")
                .clone()
        };

        if let Some(reply) = bot
            .process_message(msg.channel_id.0, &msg.author.name, &msg.content)
            .await
        {
            if let Err(why) = msg.channel_id.say(&ctx.http, reply).await {
                warn!("Error sending reply: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // logs
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // load database
    let database = Database::new().await?;

    // create bot
    let gpt = gpt::ChatGPT::new(&envs::OPENAI_KEY);
    let google_tl = translate::GoogleTranslate::new();
    let deepl_tl = translate::DeepLTranslate::new(&envs::DEEPL_KEY);
    let bot = bot::Bot::new(database.clone(), gpt, google_tl, deepl_tl);

    // create client
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let token = envs::DISCORD_TOKEN.to_string();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await?;

    // insert data
    {
        let mut data = client.data.write().await;
        data.insert::<DatabaseContainer>(database);
        data.insert::<BotContainer>(bot);
    }

    // Start bot and wait for enter
    tokio::select! {
        _ = client.start()  => {
            info!("Client stopped");
        }
        _ = tokio::task::spawn_blocking(||{
            wait_enter()
        })  =>  {
            info!("User pressed enter");
        }
    }

    // Stop the client
    client.shard_manager.lock().await.shutdown_all().await;
    info!("Kasumi stopped");
    Ok(())
}

fn wait_enter() {
    use std::io;
    let mut user_input = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    let _ = stdin.read_line(&mut user_input);
}
