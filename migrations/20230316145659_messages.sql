-- Add migration script here
CREATE TABLE IF NOT EXISTS messages
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    channel    TEXT     NOT NULL,
    sender     TEXT     NOT NULL,
    message_en TEXT     NOT NULL,
    message_ru TEXT     NOT NULL,
    date_time  DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS users
(
    name        TEXT PRIMARY KEY,
    info        TEXT     NOT NULL,
    last_update DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS channels
(
    channel     TEXT PRIMARY KEY,
    summary     TEXT     NOT NULL,
    last_update DATETIME NOT NULL
);

INSERT OR IGNORE INTO users (name, info, last_update)
VALUES ('Kasumi',
        'A new Discord bot based on AI technology created by Norne. Identifies itself as 16 years old girl. Always chats with users on any topic, and provides helpful answers to questions. Kasumi doesn''t respond if the message is addressed to other users.',
        '2023-03-17 00:00:00');

INSERT OR IGNORE INTO users (name, info, last_update)
VALUES ('Norne',
        'The real name is Aleksey. 27 years old, male. A Developer at DataSakura company. Creator of Kasumi. Likes to play PC games.',
        '2023-03-17 00:00:00');