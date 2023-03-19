use regex::{Captures, Regex};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info, warn};
use tracing_appender::non_blocking::WorkerGuard;

use crate::database::{Database, DbMessage};

mod bot;
mod database;
mod envs;
mod gpt;
mod prompts;
mod summarizer;

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

        let mut bot = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<BotContainer>()
                .expect("Expected BotContainer in TypeMap.")
                .clone()
        };

        let message = Self::replace_ids(&ctx, &msg).await;

        if let Some(reply) = bot
            .process_message(msg.channel_id.0, &msg.author.name, &message)
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

impl Handler {
    // TODO: refactor this
    async fn replace_ids(ctx: &Context, msg: &Message) -> String {
        let name_re = Regex::new(r"<@(\d+?)>").unwrap();
        let users = if let Some(guild_id) = msg.guild_id {
            guild_id.members(&ctx.http, None, None).await.ok()
        } else {
            None
        };
        let message = if let Some(users) = users {
            name_re
                .replace_all(&msg.content, |m: &Captures| {
                    users
                        .iter()
                        .find(|u| u.user.id.0 == m[1].parse::<u64>().unwrap())
                        .map(|u| u.user.name.clone())
                        .unwrap_or_else(|| m[0].to_string())
                })
                .to_string()
        } else {
            msg.content.clone()
        };
        message
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // logs
    let _guard = init_logs();

    // load database
    let database = Database::new().await?;

    // create bot
    let gpt = gpt::ChatGPT::new(&envs::OPENAI_KEY);
    let bot = bot::Bot::new(database.clone(), gpt.clone());

    // create summarizer
    let summarizer = summarizer::Summarizer::new(gpt.clone(), database.clone());

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
        data.insert::<BotContainer>(bot);
    }

    // Start bot and wait for enter
    tokio::select! {
        _ = client.start()  => {
            error!("Client stopped");
        }
        _ = tokio::task::spawn_blocking(||{
            wait_enter()
        })  =>  {
            info!("User pressed enter");
        }
        _ = summarizer.start() => {
            error!("Summarizer stopped");
        }
    }

    // Stop the client
    client.shard_manager.lock().await.shutdown_all().await;
    info!("Kasumi stopped");
    Ok(())
}

fn init_logs() -> WorkerGuard {
    use tracing_subscriber::filter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::Layer;

    let file_appender = tracing_appender::rolling::hourly("logs", "prefix.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let file_subscriber = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(filter::LevelFilter::DEBUG);

    let stdio_subscriber = tracing_subscriber::fmt::layer()
        .compact()
        .with_filter(filter::filter_fn(|data| data.target() != "sqlx::query"))
        .with_filter(filter::LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(stdio_subscriber)
        .with(file_subscriber)
        .init();
    guard
}

fn wait_enter() {
    use std::io;
    let mut user_input = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    let _ = stdin.read_line(&mut user_input);
}
