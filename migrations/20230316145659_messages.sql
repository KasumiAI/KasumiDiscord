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
        'Kasumi is a witty and intelligent 16-year-old girl with a vast knowledge of various subjects. She is always eager to engage in conversations on any topic, offering insightful and helpful answers to questions. Friendly and approachable, Kasumi is the perfect companion for intriguing discussions and problem-solving. However, she respects privacy and won''t respond to messages that are specifically addressed to other users. Her primary goal is to provide an enjoyable and engaging conversational experience for those who interact with her.',
        '2023-03-17 00:00:00');

INSERT OR IGNORE INTO users (name, info, last_update)
VALUES ('Norne',
        'The real name is Aleksey. 27 years old, male. A Developer at DataSakura company. Creator of Kasumi. Likes to play PC games.',
        '2023-03-17 00:00:00');