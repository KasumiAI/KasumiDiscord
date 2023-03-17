use askama::Template;

use crate::database::DbUser;
use crate::DbMessage;

#[derive(Template)]
#[template(path = "chat.txt")]
pub struct ChatSystem<'a> {
    pub users: &'a [DbUser],
    pub date: &'a str,
    pub time: &'a str,
    pub summary: &'a str,
    pub messages: &'a [DbMessage],
}

pub const CHAT_USER_PROMPT: &str = include_str!("../templates/chat_user.txt");
