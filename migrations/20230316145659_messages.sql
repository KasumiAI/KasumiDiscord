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