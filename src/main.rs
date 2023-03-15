use chrono::prelude::*;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use crate::database::{Database, DbMessage};

mod database;
mod envs;
mod gpt;
mod translate;

struct DatabaseContainer;

impl TypeMapKey for DatabaseContainer {
    type Value = Database;
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

        if let Err(e) = database
            .add_message(&DbMessage {
                channel: msg.channel_id.to_string(),
                sender: msg.author.name.to_string(),
                message_en: msg.content.to_string(),
                message_ru: msg.content.to_string(),
                date_time: Utc::now().naive_utc(),
            })
            .await
        {
            warn!("Failed to add message to database: {:?}", e);
        }

        if msg.content == "!last" {
            let to_send = match database
                .get_messages(msg.channel_id.0, Utc::now().naive_utc(), 10)
                .await
            {
                Ok(messages) => {
                    let mut result = String::from("Messages:\n");
                    for message in messages {
                        result.push_str(&format!("- {:?}\n", message))
                    }
                    result
                }
                Err(e) => format!("Error: {:?}", e),
            };
            if let Err(why) = msg.channel_id.say(&ctx.http, to_send).await {
                warn!("Error sending message: {:?}", why);
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
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // load database
    let database = Database::new().await?;

    // create bot
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
    }

    client.start().await?;
    Ok(())
}
