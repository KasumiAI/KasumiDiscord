use std::env;

use once_cell::sync::Lazy;

pub static OPENAI_KEY: Lazy<String> = Lazy::new(|| {
    env::var("OPENAI_KEY")
        .expect("Expected a OPENAI_KEY in the environment")
        .to_string()
});

pub static DEEPL_KEY: Lazy<String> = Lazy::new(|| {
    env::var("DEEPL_KEY")
        .expect("Expected a DEEPL_KEY in the environment")
        .to_string()
});

pub static DISCORD_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("DISCORD_TOKEN")
        .expect("Expected a DISCORD_TOKEN in the environment")
        .to_string()
});

pub static DATABASE_URL: Lazy<String> = Lazy::new(|| {
    env::var("DATABASE_URL")
        .expect("Expected a DATABASE_URL in the environment")
        .to_string()
});
