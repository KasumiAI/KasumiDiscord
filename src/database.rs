use std::sync::Arc;

use chrono::prelude::*;
use sqlx::sqlite::SqlitePool;
use tokio::sync::Mutex;

use crate::envs;

#[derive(Debug)]
pub struct DbMessage {
    pub channel: String,
    pub sender: String,
    pub message_en: String,
    pub message_ru: String,
    pub date_time: NaiveDateTime,
}

#[derive(Debug)]
pub struct DbUser {
    pub name: String,
    pub info: String,
    pub last_update: NaiveDateTime,
}

#[derive(Debug)]
pub struct DbSummary {
    pub channel: String,
    pub summary: String,
    pub last_update: NaiveDateTime,
}

#[derive(Clone)]
pub struct Database {
    pool: Arc<Mutex<SqlitePool>>,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::error::Error> {
        let pool = SqlitePool::connect(&envs::DATABASE_URL).await?;
        Ok(Self {
            pool: Arc::new(Mutex::new(pool)),
        })
    }

    pub async fn get_messages(
        &self,
        channel: u64,
        after: NaiveDateTime,
        min_count: i64,
    ) -> Result<Vec<DbMessage>, sqlx::error::Error> {
        let messages = self.get_messages_by_date(channel, after).await?;
        let messages = if messages.len() < min_count as usize {
            self.get_messages_by_count(channel, min_count).await?
        } else {
            messages
        };
        Ok(messages)
    }

    async fn get_messages_by_date(
        &self,
        channel: u64,
        after: NaiveDateTime,
    ) -> Result<Vec<DbMessage>, sqlx::error::Error> {
        let channel = channel.to_string();
        let mut conn = self.pool.lock().await.acquire().await?;
        let mut messages = sqlx::query_as!(
            DbMessage,
            r#"
SELECT channel as "channel!", sender as "sender!",
message_en as "message_en!", message_ru as "message_ru!", date_time as "date_time!"
FROM messages
WHERE channel = ? AND date_time > ?
ORDER BY date_time DESC"#,
            channel,
            after
        )
        .fetch_all(&mut conn)
        .await?;
        messages.reverse();
        Ok(messages)
    }

    async fn get_messages_by_count(
        &self,
        channel: u64,
        count: i64,
    ) -> Result<Vec<DbMessage>, sqlx::error::Error> {
        let channel = channel.to_string();
        let mut conn = self.pool.lock().await.acquire().await?;
        let mut messages = sqlx::query_as!(
            DbMessage,
            r#"
SELECT channel as "channel!", sender as "sender!",
message_en as "message_en!", message_ru as "message_ru!", date_time as "date_time!"
FROM messages
WHERE channel = ?
ORDER BY date_time DESC
LIMIT ?"#,
            channel,
            count
        )
        .fetch_all(&mut conn)
        .await?;
        messages.reverse();
        Ok(messages)
    }

    pub async fn add_message(&self, message: &DbMessage) -> Result<(), sqlx::error::Error> {
        let mut conn = self.pool.lock().await.acquire().await?;
        sqlx::query!(
            r#"
INSERT INTO messages ( channel, sender, message_en, message_ru, date_time )
VALUES ( ?1, ?2, ?3, ?4, ?5 )"#,
            message.channel,
            message.sender,
            message.message_en,
            message.message_ru,
            message.date_time
        )
        .execute(&mut conn)
        .await?;
        Ok(())
    }

    pub async fn get_users(&self, names: &[&str]) -> Result<Vec<DbUser>, sqlx::error::Error> {
        let names = names
            .iter()
            .map(|name| format!("'{}'", name))
            .collect::<Vec<String>>()
            .join(", ");

        let mut conn = self.pool.lock().await.acquire().await?;
        let mut users = sqlx::query_as!(
            DbUser,
            r#"
SELECT name as "name!", info as "info!", last_update as "last_update!"
FROM users WHERE name IN ( ? )"#,
            names
        )
        .fetch_all(&mut conn)
        .await?;

        Ok(users)
    }

    pub async fn update_user(&self, name: &str, info: &str) -> Result<(), sqlx::error::Error> {
        let now = Utc::now().naive_utc();

        let mut conn = self.pool.lock().await.acquire().await?;
        sqlx::query!(
            r#"
INSERT OR REPLACE INTO users (name, info, last_update)
VALUES (?1, ?2, ?3);"#,
            name,
            info,
            now
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    pub async fn get_summary(&self, channel: u64) -> Result<Option<DbSummary>, sqlx::error::Error> {
        let channel = channel.to_string();

        let mut conn = self.pool.lock().await.acquire().await?;
        sqlx::query_as!(
            DbSummary,
            r#"
SELECT channel as "channel!", summary as "summary!", last_update as "last_update!"
FROM channels WHERE channel = ?"#,
            channel
        )
        .fetch_optional(&mut conn)
        .await
    }

    pub async fn update_summary(
        &self,
        channel: u64,
        summary: &str,
    ) -> Result<(), sqlx::error::Error> {
        let now = Utc::now().naive_utc();
        let channel = channel.to_string();

        let mut conn = self.pool.lock().await.acquire().await?;
        sqlx::query!(
            r#"
INSERT OR REPLACE INTO channels (channel, summary, last_update)
VALUES (?1, ?2, ?3);"#,
            channel,
            summary,
            now
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }
}
